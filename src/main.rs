//! Main entry point for remotivebus-kvaser service.
//!
//!! Example:
//! ```
//! remotivebus-kvaser -p /run/remotivebus/plugins/kvaser.sock
//! ```
use anyhow::Result;
use clap::Parser;
use tokio::signal::{
    self,
    unix::{SignalKind, signal},
};
use tokio_util::sync::CancellationToken;

use kvaser_remotivebus_plugin::logging;
use kvaser_remotivebus_plugin::server;

#[derive(Parser, Debug)]
#[command(
    name = "kvaser-remotivebus-plugin",
    version,
    about = "Kvaser RemotiveBus Plugin service"
)]
struct CliArgs {
    #[arg(
        short = 'p',
        default_value = "/run/remotivebus/plugins/kvaser.sock",
        long,
        help = "Unix domain socket path"
    )]
    plugin_socket_path: String,

    #[arg(short = 'l', value_parser = clap::value_parser!(log::LevelFilter), long, help = "Log level")]
    loglevel: Option<log::LevelFilter>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli_args = CliArgs::parse();
    logging::setup_log(cli_args.loglevel);

    // handle SIGINT and SIGTERM for graceful shutdown
    let shutdown = CancellationToken::new();
    let shutdown_signal = shutdown.clone();
    tokio::spawn(async move {
        wait_for_shutdown_signal().await;
        shutdown_signal.cancel();
    });

    server::serve(&cli_args.plugin_socket_path, shutdown.clone()).await?;

    log::info!("Server shut down gracefully");

    Ok(())
}

async fn wait_for_shutdown_signal() {
    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to set up SIGTERM handler");

    tokio::select! {
      _ = sigterm.recv() => {
          log::info!("Received SIGTERM, shutting down");
      }
      _ = signal::ctrl_c() => {
          log::info!("Received SIGINT (Ctrl-C), shutting down");
      }
    }
}
