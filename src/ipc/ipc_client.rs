use anyhow::{anyhow, Result};
use async_inotify::Watcher;
use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use inotify::EventMask;
use log::{debug, error, info};
use std::path::{Path, PathBuf};
use tokio::{net::UnixStream, sync::mpsc, task::JoinHandle};
use tokio_util::codec::{Framed, LengthDelimitedCodec};

pub struct IpcClient {}
impl IpcClient {
    async fn try_connect(path: &str, attempts: u32) -> Result<UnixStream> {
        for i in 0..attempts {
            match UnixStream::connect(path).await {
                Ok(unix_stream) => {
                    return Ok(unix_stream);
                }
                Err(e) => {
                    info!(
                        "Failed to connect to socket: {}. Retrying {}/{}",
                        e, attempts, i
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
        Err(anyhow!("Failed to connect to socket"))
    }
    pub async fn connect(path: &str) -> Result<Framed<UnixStream, LengthDelimitedCodec>> {
        //spawn a task to wait for the socket file to be created
        let socket_path = PathBuf::from(path);

        // check if the socket file exists and return if it does
        // TODO: there is a small chance that the file is created after this check
        // TODO 2: get rid of it and just keep retrying?
        if !socket_path.exists() {
            let socket_task: JoinHandle<Result<(), anyhow::Error>> =
                tokio::spawn(async move { wait_for_socket_file(&socket_path).await });

            info!("Waiting for socket file {} to be created", path);
            socket_task.await??;
        }

        // let (sink_tx, mut sink_rx) = mpsc::unbounded_channel::<Item>();
        // let (tx, rx) = mpsc::unbounded_channel::<Item>();

        let unix_stream = Self::try_connect(path, 30).await?;

        let stream = LengthDelimitedCodec::builder()
            .little_endian()
            // go module github.com/getlantern/framed expects 2-byte in little-endian format
            // little-endian format as length field
            .length_field_type::<u16>()
            .new_framed(unix_stream);
        Ok(stream)
    }
}

async fn wait_for_socket_file(path: &Path) -> Result<(), anyhow::Error> {
    let dir = Path::new(path).parent().unwrap();
    let mut watcher = Watcher::init();
    let mut wd = watcher.add(dir, &async_inotify::WatchMask::CREATE);
    if let Ok(mut wd) = wd {
        loop {
            if let Some(event) = watcher.next().await {
                debug!("{:?}: {:?}", event.mask(), event.path());
                if *event.mask() == EventMask::CREATE && event.path() == path {
                    info!("Socket file {} created", path.display());
                    break;
                }
            }
        }
        if let Err(e) = watcher.remove(wd) {
            return Err(anyhow!("Failed to remove watch: {}", e));
        }
    }
    Ok(())
}
