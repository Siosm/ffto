#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate url;

use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Acceptor,Listener,Command};
use std::io::process::ProcessExit;
use url::Url;

fn main() {
    let browser_command = "firefox";
    let address = "127.0.0.1:7777";

    let listener = match TcpListener::bind(address) {
        Ok(l)  => l,
        Err(e) => panic!("Could not bind to {}: {}", address, e)
    };
    let mut acceptor = match listener.listen() {
        Ok(a)  => a,
        Err(e) => panic!("Could not listen for connections: {}", e)
    };

    loop {
        match acceptor.accept() {
            Ok(s)  => {
                debug!("Accepted new incoming connection");
                spawn(move || {
                    debug!("Spawned new task");
                    handle_client(s, browser_command)
                });
            },
            Err(e) => panic!("Could not handle incoming connection: {}", e)
        }
    }
}

fn handle_client(stream: TcpStream, browser_command: &str) {
    let mut tcp_stream = stream;

    // FIXME: do we need to read in a loop?
    let message = match tcp_stream.read_to_string() {
        Ok(s)  => s,
        Err(e) => panic!("Input isn't a valid UTF-8 string: {}", e)
    };

    for line in message.split('\n') {
        debug!("Current line is: {}", line);

        match Url::parse(line) {
            Ok(u)  => {
                debug!("Found URL in: {}", line);
                if url_valid(&u) {
                    debug!("Found URL is valid");
                    spawn_browser(browser_command, format!("{}", u).as_slice());
                }
            },
            Err(e) => info!("No URL found in '{}': {}", line, e)
        }
    }
}

fn url_valid(u: &Url) -> bool {
    (u.scheme.as_slice() == "http" || u.scheme.as_slice() == "https")
        && u.host().is_some()
}


fn spawn_browser(command: &str, url: &str) {
    debug!("Spawning process: {} {}", command, url);
    let mut child = match Command::new(command).arg(url).spawn() {
        Ok(child) => child,
        Err(e)    => panic!("Failed to spawn process '{}': {}", command, e)
    };
    match child.wait() {
        Ok(p)  => {
            if p.success() {
                debug!("Process exited successfully");
            } else {
                match p {
                    ProcessExit::ExitStatus(s) => panic!("Process exited with status: {}", s),
                    ProcessExit::ExitSignal(s) => panic!("Process received signal: {}", s)
                }
            }
        },
        Err(e) => panic!("This should never happen! ({})", e)
    }
}
