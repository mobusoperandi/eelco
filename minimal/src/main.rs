use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    thread,
};

fn main() {
    let mut command = Command::new(std::env::var("NIX_CMD").unwrap());
    command
        .arg("repl")
        .env("PRINTF", "'ever")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = command.spawn().unwrap();
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let pipes: [(&'static str, Box<dyn Read + Send>); 2] =
        [("stdout", Box::new(stdout)), ("stderr", Box::new(stderr))];

    let _join_handles: Vec<_> = pipes
        .into_iter()
        .map(|(name, mut pipe)| {
            thread::spawn(move || {
                let mut string = String::new();
                loop {
                    let mut buffer = [0; 1000];
                    let num_bytes = pipe.read(&mut buffer).unwrap();
                    string.extend(buffer.into_iter().map(|ch| ch as char));
                    if num_bytes > 0 {
                        println!("{name} {num_bytes}:\n{string}");
                    }
                }
            })
        })
        .collect();

    //thread::sleep(time::Duration::from_secs(1));
    let mut child_stdin = child.stdin.take().unwrap();
    let written = child_stdin.write(b"1 + 1\n").unwrap();
    println!("written to stdin1: {written}");

    //thread::sleep(time::Duration::from_secs(1));
    let written = child_stdin.write(b"1 + 2\n").unwrap();
    println!("written to stdin2: {written}");

    thread::park();
}
