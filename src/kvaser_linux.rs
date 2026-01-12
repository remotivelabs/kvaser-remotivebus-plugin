#![allow(non_upper_case_globals)]

use crate::frame::Frame;
use crate::kvaser_raw_binding::*;
use crate::masterslave::{FrameReader, Master, Slave};
use crate::msg::HostMode;
use crate::noechoslave::NoEchoSlave;
use anyhow::Result;
use once_cell::sync::OnceCell;
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem::size_of_val;
use std::mem::zeroed;
use std::os::raw::c_void;

pub struct KvaserLinux {
    name: String,
    handle: LinHandle,
    _not_send: PhantomData<Cell<()>>,
}

static KVASER_INIT: OnceCell<Result<HashMap<String, i32>>> = OnceCell::new();

impl KvaserLinux {
    pub fn new_slave(name: &str, device_id: &str, baudrate: u32) -> Result<impl Slave> {
        Ok(NoEchoSlave::new(KvaserLinux::new(
            name,
            device_id,
            HostMode::Slave,
            baudrate,
        )?))
    }

    pub fn new_master(name: &str, device_id: &str, baudrate: u32) -> Result<impl Master> {
        KvaserLinux::new(name, device_id, HostMode::Master, baudrate)
    }

    fn new(name: &str, device_id: &str, host_mode: HostMode, baudrate: u32) -> Result<KvaserLinux> {
        if !has_mhydra_device()? {
            return Err(anyhow::anyhow!(
                "No mhydra devices found in /dev. Is the mhydra driver installed and hw connected?"
            ));
        }

        let id_map = KVASER_INIT
            .get_or_init(|| {
                log::info!("linInitializeLibrary");
                unsafe { linInitializeLibrary() };

                let id_map = make_device_id_to_channel_id_map()?;

                log::info!("Found devices {id_map:?}");

                Ok(id_map)
            })
            .as_ref()
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;

        let channel_id = id_map.get(device_id).copied().ok_or_else(|| {
            anyhow::anyhow!("Kvaser device with device_id {} not found", device_id)
        })?;

        log::info!("Opening {name} with device_id {device_id} and channel_id {channel_id}");

        let native_host_mode = if host_mode == HostMode::Slave {
            LIN_SLAVE
        } else {
            LIN_MASTER
        } as i32;

        let handle = unsafe { linOpenChannel(channel_id, native_host_mode) };

        match handle {
            handle if handle >= 0 => match Self::configure(handle, baudrate) {
                Ok(()) => Ok(KvaserLinux {
                    name: name.to_string(),
                    handle,
                    _not_send: PhantomData,
                }),

                Err(err) => {
                    unsafe { linClose(handle) };
                    Err(anyhow::anyhow!("configuration of {} failed: {}", name, err))
                }
            },
            LinStatus_linERR_NOTFOUND => Err(anyhow::anyhow!(
                "Bus {} ({}) not found - is the LIN bus powered up with 12 V? (error {})",
                channel_id,
                name,
                handle
            )),

            _ => Err(anyhow::anyhow!(
                "Failed to open channel {} ({}) (error {})",
                channel_id,
                name,
                handle
            )),
        }
    }

    fn configure(handle: LinHandle, baudrate: u32) -> Result<()> {
        unsafe {
            linBusOff(handle);

            let mut res = linBusOff(handle);
            if res != LinStatus_linOK {
                return Err(anyhow::anyhow!("linBusOff failed with {}", res));
            }

            res = linSetBitrate(handle, baudrate);
            if res != LinStatus_linOK {
                return Err(anyhow::anyhow!("linSetBitrate failed with {}", res));
            }

            res = linBusOn(handle);
            if res != LinStatus_linOK {
                return Err(anyhow::anyhow!("linBusOn failed with {}", res));
            }

            res = linSetupLIN(handle, LIN_VARIABLE_DLC | LIN_ENHANCED_CHECKSUM, baudrate);
            if res != LinStatus_linOK {
                return Err(anyhow::anyhow!("linSetupLIN failed with {}", res));
            }
        }

        Ok(())
    }
}

impl FrameReader for KvaserLinux {
    fn name(&self) -> &str {
        &self.name
    }

