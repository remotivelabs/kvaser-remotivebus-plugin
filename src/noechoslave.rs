use crate::frame::Frame;
use crate::masterslave::{FrameReader, Slave};

use anyhow::Result;
use std::collections::HashSet;

pub struct NoEchoSlave<S: Slave> {
    target: S,

    updated_frames: HashSet<u32>,
}

impl<S: Slave> NoEchoSlave<S> {
    pub fn new(target: S) -> Self {
        NoEchoSlave {
            target,
            updated_frames: HashSet::new(),
        }
    }
}

impl<S: Slave> FrameReader for NoEchoSlave<S> {
    fn name(&self) -> &str {
        self.target.name()
    }

    fn try_read(&mut self) -> Option<Frame> {
        let f = self.target.try_read();
        if let Some(mut f) = f {
            if self.updated_frames.contains(&f.id) {
                f.msg.clear();
            }

            return Some(f);
        }

        f
    }
}

impl<S: Slave> Slave for NoEchoSlave<S> {
    fn update(&mut self, f: &Frame) -> Result<()> {
        self.updated_frames.insert(f.id);

        self.target.update(f)
    }
}
