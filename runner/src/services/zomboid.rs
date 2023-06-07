use ctrlc::set_handler;
use std::process::Stdio;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{ChildStdin, Command};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

/// kills server gracefully
async fn killer(mut conditional: Receiver<i32>, stdin: Sender<Vec<u8>>) {
    conditional.recv().await.unwrap_or_default();
    // We should die after this anyways, so let's ignore errors.
    if let Err(err) = stdin.send("save\nquit".as_bytes().to_vec()).await {
        error!("Failed to inform server to quit: {:?}", err)
    }
}

async fn reader(mut source: Receiver<Vec<u8>>, mut target: ChildStdin) {
    let res = source.recv().await;
    if let Some(data) = res {
        target
            .write_all(&data)
            .await
            .expect("unable to write stdin to server");
    }
}

async fn from_stdin(destination: Sender<Vec<u8>>) {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    while let Ok(raw_line) = lines.next_line().await {
        if let Some(line) = raw_line {
            if let Err(err) = destination.send(line.as_bytes().to_vec()).await {
                info!("Unable to write user input to server: {:?}", err);
                return;
            }
        }
    }
}

/// Starts and runs the game. Kills and ends the game if killcondition returns true
async fn run_game(
    path: &str,
    condition: Receiver<i32>,
    server_parameters: Vec<String>,
) -> anyhow::Result<()> {
    // We need to run the game, read stdout until the admin prompt shows up, fullfill it, then start the read/write loop.
    let mut gamebuilder = Command::new(path);
    gamebuilder.stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for param in server_parameters {
        gamebuilder.arg(param);
    }

    let mut game = gamebuilder.spawn()?;
    let stdin = game.stdin.take().unwrap();
    // A function takes rx end of mpsc pipe, always writing it into game stdin
    // Killer function writes towards 1 tx side of pipe.
    // Another reads from software stdin if its open
    let (tx, rx) = mpsc::channel(32);

    let _t1 = tokio::spawn(killer(condition, tx.clone()));
    let _t2 = tokio::spawn(from_stdin(tx));
    let _t3 = tokio::spawn(reader(rx, stdin));
    game.wait().await?;
    Ok(())
}

/// Program entrypoint, prepare sigterm handler,
/// wrap and start the game.
pub async fn run(zomboid_path: &str, server_parameters: Vec<String>) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    set_handler(move || {
        tx.blocking_send(32).expect("Unable to kill zomboid server");
    })?;

    run_game(zomboid_path, rx, server_parameters).await?;

    Ok(())
}
