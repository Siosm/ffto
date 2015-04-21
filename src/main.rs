#[macro_use] extern crate log;
extern crate url;
extern crate rustc_serialize;
extern crate docopt;

use docopt::Docopt;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::os::unix::process::ExitStatusExt;
use std::process::Command;
use std::process::exit;
use std::thread;
use url::Url;

// Docopt usage string
static USAGE: &'static str = "
Usage: ffto [--command=<cmd>] [--address=<addr>]
       ffto --help

Options:
    -h, --help        Show this message.
    --command=<cmd>   Command executed for each URL received [default: firefox].
    --address=<addr>  Address and port to listen to [default: 127.0.0.1:7777].
";

#[derive(RustcDecodable)]
struct Args {
    flag_help: bool,
    flag_command: Option<String>,
    flag_address: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    if args.flag_help {
        print!("{}", USAGE);
        exit(0);
    }

    let browser_command = match args.flag_command {
        Some(cmd) => cmd.clone(),
        None => format!("firefox")
    };
    let address = match args.flag_address {
        Some(addr) => addr,
        None => format!("129.0.0.1:7777")
    };

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
                    handle_client(stream, &command)
                });
            },
            Err(e) => panic!("Could not handle incoming connection: {}", e)
        }
    }

    drop(listener);
}

fn handle_client(mut stream: TcpStream, browser_command: &String) {
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

fn spawn_browser(command: &String, url: &String) {
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
