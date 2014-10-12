#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate url;

use std::from_str::from_str;
use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Acceptor,Listener,Command};
use std::string::String;
use url::Url;

// Note: Error handling could be improved
fn main() {
    let browser_command = "firefox";
    let address = "127.0.0.1";
    let port = 7777;

    // Prepare a socket listening on localhost:7777
    let listener = TcpListener::bind(address, port).unwrap();
    let mut acceptor = listener.listen().unwrap();

    // Infinite loop to keep handling new connections.
    loop {
        let tcp_stream = acceptor.accept().unwrap();
        debug!("Accepted new connection");
        spawn(proc() {
            handle_client(tcp_stream, browser_command)
        });
    }
}

// Accept a new connection and listen for URLs
fn handle_client(stream: TcpStream, browser_command: &'static str) {
    debug!("Spawned new task");

    let mut tcp_stream = stream;

    // Note that as soon as read_to_str() returns, everything sent
    // to the socket after this point will be discarded as once
    // we're done working with the content we've just read, the
    // tcpStream will be freed.
    let message = tcp_stream.read_to_string().unwrap();

    // Iterate over the lines in the received message
    for line in message.as_slice().split('\n') {
        debug!("Current line is: {}", line);

        // This tries to convert the line to a Url struct
        let url = from_str(line);

        // On failure it returns None; on success a valid URL
        match url {
            None     => { info!("No Url found") }
            Some(u)  => {
                debug!("Found Url in: {}", line);
                if check_url(&u) {
                    spawn_process(&u, browser_command)
                }
            }
        }
    }
}

// Check that the URL is actually usable
fn check_url(u: &Url) -> bool {
    (u.scheme == String::from_str("http") ||
     u.scheme == String::from_str("https"))
        && !u.host.is_empty()
}

// Spawn a browser to access the URL
fn spawn_process(u: &Url, command: &'static str) {
    debug!("Spawning process: {} {}", command, u);
    let url = format!("{}", u);
    let mut child = match Command::new(command).arg(url).spawn() {
        Ok(child) => child,
        Err(e)    => fail!("Failed to spawn process: {}. {}", command, e),
    };
    child.wait().unwrap().success();
}
