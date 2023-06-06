use anyhow::bail;
// Wrap program. We need to be able to enter arbitrary stdin and read stdout/stderr.
// Catch first admin setup, allow setting what workshop items should be added.
// use steamcmd for that.
// read from stdin and pipe it directly to zomboid.
// Catch sigterm, upon receiving it enter "save\nquit\n", then exit once it dies.
use ctrlc::set_handler;
use std::{
    io::Write,
    process::{Child, Command},
    sync::mpsc::{self, TryRecvError},
};

#[derive(Default)]
struct Zomboid {}

impl Zomboid {
    /// Starts and runs the game. Kills and ends the game if killcondition returns true
    fn run(
        mut self,
        path: &str,
        admin_name: &str,
        password: &str,
        killcondition: impl Fn() -> bool,
    ) -> anyhow::Result<()> {
        // We need to run the game, read stdout until the admin prompt shows up, fullfill it, then start the read/write loop.
        let mut game = Command::new(path).spawn()?;
        let mut stdin = game.stdin.take().unwrap();
        let stderr = game.stderr.take().unwrap();
        let stdout = game.stderr.take().unwrap();

        loop {
            if killcondition() {
                stdin.write_all("save\nquit\n".as_bytes())?;
                game.wait()?;
                break;
            }
        }
        Ok(())
    }
    fn prepare(
        steamcmd: &str,
        install_path: &str,
        workshop_ids: Option<Vec<usize>>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
pub fn run(
    steamcmd_path: &str,
    zomboid_path: &str,
    install_path: &str,
    workshop_ids: Option<Vec<usize>>,
    admin_name: &str,
    admin_password: &str,
) -> anyhow::Result<()> {
    let (tx, rx) = mpsc::channel();
    set_handler(move || {
        tx.send(1337).expect("Unable to kill zomboid server");
    })?;
    let game = Zomboid::default();

    let condition = move || {
        let time_to_die = rx.try_recv();
        if let Err(err) = time_to_die {
            if let TryRecvError::Empty = err {
                // Continue running
                return false;
            } else {
                panic!("Other end of channel was closed, unable to continue operations")
            }
        } else {
            // Kill the game
            return true;
        }
    };

    todo!()
}
