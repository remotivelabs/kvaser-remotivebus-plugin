use anyhow::Result;

use crate::frame::Frame;

pub trait FrameReader {
    // Friendly name
    fn name(&self) -> &str;

    // Poll and read if a frame is available
    fn try_read(&mut self) -> Option<Frame>;
}

pub trait Slave: FrameReader {
    // Update frame payload
    fn update(&mut self, f: &Frame) -> Result<()>;
}

pub trait Master: FrameReader {
    // Write id + payload
    fn write(&mut self, frame: &Frame) -> Result<()>;

    // Request payload to be updated by owner of frame
    fn request_update(&mut self, id: u32) -> Result<()>;
}
