extern crate extra;

use std::from_str::from_str;
use std::io::net::tcp::{TcpListener,TcpStream};
use std::io::{Acceptor,Listener};
use std::run::{Process,ProcessOptions};

use extra::url::Url;

// Note: Error handling could be improved
fn main() {
	let browserCommand = "firefox";
	let address = "127.0.0.1:7777";

	// Prepare a socket listening on localhost:7777
	let addr = from_str(address).expect(format!("Invalid address: {}", address));
	let listener = TcpListener::bind(addr).unwrap();
	let mut acceptor = listener.listen().unwrap();

	// Infinite loop to keep handling new connections.
	loop {
		let tcpStream = acceptor.accept().unwrap();
		debug!("Accepted new connection");
		spawn(proc() {
			handleClient(tcpStream, browserCommand)
		});
	}
}

// Accept a new connection and listen for URLs
fn handleClient(tcpStream: TcpStream, browserCommand: &'static str) {
	debug!("Spawned new task");

	let mut tcpStream = tcpStream;

	// Note that as soon as read_to_str() returns, everything sent
	// to the socket after this point will be discarded as once
	// we're done working with the content we've just read, the
	// tcpStream will be freed.
	let message = tcpStream.read_to_str().unwrap();

	// Iterate over the lines in the received message
	for line in message.lines() {
		debug!("Current line is: {}", line);

		// This tries to convert the line to a Url struct
		let url = from_str(line);

		// On failure it returns None; on success a valid URL
		match url {
			None     => { info!("No Url found") }
			Some(u)  => {
				debug!("Found Url in: {}", line);
				if checkUrl(&u) {
					spawnProcess(&u, browserCommand)
				}
			}
		}
	}
}

// Check that the URL is actually usable
fn checkUrl(u: &Url) -> bool {
	(u.scheme == ~"http" || u.scheme == ~"https" ) && u.host != ~""
}

// Spawn a browser to access the URL
fn spawnProcess(u: &Url, command: &'static str) {
	debug!("Spawning process {} {}", command, u.to_str());
	let pOptions = ProcessOptions::new();
	let mut child = Process::new(command, [u.to_str()], pOptions).unwrap();
	child.finish();
}
