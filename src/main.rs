#[macro_use] extern crate log;
extern crate url;
extern crate rustc_serialize;
extern crate clap;

use clap::{Arg, App};
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::thread;
use url::Url;

fn main() {
    let matches = App::new("ffto")
        .version("0.0.2")
        .author("Timothée Ravier <tim@siosm.fr>")
        .about("Open URLs received as input in the default browser")
        .arg(Arg::with_name("command")
             .short("c")
             .long("command")
             .value_name("cmd")
             .help("Command executed for each URL received")
             .default_value("xdg-open"))
        .arg(Arg::with_name("listen address")
             .short("l")
             .long("listen-address")
             .value_name("addr")
             .help("Address and port to listen to")
             .default_value("127.0.0.1:7777"))
        .get_matches();

    let browser_command = matches.value_of("command").unwrap().to_string();
    let address = matches.value_of("listen address").unwrap();

    let listener = match TcpListener::bind(&address[..]) {
        Ok(l)  => l,
        Err(e) => panic!("Could not bind to {}: {}", address, e)
    };

    for stream in listener.incoming() {
        match stream {
            Ok(stream)  => {
                debug!("Accepted new incoming connection");
                let command = browser_command.clone();
                thread::spawn(move || {
                    debug!("Spawned new thread to handle connection");
                    handle_client(stream, command)
                });
            },
            Err(e) => panic!("Could not handle incoming connection: {}", e)
        }
    }

    drop(listener);
}

fn handle_client(mut stream: TcpStream, browser_command: String) {
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
                    spawn_browser(&browser_command, &format!("{}", u));
                }
            },
            Err(e) => info!("No URL found in '{}': {}", line, e)
        }
    }
}

fn url_valid(u: &Url) -> bool {
    (u.scheme() == "http" || u.scheme() == "https")
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
        info!("stdout:\n{:?}\n\nstderr:\n{:?}",
              output.stdout,
              output.stderr);
        match output.status.code() {
            None => {
                match output.status.signal() {
                    None => panic!("Should never happen!"),
                    Some(i) => panic!("Process received signal: {}", i)
                }
            }
            Some(i) => panic!("Process exited with status: {}", i)
        }
    }
}
