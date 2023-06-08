use ctrlc::set_handler;
use std::process::{exit, Stdio};
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

/// kills server gracefully
async fn killer(mut conditional: Receiver<i32>, stdin: Sender<Vec<u8>>) {
    conditional.recv().await.unwrap_or_default();
    info!("Signal recieved, stopping server gracefully...");
    // We should die after this anyways, so let's ignore errors.
    if let Err(err) = stdin.send("quit\n".as_bytes().to_vec()).await {
        error!("Failed to inform server to quit: {}", err)
    }
}

async fn reader(mut source: Receiver<Vec<u8>>, mut target: tokio::process::ChildStdin) {
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

async fn from_stdin(destination: Sender<Vec<u8>>) {
    let stdin = io::stdin();
    let reader = BufReader::new(stdin);
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

/// Starts and runs the game. Kills and ends the game if killcondition returns true
async fn run_game(
    path: &str,
    condition: Receiver<i32>,
    server_parameters: String,
) -> anyhow::Result<()> {
    // First we must create a stdlib Command so we can set the GID on UNIX,
    // since it's unstable on Tokio.
    #[allow(unused_mut)] // Else we get compiler warnings on windows
    let mut raw_gamebuilder = std::process::Command::new(path);
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::prelude::CommandExt;
        // Ensures CTRL+c in the terminal won't be sent directly to the server on UNIX
        raw_gamebuilder.process_group(0);
    }
    // Now let's convert it to the tokio variant and continue.
    let mut gamebuilder = tokio::process::Command::from(raw_gamebuilder);
    gamebuilder
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    for param in server_parameters.split(',') {
        gamebuilder.arg(param);
    }

    let mut game = gamebuilder.spawn()?;
    let stdin = game.stdin.take().unwrap();

    let (tx, rx) = mpsc::channel(32);

    let _t1 = tokio::spawn(killer(condition, tx.clone()));
    let _t2 = tokio::spawn(from_stdin(tx));
    let _t3 = tokio::spawn(reader(rx, stdin));
    let status = game.wait().await?;
    info!("{}", status);

    // let's exit hard with the same status code.
    // We do this to propagate errors to caller, and to ensure our
    // other routines die.
    exit(status.code().unwrap());
}

/// Program entrypoint, prepare sigterm handler,
/// wrap and start the game.
pub async fn run(zomboid_path: &str, server_parameters: String) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    set_handler(move || {
        tx.blocking_send(32).expect("Unable to kill zomboid server");
    })?;
    info!("Installed signal handler for stopping server");
    run_game(zomboid_path, rx, server_parameters).await?;

    Ok(())
}
