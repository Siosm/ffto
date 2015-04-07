#[macro_use] extern crate log;
extern crate url;

use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::os::unix::process::ExitStatusExt;
use std::thread;
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
                thread::spawn(move || {
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
        Ok(_) => {},
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
                    spawn_browser(browser_command, &format!("{}", u));
                }
            },
            Err(e) => info!("No URL found in '{}': {}", line, e)
        }
    }
}

fn url_valid(u: &Url) -> bool {
    (u.scheme == "http" || u.scheme == "https")
        && u.host().is_some()
}

fn spawn_browser(command: &str, url: &str) {
    debug!("Spawning process: {} {}", command, url);
    let output = match Command::new(command).arg(url).output() {
        Ok(output) => output,
        Err(e) => panic!("Failed to spawn process '{} {}': {}", command, url, e)
    };
    if output.status.success() {
        debug!("Process exited successfully");
    } else {
        match output.status.code() {
            None => {
                match output.status.signal() {
                    None => panic!("Should never happen!"),
                    Some(i) => panic!("Process received signal: {}", i)
                }
            }
            Some(i) => panic!("Process exited with status: {}", i)
        }
        info!("stdout:\n{:?}\n\nstderr:\n{:?}", output.stdout, output.stderr);
    }
}
