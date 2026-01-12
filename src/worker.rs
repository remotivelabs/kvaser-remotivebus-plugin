use crate::frame;
use crate::kvaser::KvaserLinux;
use crate::ldf;
use crate::masterslave::{FrameReader, Master, Slave};
use crate::msg;
use crate::msg::HostMode;
use crate::simulator::{MasterSimulator, SlaveSimulator};
use anyhow::Result;
use socketcan::frame::AsPtr;
use socketcan::{self, Frame};
use tokio::select;
use tokio::sync::oneshot;
use tokio::time::{Duration, interval};

pub async fn run(
    vbus_id: &str,
    config: msg::Config,
    cancel_rx: oneshot::Receiver<()>,
) -> Result<()> {
    log::info!("Opening vbus {vbus_id}");

    let vbus = socketcan::tokio::CanSocket::open(vbus_id)?;

    log::info!("Opened vbus {vbus_id}");

    match config.plugin {
        msg::Plugin::Simulator(sim_config) => {
            let ldf = ldf::parse_file(&sim_config.database)?;
            let base_tick_ms = Duration::from_millis(ldf.nodes.base_tick_ms as u64);

            match sim_config.host_mode {
                HostMode::Slave => {
                    let mut slave = MasterSimulator::new(
                        &sim_config.name,
                        ldf,
                        &sim_config.schedule_table_name,
                    )?;

                    run_slave_role(&mut slave, &vbus, cancel_rx, base_tick_ms).await
                }

                HostMode::Master => {
                    let mut master = SlaveSimulator::new(&sim_config.name)?;

                    run_master_role(&mut master, &vbus, cancel_rx, base_tick_ms).await
                }
            }
        }

        msg::Plugin::Lin(lin_config) => {
            let base_tick_ms = Duration::from_millis(u64::from(lin_config.base_tick_ms));

            match lin_config.host_mode {
                HostMode::Slave => {
                    let mut slave = KvaserLinux::new_slave(
                        lin_config.name.as_ref().unwrap_or(&config.host_device),
                        &lin_config.device_id,
                        u32::from(config.baudrate),
                    )?;

                    run_slave_role(&mut slave, &vbus, cancel_rx, base_tick_ms).await
                }

                HostMode::Master => {
                    let mut master = KvaserLinux::new_master(
                        lin_config.name.as_ref().unwrap_or(&config.host_device),
                        &lin_config.device_id,
                        u32::from(config.baudrate),
                    )?;

                    run_master_role(&mut master, &vbus, cancel_rx, base_tick_ms).await
                }
            }
        }
    }
}

async fn run_slave_role(
    slave: &mut impl Slave,
    vbus: &socketcan::tokio::CanSocket,
    mut cancel_rx: oneshot::Receiver<()>,
    poll_interval: Duration,
) -> Result<()> {
    let mut ticker = interval(poll_interval);

    loop {
        select! {
            _ = ticker.tick() => {
        read_and_forward_lin_frame(slave, vbus).await?
            }

            _ = &mut cancel_rx => {
        log::info!("Requested to stop");
        return Ok(());
            }

            result = vbus.read_frame() => {
        match result {
            Ok(frame) => read_and_forward_network_slave_frame(slave, frame).await,
            Err(e) => {
            return Err(anyhow::anyhow!("Failed to read socket frame {e:?}"));
            }
        }
            }
        }
    }
}

async fn run_master_role(
    master: &mut impl Master,
    vbus: &socketcan::tokio::CanSocket,
    mut cancel_rx: oneshot::Receiver<()>,
    poll_interval: Duration,
) -> Result<()> {
    let mut ticker = interval(poll_interval);

    loop {
        select! {
            _ = ticker.tick() => {
        read_and_forward_lin_frame(master, vbus).await?
            }

            _ = &mut cancel_rx => {
        log::info!("Requested to stop");
        return Ok(());
            }

            result = vbus.read_frame() => {
        match result {
            Ok(frame) =>  read_and_forward_network_master_frame(master, frame).await?,
            Err(e) => {
            return Err(anyhow::anyhow!("Failed to read socket frame {e:?}"));
            }
        }
            }
        }
    }
}

async fn read_and_forward_lin_frame(
    reader: &mut impl FrameReader,
    vbus: &socketcan::tokio::CanSocket,
) -> Result<()> {
    if let Some(frame) = reader.try_read() {
        let name = reader.name();

        log::debug!("{name} Read LIN bus frame {frame:?}");

        if let Some(frame) = socketcan::CanDataFrame::from_raw_id(frame.id, &frame.msg) {
            log::debug!("{name} Sent frame={frame:?}");

            vbus.write_frame(socketcan::CanFrame::Data(frame)).await?;
        } else {
            log::error!("{name} Failed to build frame from {frame:?}");
        }
    }

    Ok(())
}

async fn read_and_forward_network_master_frame(
    master: &mut impl Master,
    frame: socketcan::frame::CanFrame,
) -> Result<()> {
    let name = master.name();

    match frame::parse_packet(frame.as_bytes()) {
        Ok(packet) => {
            let frame = packet.frame;

            if frame.msg.is_empty() {
                log::debug!("{name} master requests update for frame id {}", frame.id);

                master.request_update(frame.id)
            } else {
                log::debug!("{name} write master frame {frame:?}");

                master.write(&frame)
            }
        }

        Err(err) => {
            log::error!("{name} Failed to read network frame {err}");
            Ok(()) // Don't shutdown due to bad network frames
        }
    }
}

async fn read_and_forward_network_slave_frame(
    slave: &mut impl Slave,
    frame: socketcan::frame::CanFrame,
) {
    let name = slave.name();

    match frame::parse_packet(frame.as_bytes()) {
        Ok(packet) => {
            let frame = packet.frame;

            log::debug!("{name} slave update of frame {frame:?}");

            if let Err(err) = slave.update(&frame) {
                let name = slave.name();

                log::error!("{name} Failed to update frame - {err:?}");
            }
        }

        Err(err) => {
            log::error!("{name} Failed to read network frame {err}");
        }
    }
}
