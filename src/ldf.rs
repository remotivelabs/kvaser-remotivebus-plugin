use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, PartialEq)]
pub struct Header {
    pub baudrate: u32,
}

#[derive(Debug, PartialEq)]
pub struct Nodes {
    pub master: String,
    pub base_tick_ms: u32,
}

#[derive(Debug, PartialEq)]
pub struct Frame {
    pub name: String,
    pub id: u32,
    pub owner: String,
    pub size: u8,
}

#[derive(Debug, PartialEq)]
pub struct ScheduleTableItem {
    pub name: String,
    pub delay: f32,
}

#[derive(Debug, PartialEq)]
pub struct ScheduleTable {
    pub name: String,
    pub items: Vec<ScheduleTableItem>,
}

#[derive(Debug, PartialEq)]
pub struct LDF {
    pub header: Header,
    pub nodes: Nodes,
    pub frames: HashMap<String, Frame>,
    pub schedule_tables: HashMap<String, ScheduleTable>,
}

fn parse_ldf_lines<I>(lines: &mut I) -> Result<LDF>
where
    I: Iterator<Item = io::Result<String>>,
{
    let baudrate_re = Regex::new(r"^LIN_speed = ([0-9]+\.[0-9]+) kbps;")?;

    let mut ldf = LDF {
        header: Header { baudrate: 0 },
        nodes: Nodes {
            base_tick_ms: 0,
            master: "".to_string(),
        },
        frames: HashMap::new(),
        schedule_tables: HashMap::new(),
    };

    while let Some(line) = lines.next() {
        let line = line?;

        match line.as_str() {
            "Nodes {" => {
                ldf.nodes = parse_nodes(lines)?;
            }
            "Frames {" => {
                ldf.frames = parse_frames(lines)?;
            }
            "Schedule_tables {" => ldf.schedule_tables = parse_schedule_tables(lines)?,
            _ => {
                if let Some(m) = baudrate_re.captures(&line) {
                    let baudrate: f32 = m[1].parse()?;
                    ldf.header.baudrate = (baudrate * 1000.0) as u32;
                }
            }
        }
    }

    Ok(ldf)
}

fn parse_nodes<I>(lines: &mut I) -> Result<Nodes>
where
    I: Iterator<Item = io::Result<String>>,
{
    let mut nodes = Nodes {
        master: "".to_string(),
        base_tick_ms: 0,
    };

    let master_re = Regex::new(r"^\s*Master: ([A-Za-z0-9]+), ([0-9]+\.[0-9]+) ms")?;

    for line in lines.by_ref() {
        let line = line?;

        if line == "}" {
            return Ok(nodes);
        } else if let Some(m) = master_re.captures(&line) {
            let master_name = m[1].to_string();
            let base_tick: f32 = m[2].parse()?;

            nodes.master = master_name;
            nodes.base_tick_ms = base_tick as u32
        }
    }

    Err(anyhow::anyhow!("Nodes section never ended!"))
}

fn parse_frames<I>(lines: &mut I) -> Result<HashMap<String, Frame>>
where
    I: Iterator<Item = io::Result<String>>,
{
    let mut frames: HashMap<String, Frame> = HashMap::new();

    let frame_re = Regex::new(r"^\s*([A-Za-z0-9]+):\s+0x([0-9A-Fa-f]+),\s+(\w+),\s+(\d+)\s*\{")?;

    for line in lines.by_ref() {
        let line = line?;

        if let Some(m) = frame_re.captures(&line) {
            let name = m[1].to_string();

            frames.insert(
                name.clone(),
                Frame {
                    name,
                    id: u32::from_str_radix(&m[2], 16)?,
                    owner: m[3].to_string(),
                    size: m[4].parse()?,
                },
            );
        } else if line == "}" {
            return Ok(frames);
        }
    }

    Err(anyhow::anyhow!("Frames section never ended!"))
}

