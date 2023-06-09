/// Zomboid has implemented their start-server.sh in a shitty way
/// where it calls zomboid in a subshell, meaning our signals will not
/// reach the game itself... This works around it
mod bootstrap;
/// Handles communication between threads
mod handlers;

pub use bootstrap::patch_start_script;

use handlers::{from_stdin, killer, reader};
use std::process::Stdio;
use std::time::Duration;
use tokio::sync::mpsc::{self, Receiver};
use tokio::task::spawn_blocking;
use wait_timeout::ChildExt;

/// Starts and runs the game. Returns a standard lib child and not tokio
/// since tokio is incompatible with "with-timeout" crate, but
/// std stdin can be converted to tokio stdin when we need it.
pub async fn run(path: &str, server_parameters: String) -> anyhow::Result<std::process::Child> {
    #[allow(unused_mut)] // Else we get compiler warnings on windows due to the following
    let mut gamebuilder = std::process::Command::new(path);
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::prelude::CommandExt;
        // Ensures CTRL+c in the terminal won't be sent directly to the server on UNIX
        gamebuilder.process_group(0);
    }
    gamebuilder
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    gamebuilder.args(server_parameters.split(','));

    let (rx, mut tx) = mpsc::channel(1);
    spawn_blocking(move || {
        // This is blocking
        let game = gamebuilder.spawn();

        rx.blocking_send(game)
            .expect("Unable to send child after starting it");
    });

    let game = tx.recv().await.expect("Unable to start child")?;
    Ok(game)
}

/// Ensures the game exits properly. Returns status code
pub async fn wait_for(
    mut game: std::process::Child,
    condition: Receiver<()>,
    exit_timeout: Duration,
) -> anyhow::Result<i32> {
    let stdlib_stdin = game.stdin.take().unwrap();
    let stdin = tokio::process::ChildStdin::from_std(stdlib_stdin)?;

    let (stdin_tx, stdin_rx) = mpsc::channel(32);
    let (message_writer, mut message_reader) = mpsc::channel(2);
    /*
        Main will have one receiver, from which it'll read. Killer will send something on that receiver
        once its triggered, whereas main will wait for game with a timeout. If the timeout expires, kill the game
    */
    tokio::spawn(killer(condition, stdin_tx.clone(), message_writer));

    tokio::spawn(from_stdin(stdin_tx));
    tokio::spawn(reader(stdin_rx, stdin));

    // Wait until the child is attempting to exit
    message_reader.recv().await;

    let (status_writer, mut status_reader) = mpsc::channel(1);

    spawn_blocking(move || {
        let exited = game
            .wait_timeout(exit_timeout)
            .expect("Unable to wait for child");
        let code = match exited {
            // All is good
            Some(status) => {
                info!("Server has exited");
                status.code().unwrap_or(0)
            }
            // Game did not exit
            None => {
                info!("Child timed out, killing it");
                game.kill().expect("Unable to kill child");
                // We will always wait for the game even if it was killed to avoid zombies (heh)
                // on Unix systems
                game.wait()
                    .expect("Unable to wait for child")
                    .code()
                    // -1 since it did not exit properly
                    .unwrap_or(-1)
            }
        };
        status_writer
            .blocking_send(code)
            .expect("Unable to inform main of exit status");
    });
    let exitcode = status_reader.recv().await.unwrap_or_else(|| {
        error!("status_writer exited without writing to channel!");
        -1
    });
    Ok(exitcode)
}
