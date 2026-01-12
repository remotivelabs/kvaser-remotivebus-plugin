//! Entrypoint for sending start and stop messages to a running instance of remotivebus-kvaser.
//!
//! Example:
//! ```
//! cargo run --bin send-msg -- -p /tmp/kvaser.sock -m start.json
//! ```
use anyhow::{Context, Result};
use clap::Parser;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;

use kvaser_remotivebus_plugin::msg;

#[derive(Parser, Debug)]
#[command(
    name = "send-msg",
    version,
    about = "Send message to remotivebus-kvaser service"
)]
struct CliArgs {
    #[arg(
        short = 'p',
        default_value = "/run/remotivebus/plugins/kvaser.sock",
        long,
        help = "Unix domain socket path"
    )]
    plugin_socket_path: String,

    #[arg(short = 'm', long, help = "Path to message JSON to be sent to server")]
    msg_path: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    run(&cli_args.plugin_socket_path, &cli_args.msg_path).await?;
    Ok(())
}

/// Connects to the remotivebus-kvaser server via Unix domain socket and sends the specified message.
/// The socket must already be created by the server.
async fn run(plugin_socket_path: &str, msg_path: &str) -> Result<()> {
    let raw_msg = fs::read_to_string(&msg_path)
        .await
        .context(format!("Failed to read {}", msg_path))?;

    let msg: msg::Message =
        serde_json::from_str(&raw_msg).context("Failed to parse message json")?;

    let mut stream = UnixStream::connect(plugin_socket_path)
        .await
        .context("Failed to connect. Is remotivebus-kvaser server running?")?;

    let bytes = serde_json::to_vec(&msg).context("Failed to serialize message")?;

    stream.write_all(&bytes).await?;

    Ok(())
}