    fn try_read(&mut self) -> Option<Frame> {
        let mut msg: [u8; 16] = [0; 16];

        let mut id: u32 = 0;
        let mut msg_len: u32 = 0;

        let mut info: LinMessageInfo = unsafe { zeroed() };

        let res = unsafe {
            linReadMessage(
                self.handle,
                &mut id,
                msg.as_mut_ptr() as *mut c_void,
                &mut msg_len,
                std::ptr::null_mut(),
                &mut info,
            )
        };

        if res != LinStatus_linOK {
            return None;
        }

        let mut msg = msg.to_vec();
        msg.truncate(msg_len as usize);

        Some(Frame { id, msg })
    }
}

impl Slave for KvaserLinux {
    fn update(&mut self, f: &Frame) -> Result<()> {
        let msg_data: *const c_void = f.msg.as_ptr() as *const c_void;

        log::debug!("Updating frame {:x} with {:?}", f.id, f.msg);

        let res = unsafe { linUpdateMessage(self.handle, f.id, msg_data, f.msg.len() as u32) };

        if res != LinStatus_linOK {
            return Err(anyhow::anyhow!(
                "update failed to update using frame {:?} (err {})",
                f,
                res
            ));
        }

        Ok(())
    }
}

impl Master for KvaserLinux {
    fn write(&mut self, frame: &Frame) -> Result<()> {
        let msg_data: *const c_void = frame.msg.as_ptr() as *const c_void;

        let res =
            unsafe { linWriteMessage(self.handle, frame.id, msg_data, frame.msg.len() as u32) };

        if res != LinStatus_linOK {
            return Err(anyhow::anyhow!(
                "Failed to write frame {:?} (err {})",
                frame,
                res
            ));
        }

        Ok(())
    }

    fn request_update(&mut self, id: u32) -> Result<()> {
        let res = unsafe { linRequestMessage(self.handle, id) };

        if res != LinStatus_linOK {
            return Err(anyhow::anyhow!(
                "Failed to request payload for id {:?} (err {})",
                id,
                res
            ));
        }

        Ok(())
    }
}

impl Drop for KvaserLinux {
    fn drop(&mut self) {
        log::info!("KvaserLinux::drop {}", self.name);

        unsafe { linClose(self.handle) };
    }
}

fn make_device_id_to_channel_id_map() -> Result<HashMap<String, i32>> {
    let mut num_chans: i32 = 0;
    let res = unsafe { canGetNumberOfChannels(&mut num_chans as *mut i32) };

    if res != canStatus_canOK {
        return Err(anyhow::anyhow!(
            "canGetNumberOfChannels failed with {}",
            res
        ));
    }

    log::info!("Found {num_chans} kvaser channels");
    struct SerialAndChan {
        raw_serial: [u32; 2],
        local_channel: u32,
    }

    let mut prev_serial_and_chan: Option<SerialAndChan> = None;
    let results: HashMap<String, i32> = (0..num_chans)
        .map(|i| {
            let mut raw_serial: [u32; 2] = [0; 2];
            unsafe {
                canGetChannelData(
                    i,
                    canCHANNELDATA_CARD_SERIAL_NO as i32,
                    raw_serial.as_mut_ptr() as *mut c_void,
                    size_of_val(&raw_serial),
                )
            };

            if let Some(prev_serial_and_chan) = prev_serial_and_chan.as_mut() {
                if prev_serial_and_chan.raw_serial == raw_serial {
                    prev_serial_and_chan.local_channel += 1
                } else {
                    *prev_serial_and_chan = SerialAndChan {
                        raw_serial,
                        local_channel: 1,
                    };
                }
            }

            let prev_serial_and_chan = prev_serial_and_chan.get_or_insert(SerialAndChan {
                raw_serial,
                local_channel: 1,
            });

            (
                format!(
                    "{}{}:{}",
                    raw_serial[1], raw_serial[0], prev_serial_and_chan.local_channel
                ),
                i,
            )
        })
        .collect();

    Ok(results)
}

fn has_mhydra_device() -> std::io::Result<bool> {
    let entries = std::fs::read_dir("/dev")?;

    for entry in entries {
        let entry = entry?;
        if let Some(name) = entry.file_name().to_str()
            && name.starts_with("mhydra")
        {
            return Ok(true);
        }
    }

    Ok(false)
}
