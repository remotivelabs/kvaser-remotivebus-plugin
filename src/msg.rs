use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action", content = "bus")]
pub enum Message {
    #[serde(rename = "start")]
    StartAction(Config),
    #[serde(rename = "stop")]
    StopAction(Config),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// LIN host device name, e.g. "hostlin0"
    pub host_device: String,

    /// Baudrate for the LIN device in bits per second. Defaults to 19200 if not specified.
    #[serde(default)]
    pub baudrate: Baudrate,

    /// remotivebus-kvaser specific configuration
    pub plugin: Plugin,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum Plugin {
    /// LIN plugin configuration
    #[serde(rename = "lin")]
    Lin(Lin),

    /// Simulator plugin configuration
    #[serde(rename = "simulator")]
    Simulator(Simulator),
}

/// Custom deserialization that defaults to "lin" when type field is missing
impl<'de> Deserialize<'de> for Plugin {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        match value.get("type").and_then(|v| v.as_str()) {
            Some("lin") => {
                let lin: Lin = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Plugin::Lin(lin))
            }
            Some("simulator") => {
                let sim: Simulator =
                    serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Plugin::Simulator(sim))
            }
            Some(other) => Err(serde::de::Error::custom(format!(
                "unknown plugin type: {}",
                other
            ))),
            None => {
                // Default to "lin" when type field is missing
                let lin: Lin = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
                Ok(Plugin::Lin(lin))
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Lin {
    /// LIN driver name ("kvaser")
    pub driver: String,

    /// Optional name for the LIN interface. Defaults to device name.
    pub name: Option<String>,

    /// LIN host mode ("master" or "slave")
    pub host_mode: HostMode,

    /// LIN device id, e.g., "011121:1"
    pub device_id: String,

    /// LIN base tick in milliseconds
    #[serde(default)]
    pub base_tick_ms: BaseTick,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Simulator {
    /// Simulator driver name ("simulator")
    pub driver: String,

    /// Optional name for the simulator interface. Defaults to "simulator".
    #[serde(default = "default_simulator_name")]
    pub name: String,

    /// Simulator host mode ("master" or "slave")
    pub host_mode: HostMode,

    /// Schedule table name from LDF file
    pub schedule_table_name: String,

    /// Path to LDF database file
    pub database: String,
}

fn default_simulator_name() -> String {
    "simulator".to_string()
}

/// LIN host mode ("master" or "slave")
#[derive(Debug, Deserialize, Serialize, Copy, Clone, PartialEq)]
pub enum HostMode {
    #[serde(rename = "slave")]
    Slave,
    #[serde(rename = "master")]
    Master,
}

/// LIN Baudrate in bps. Default is 19200 bps.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct Baudrate(pub u32);

impl Default for Baudrate {
    fn default() -> Self {
        Baudrate(19_200)
    }
}

impl From<Baudrate> for u32 {
    fn from(b: Baudrate) -> u32 {
        b.0
    }
}

impl From<Baudrate> for u64 {
    fn from(b: Baudrate) -> u64 {
        b.0 as u64
    }
}

/// LIN basetick in milliseconds. Default is 5 ms.
#[derive(Debug, Copy, Clone, PartialEq, Deserialize, Serialize)]
pub struct BaseTick(pub u32);
impl Default for BaseTick {
    fn default() -> Self {
        BaseTick(5)
    }
}

impl From<BaseTick> for u32 {
    fn from(b: BaseTick) -> u32 {
        b.0
    }
}

impl From<BaseTick> for u64 {
    fn from(b: BaseTick) -> u64 {
        b.0 as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_start_action() {
        let json = include_str!("testdata/start.json");
        let message: Message = serde_json::from_str(json).expect("Failed to parse start.json");

        let Message::StartAction(config) = message else {
            panic!("Expected StartAction");
        };
        assert_eq!(config.host_device, "myhostvlin");
        assert_eq!(config.baudrate, Baudrate(19_200));

        let Plugin::Lin(lin) = config.plugin else {
            panic!("Expected Lin plugin");
        };
        assert_eq!(lin.driver, "kvaser");
        assert_eq!(lin.host_mode, HostMode::Master);
        assert_eq!(lin.device_id, "011121:1");
        assert_eq!(lin.base_tick_ms, BaseTick(5));
    }

    #[test]
    fn test_deserialize_start_action_all_options() {
        let json = include_str!("testdata/start_full.json");
        let message: Message = serde_json::from_str(json).expect("Failed to parse start_full.json");

        let Message::StartAction(config) = message else {
            panic!("Expected StartAction");
        };
        assert_eq!(config.host_device, "myhostvlin");
        assert_eq!(config.baudrate, Baudrate(9_600));

        let Plugin::Lin(lin) = config.plugin else {
            panic!("Expected Lin plugin");
        };
        assert_eq!(lin.driver, "kvaser");
        assert_eq!(lin.name, Some("MyVLIN_DEBUG".to_string()));
        assert_eq!(lin.host_mode, HostMode::Slave);
        assert_eq!(lin.device_id, "011121:2");
        assert_eq!(lin.base_tick_ms, BaseTick(5));
    }

    #[test]
    fn test_deserialize_stop_action() {
        let json = include_str!("testdata/stop.json");
        let message: Message = serde_json::from_str(json).expect("Failed to parse stop.json");

        let Message::StopAction(config) = message else {
            panic!("Expected StopAction");
        };

        assert_eq!(config.host_device, "myhostvlin");
        assert_eq!(config.baudrate, Baudrate(19_200));
        let Plugin::Lin(lin) = config.plugin else {
            panic!("Expected Lin plugin");
        };
        assert_eq!(lin.driver, "kvaser");
        assert_eq!(lin.host_mode, HostMode::Master);
        assert_eq!(lin.device_id, "011121:1");
        assert_eq!(lin.base_tick_ms, BaseTick(5));
    }

    #[test]
    fn test_serialize_start_action() {
        let config = Config {
            host_device: "testlin".to_string(),
            baudrate: Baudrate(9600),
            plugin: Plugin::Lin(Lin {
                driver: "kvaser".to_string(),
                name: None,
                host_mode: HostMode::Slave,
                device_id: "1".to_string(),
                base_tick_ms: BaseTick(5),
            }),
        };

        let message = Message::StartAction(config);
        let json = serde_json::to_string(&message).expect("Failed to serialize");

        assert!(json.contains(r#""action":"start"#));
        assert!(json.contains(r#""host_device":"testlin"#));
        assert!(json.contains(r#""baudrate":9600"#));
    }

    #[test]
    fn test_serialize_stop_action() {
        let config = Config {
            host_device: "testlin".to_string(),
            baudrate: Baudrate(9600),
            plugin: Plugin::Lin(Lin {
                driver: "kvaser".to_string(),
                name: None,
                host_mode: HostMode::Slave,
                device_id: "1".to_string(),
                base_tick_ms: BaseTick(5),
            }),
        };

        let message = Message::StopAction(config);
        let json = serde_json::to_string(&message).expect("Failed to serialize");

        assert!(json.contains(r#""action":"stop"#));
        assert!(json.contains(r#""host_device":"testlin"#));
    }

    #[test]
    fn test_roundtrip_serialization() {
        let json_start = include_str!("testdata/start.json");
        let message: Message = serde_json::from_str(json_start).expect("Failed to parse");
        let serialized = serde_json::to_string(&message).expect("Failed to serialize");
        let deserialized: Message =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        let (Message::StartAction(d1), Message::StartAction(d2)) = (message, deserialized) else {
            panic!("Roundtrip failed - variant mismatch");
        };

        assert_eq!(d1.host_device, d2.host_device);
        assert_eq!(d1.baudrate, d2.baudrate);
    }
}
