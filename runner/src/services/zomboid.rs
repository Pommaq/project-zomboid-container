use ctrlc::set_handler;
use std::process::Stdio;
use tokio::io::{AsyncWriteExt };
use tokio::process::{ChildStdin, Command};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;

/// kills server gracefully
async fn killer(mut conditional: Receiver<i32>, mut stdin: ChildStdin) {
    conditional.recv().await.unwrap_or_default();
    stdin.write_all("save\nquit".as_bytes()).await.unwrap();
}

/// Starts and runs the game. Kills and ends the game if killcondition returns true
async fn run_game(path: &str, password: &str, condition: Receiver<i32>) -> anyhow::Result<()> {
    // We need to run the game, read stdout until the admin prompt shows up, fullfill it, then start the read/write loop.
    let mut game = Command::new(path)
        .arg("-adminpassword")
        .arg(password)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let stdin = game.stdin.take().unwrap();

    // Start the tasks
    let _t1 = tokio::spawn(killer(condition, stdin));

    game.wait().await?;
    Ok(())
}

/// Program entrypoint, prepare sigterm handler,
/// wrap and start the game.
pub async fn run(zomboid_path: &str, admin_password: &str) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel(32);
    set_handler(move || {
        tx.blocking_send(32).expect("Unable to kill zomboid server");
    })?;

    run_game(zomboid_path, admin_password, rx).await?;

    Ok(())
}