fn parse_schedule_tables<I>(lines: &mut I) -> Result<HashMap<String, ScheduleTable>>
where
    I: Iterator<Item = io::Result<String>>,
{
    let mut schedule_tables: HashMap<String, ScheduleTable> = HashMap::new();

    while let Some(line) = lines.next() {
        let line = line?;

        if line.ends_with("{") {
            let table = parse_schedule_table(lines, &line)?;

            schedule_tables.insert(table.name.clone(), table);
        } else if line == "}" {
            return Ok(schedule_tables);
        }
    }

    Err(anyhow::anyhow!("Schedule_Tables section never ended!"))
}

fn parse_schedule_table<I>(lines: &mut I, line: &str) -> Result<ScheduleTable>
where
    I: Iterator<Item = io::Result<String>>,
{
    let name_re = Regex::new(r"^\s*([A-Za-z0-9]+)\s\{")?;
    let table_entry_re = Regex::new(r"^\s*([A-Za-z0-9]+)\sdelay\s([0-9]+\.[0-9]+) ms;")?;

    let mut schedule_table = ScheduleTable {
        name: name_re
            .captures(line)
            .ok_or(anyhow::anyhow!("Schedule table name is missing"))?[1]
            .to_string(),
        items: Vec::new(),
    };

    for line in lines.by_ref() {
        let line = line?;

        if line.ends_with("}") {
            return Ok(schedule_table);
        } else if let Some(m) = table_entry_re.captures(&line) {
            schedule_table.items.push(ScheduleTableItem {
                name: m[1].to_string(),
                delay: m[2].parse()?,
            })
        }
    }

    Err(anyhow::anyhow!("Schedule_Table section never ended!"))
}

pub fn parse_file(ldf_path: &str) -> Result<LDF> {
    let file = File::open(ldf_path)?;
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    parse_ldf_lines(&mut lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    #[test]
    fn test_should_parse_ldf() {
        let ldf = parse_file("src/testdata/mini.ldf").unwrap();
        assert_eq!(ldf.header, Header { baudrate: 19200 });

        assert_eq!(
            ldf.nodes,
            Nodes {
                master: "TheMaster".to_string(),
                base_tick_ms: 5
            }
        );

        assert_eq!(
            ldf.frames,
            HashMap::from([
                (
                    "Slave1LinFrame01".to_string(),
                    Frame {
                        name: "Slave1LinFrame01".to_string(),
                        id: 49,
                        owner: "Slave1".to_string(),
                        size: 7
                    }
                ),
                (
                    "MasterLinFrame01".to_string(),
                    Frame {
                        name: "MasterLinFrame01".to_string(),
                        id: 50,
                        owner: "TheMaster".to_string(),
                        size: 8
                    }
                ),
                (
                    "Slave2LinFrame02".to_string(),
                    Frame {
                        name: "Slave2LinFrame02".to_string(),
                        id: 50,
                        owner: "Slave2".to_string(),
                        size: 8
                    }
                )
            ])
        );

        assert_eq!(
            ldf.schedule_tables,
            HashMap::from([
                (
                    "MiniLinRequestScheduleTable".to_string(),
                    ScheduleTable {
                        name: "MiniLinRequestScheduleTable".to_string(),
                        items: vec![ScheduleTableItem {
                            name: "MasterReq".to_string(),
                            delay: 15.0
                        }]
                    }
                ),
                (
                    "MiniLinResponseScheduleTable".to_string(),
                    ScheduleTable {
                        name: "MiniLinResponseScheduleTable".to_string(),
                        items: vec![ScheduleTableItem {
                            name: "SlaveResp".to_string(),
                            delay: 15.0
                        }]
                    }
                ),
                (
                    "TheScheduleTable01".to_string(),
                    ScheduleTable {
                        name: "TheScheduleTable01".to_string(),
                        items: vec![
                            ScheduleTableItem {
                                name: "Slave1LinFrame01".to_string(),
                                delay: 15.0
                            },
                            ScheduleTableItem {
                                name: "Slave2LinFrame02".to_string(),
                                delay: 10.0
                            },
                            ScheduleTableItem {
                                name: "MasterLinFrame01".to_string(),
                                delay: 10.0
                            }
                        ]
                    }
                )
            ])
        );
    }
}
