//! This module implements the server that listens for incoming messages via Unix domain socket
//! and starts/stops plugin connections accordingly.
use anyhow::Result;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use tokio::io::AsyncReadExt;
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::{mpsc, oneshot};
use tokio::task;
use tokio::time::{self, Duration};
use tokio_util::sync::CancellationToken;

use crate::msg;
use crate::worker;

#[derive(Debug)]
struct TaskResult {
    id: String,
    result: Result<()>,
}

struct Task {
    handle: task::JoinHandle<Result<()>>,
    cancel_tx: oneshot::Sender<()>,
}

pub async fn serve(socket_path: &str, shutdown: CancellationToken) -> Result<()> {
    log::info!("Listening on Unix socket {socket_path}");

    let _ = std::fs::remove_file(socket_path);

    let mut tasks = HashMap::<String, Task>::new();

    let listener = UnixListener::bind(socket_path)?;

    let (exit_tx, mut exit_rx) = mpsc::channel::<TaskResult>(128);

    loop {
        tokio::select! {
            exit_res = exit_rx.recv() => {
        if let Some(exit_res) = exit_res {
            log::debug!("Got task result ({:?}) for task with id {}", exit_res.result, exit_res.id);

            if let Some(task) = tasks.remove(&exit_res.id) {
            log::debug!("Waiting for task {} to finish", exit_res.id);

            let _ = task.handle.await;

            log::debug!("Waited for task {} to finish", exit_res.id);
            }
        }
            }

            result = listener.accept() => {
        let (mut sock, _addr) = result?;

        match read_json_from_socket::<msg::Message>(&mut sock).await {
                    Ok(action) => {
            handle_msg(action, &mut tasks, exit_tx.clone()).await;
                    }

                    Err(e) => {
            log::error!("Failed to read action {e:?}");
                    }
        }
            }
            _ = shutdown.cancelled() => {
                log::debug!("Shutdown signal received, stopping server");
                break Ok(());
            }
        }
    }
}

async fn handle_msg(
    msg: msg::Message,
    tasks: &mut HashMap<String, Task>,
    exit_tx: mpsc::Sender<TaskResult>,
) {
    log::debug!("Received new message: {:?}", msg);

    match msg {
        msg::Message::StartAction(config) => {
            handle_start_action(config, tasks, exit_tx).await;
        }
        msg::Message::StopAction(config) => {
            handle_stop_action(config, tasks).await;
        }
    }
}

async fn handle_start_action(
    config: msg::Config,
    tasks: &mut HashMap<String, Task>,
    exit_tx: mpsc::Sender<TaskResult>,
) {
    let id: String = config.host_device.clone();

    let (cancel_tx, cancel_rx) = tokio::sync::oneshot::channel();

    let task = Task {
        handle: tokio::spawn(async move {
            let id = config.host_device.clone();

            log::info!("Launched task for {id}");

            let res = worker::run(&id, config, cancel_rx).await;

            log::log!(
                if res.is_ok() {
                    log::Level::Debug
                } else {
                    log::Level::Error
                },
                "Task {id} exited - {res:?}"
            );

            if let Err(e) = exit_tx.send(TaskResult { id, result: res }).await {
                log::error!("Failed to write exit code {e}");
            }

            Ok(())
        }),
        cancel_tx,
    };

    tasks.insert(id, task);
}

async fn handle_stop_action(config: msg::Config, tasks: &mut HashMap<String, Task>) {
    if let Some(mut task) = tasks.remove(&config.host_device) {
        let _ = task.cancel_tx.send(());

        tokio::select! {
            res = &mut task.handle => {
        match res {
                    Ok(Ok(())) => {
            log::info!("Task {} exited", config.host_device);
                    }
                    Ok(Err(err)) => {
            log::error!("Task {} failed: {err}", config.host_device);
                    }
                    Err(join) => log::error!("join err: {join}"), // panicked / cancelled
        }
            }

            _ = time::sleep(Duration::from_secs(1)) => {
        task.handle.abort();
        let _ = task.handle.await;
        log::warn!("Task aborted after 1s");
            }
        }
    } else {
        log::warn!("Task {} is not running", config.host_device);
    }
}

async fn read_json_from_socket<T>(socket: &mut UnixStream) -> anyhow::Result<T>
where
    T: DeserializeOwned,
{
    let mut buf = [0u8; 2048];

    let n = socket.read(&mut buf).await?;

    let json_string = &buf[..n];

    match serde_json::from_slice::<T>(json_string) {
        Ok(v) => Ok(v),
        Err(e) => Err(anyhow::anyhow!(
            "Failed to parse json ({}) - {}",
            String::from_utf8_lossy(json_string),
            e
        )),
    }
}
