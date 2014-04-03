extern mod extra;

use std::from_str::from_str;
use std::io::net::ip::SocketAddr;
use std::io::net::tcp::TcpListener;
use std::io::{Acceptor,Listener};
use std::run::{Process,ProcessOptions};

use extra::url::Url;


fn main() {
        let address = "127.0.0.1:7777";
        let addr: SocketAddr = from_str(address).expect(format!("Invalid address: {}", address));
        let listener = TcpListener::bind(addr).expect(format!("Failed to bind to: {}", address));
        let mut acceptor = listener.listen().expect("Could not listen");
        loop {
                let mut tcpStream = acceptor.accept().expect("Could not accept connection");
                let message = tcpStream.read_to_str();
                for line in message.lines() {
                        // println!("Got message: {}", line);
                        let url: Option<Url> = from_str(line);
                        match url {
                                None     => { println("No Url found") }
                                Some(u)  => {
                                        // println!("Found Url in: {}", line);
                                        if checkUrl(&u) {
                                                spawnProcess(&u, "firefox")
                                        }
                                }
                        }
                }
        }
}

fn checkUrl(u: &Url) -> bool {
        (u.scheme == ~"http" || u.scheme == ~"https" )
                && u.host != ~""
}

fn spawnProcess(u: &Url, command: &'static str) {
        // println!("{} {}", command, u.to_str());
        let pOptions = ProcessOptions::new();
        let mut child = Process::new(command, [u.to_str()], pOptions).expect("Could not fork process");
        child.finish();
}