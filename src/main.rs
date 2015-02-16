 #![feature(io, net, core, process, std_misc)]

#[macro_use] extern crate log;
extern crate url;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::os::unix::ExitStatusExt;
use std::thread::Thread;
use url::Url;

fn main() {
    let browser_command = "firefox";
    let address = "127.0.0.1:7777";

    let listener = match TcpListener::bind(address) {
        Ok(l)  => l,
        Err(e) => panic!("Could not bind to {}: {}", address, e)
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream)  => {
                debug!("Accepted new incoming connection");
                let _ = Thread::scoped(move || {
                    debug!("Spawned new thread to handle connection");
                    handle_client(stream, browser_command)
                });
            },
            Err(e) => panic!("Could not handle incoming connection: {}", e)
        }
    }

    drop(listener);
}

fn handle_client(mut stream: TcpStream, browser_command: &str) {
    let mut message = String::new();
    match stream.read_to_string(& mut message) {
        Ok(()) => {},
        Err(e) => {
            error!("Input isn't a valid UTF-8 string: {}", e);
            return
        }
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
    let status = match Command::new(command).arg(url).status() {
        Ok(status) => status,
        Err(e) => panic!("Failed to spawn process '{} {}': {}", command, url, e)
    };
    if status.success() {
        debug!("Process exited successfully");
    } else {
        match status.code() {
            None => {
                match status.signal() {
                    None => panic!("Should never happen!"),
                    Some(i) => panic!("Process received signal: {}", i)
                }
            }
            Some(i) => panic!("Process exited with status: {}", i)
        }
    }
}
