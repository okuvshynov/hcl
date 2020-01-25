use std::io::{Error, ErrorKind};
use std::process::{Command, Stdio};

fn spawn(cmd: &str) -> std::io::Result<std::process::Child> {
    if cfg!(unix) {
        return Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .arg(cmd)
            .stdout(Stdio::piped())
            .spawn();
    }
    panic!("unsupported environment");
}

pub fn spawned_stdout(cmd: &str) -> std::io::Result<std::process::ChildStdout> {
    let child = spawn(cmd)?;
    child
        .stdout
        .ok_or_else(|| Error::new(ErrorKind::NotFound, "no stdout in child process"))
}
