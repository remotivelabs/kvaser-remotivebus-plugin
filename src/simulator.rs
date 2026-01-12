#![allow(clippy::new_ret_no_self)]

use crate::frame;
use crate::ldf;
use crate::masterslave::{FrameReader, Master, Slave};

use anyhow::Result;

pub struct MasterSimulator {
    name: String,
    schedule_table_name: String,
    ldf: ldf::LDF,
    table_index: i32,
    elapsed_in_table_index: u32,
}

impl MasterSimulator {
    pub fn new(name: &str, ldf: ldf::LDF, schedule_table_name: &str) -> Result<impl Slave> {
        log::info!("MasterSimulator::new {name}");

        Ok(MasterSimulator {
            name: name.to_string(),
            schedule_table_name: schedule_table_name.to_string(),
            ldf,
            table_index: 0,
            elapsed_in_table_index: 0,
        })
    }

    fn try_read(&mut self) -> Option<frame::Frame> {
        if let Some(table) = self.ldf.schedule_tables.get(&self.schedule_table_name) {
            let table_entry = &table.items[self.table_index as usize];

            if let Some(frame) = self.ldf.frames.get(&table_entry.name) {
                let elapsed_in_table_index = self.elapsed_in_table_index;

                self.elapsed_in_table_index += self.ldf.nodes.base_tick_ms;

                if self.elapsed_in_table_index >= table_entry.delay as u32 {
                    self.table_index = (self.table_index + 1) % table.items.len() as i32;
                    self.elapsed_in_table_index = 0;
                }

                if elapsed_in_table_index == 0 {
                    let msg = if frame.owner == self.ldf.nodes.master {
                        (0..frame.size).collect()
                    } else {
                        vec![]
                    };

                    return Some(frame::Frame { id: frame.id, msg });
                }
            }
        }

        None
    }
}

pub struct SlaveSimulator {
    name: String,
}

impl SlaveSimulator {
    pub fn new(name: &str) -> Result<impl Master> {
        log::info!("SlaveSimulator::new {name}");

        Ok(SlaveSimulator { name: name.into() })
    }

    fn try_read(&mut self) -> Option<frame::Frame> {
        None
    }
}

impl FrameReader for MasterSimulator {
    fn name(&self) -> &str {
        &self.name
    }

    fn try_read(&mut self) -> Option<frame::Frame> {
        MasterSimulator::try_read(self)
    }
}

impl FrameReader for SlaveSimulator {
    fn name(&self) -> &str {
        &self.name
    }

    fn try_read(&mut self) -> Option<frame::Frame> {
        SlaveSimulator::try_read(self)
    }
}

impl Slave for MasterSimulator {
    fn update(&mut self, _f: &frame::Frame) -> Result<()> {
        Ok(())
    }
}

impl Master for SlaveSimulator {
    fn write(&mut self, _frame: &frame::Frame) -> Result<()> {
        Ok(())
    }

    fn request_update(&mut self, _id: u32) -> Result<()> {
        Ok(())
    }
}

impl Drop for SlaveSimulator {
    fn drop(&mut self) {
        log::info!("SlaveSimulator::drop {}", self.name);
    }
}

impl Drop for MasterSimulator {
    fn drop(&mut self) {
        log::info!("MasterSimulator::drop {}", self.name);
    }
}
