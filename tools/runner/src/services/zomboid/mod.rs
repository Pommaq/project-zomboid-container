/// Zomboid has implemented their start-server.sh in a shitty way
/// where it calls zomboid in a subshell, meaning our signals will not
/// reach the game itself... This works around it
mod bootstrap;
/// Handles communication between threads
mod handlers;
pub use bootstrap::patch_start_script;

use ctrlc::set_handler;
use handlers::{from_stdin, graceful_kill, reader, timeout_handler, ExitReason};
use std::process::{exit, Stdio};
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

/// Starts and runs the game. Kills and ends the game if killcondition returns true
async fn run_game(
    path: &str,
    condition: Receiver<i32>,
    server_parameters: String,
    timeout: Duration,
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
    gamebuilder.args(server_parameters.split(','));

    let mut game = gamebuilder.spawn()?;
    let stdin = game.stdin.take().unwrap();

    let (tx, rx) = mpsc::channel(32);
    let (reason_tx, mut reason_rx) = mpsc::channel(2);

    let _t1 = tokio::spawn(graceful_kill(condition, reason_tx.clone(), tx.clone()));
    if !timeout.is_zero() {
        let _t2 = tokio::spawn(timeout_handler(reason_tx, timeout));
    }
    let _t3 = tokio::spawn(from_stdin(tx));
    let _t4 = tokio::spawn(reader(rx, stdin));

    match reason_rx.recv().await {
        None => {
            error!("Reason channel closed before game has been called to exit!");
        }
        Some(reason) => {
            match reason {
                ExitReason::Standard => { /* We should just wait on the game*/ }
                ExitReason::Timeout => {
                    info!("Killing game");
                    if let Err(error) = game.kill().await {
                        error!("Unable to kill game: {}", error);
                    }
                }
            }
        }
    }
    // We will always wait for the game even if it was killed to avoid zombies (heh)
    // on Unix systems
    let code = match game.wait().await {
        Ok(status) => status.code().unwrap_or_else(|| {
            error!("Unable to extract status code from {}", status);
            255
        }),
        Err(err) => {
            error!("Unable to extract status code: {}", err);
            255
        }
    };
    info!("Exit status: {}", code);

    // let's exit hard with the same status code when we can.
    // We do this to propagate errors to caller, and to ensure our
    // other routines die.
    exit(code)
}

/// Program entrypoint, prepare sigterm handler,
/// wrap and start the game.
pub async fn run(
    zomboid_path: &str,
    server_parameters: String,
    timeout: Duration,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    set_handler(move || {
        tx.blocking_send(32).expect("Unable to kill zomboid server");
    })?;
    info!("Installed signal handler for stopping server");
    run_game(zomboid_path, rx, server_parameters, timeout).await?;
    Ok(())
}
