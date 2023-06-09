use std::time::Duration;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::sleep;

pub enum ExitReason {
    Standard,
    Timeout,
}

pub async fn timeout_handler(reason_channel: Sender<ExitReason>, timeout: Duration) {
    sleep(timeout).await;
    info!("Timed out, informing main to kill the process...");
    if let Err(err) = reason_channel.send(ExitReason::Timeout).await {
        error!("Unable to transmit kill: {}", err);
    }
}

pub async fn graceful_kill(
    mut conditional: Receiver<i32>,
    reason_channel: Sender<ExitReason>,
    stdin: Sender<Vec<u8>>,
) {
    conditional.recv().await.unwrap_or_default();
    info!("Signal recieved, stopping server gracefully...");
    // We should die after this anyways, so let's ignore errors.
    if let Err(err) = stdin.send("quit\n".as_bytes().to_vec()).await {
        error!("Failed to inform server to quit: {}", err)
    }
    if let Err(err) = reason_channel.send(ExitReason::Standard).await {
        error!("Failed to inform backend regarding exit reason: {}", err)
    }
}

pub async fn reader(mut source: Receiver<Vec<u8>>, mut target: tokio::process::ChildStdin) {
    loop {
        let res = source.recv().await;
        if let Some(data) = res {
            debug!("Writing row {:x?} to child", &data);
            target
                .write_all(&data)
                .await
                .expect("unable to write stdin to server");
        }
    }
}

pub async fn from_stdin(destination: Sender<Vec<u8>>) {
    let stdin = io::stdin();
    let reader = io::BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Ok(raw_line) = lines.next_line().await {
        if let Some(line) = raw_line {
            if let Err(err) = destination
                .send(format!("{}\n", line).into_bytes().to_vec())
                .await
            {
                info!("Unable to write user input to server: {:?}", err);
                return;
            }
        }
    }
}
