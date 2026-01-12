use anyhow::{Context, Result};
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Packet {
    pub frame: Frame,
}

#[derive(PartialEq)]
pub struct Frame {
    pub id: u32,
    pub msg: Vec<u8>,
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{ id: {:#x}, msg={:?}}}", self.id, self.msg)
    }
}

pub fn parse_packet(packet: &[u8]) -> Result<Packet> {
    if packet.len() >= (2 * std::mem::size_of::<u32>()) {
        let id_bytes: [u8; 4] = packet[0..std::mem::size_of::<u32>()]
            .try_into()
            .context("Failed to extract frame id")?;
        let id = u32::from_ne_bytes(id_bytes);

        let msg_len: usize = packet[std::mem::size_of::<u32>()].into();

        let mut msg: Vec<u8> = packet[(2 * std::mem::size_of::<u32>())..].to_vec();

        if msg_len < msg.len() {
            msg.truncate(msg_len);

            return Ok(Packet {
                frame: Frame { id, msg },
            });
        } else if !msg.is_empty() && msg_len > msg.len() {
            return Err(anyhow::anyhow!(
                "Length field {} in packet is larger than payload length {}",
                msg_len,
                packet.len()
            ));
        }

        Ok(Packet {
            frame: Frame { id, msg },
        })
    } else {
        Err(anyhow::anyhow!(
            "Packet is too small, only {} bytes",
            packet.len()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fail_to_parse_empty_packet_as_frame() {
        let empty_packet: [u8; 0] = [];

        assert!(parse_packet(&empty_packet).is_err())
    }

    #[test]
    fn test_parse_packet_with_can_padding() {
        let raw_packet: [u8; 16] = [0x31, 0, 0, 0, 3, 0, 0, 0, 10, 20, 30, 0, 0, 0, 0, 0];

        assert_eq!(
            parse_packet(&raw_packet).unwrap(),
            Packet {
                frame: Frame {
                    id: 0x31,
                    msg: vec![10, 20, 30]
                }
            }
        );
    }

    #[test]
    fn test_parse_request_update_frame() {
        let raw_packet: [u8; 16] = [0x31, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

        let frame = parse_packet(&raw_packet).unwrap().frame;
        assert_eq!(frame.id, 0x31);
        assert!(frame.msg.is_empty());
    }
}
