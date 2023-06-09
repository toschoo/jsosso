use std::net::{TcpListener, TcpStream};
use std::io::{self, Read, Write, ErrorKind};
use std::str;
use std::time;
use std::thread;
use pacosso::{Opts, ParseResult};
use pacosso::error::{ParseError};
use jsosso::parsing::{parse};
use jsosso::Json;

fn main() {
   let listener = match TcpListener::bind("127.0.0.1:6049") {
       Ok(l) => l,
       Err(e) => {
           handle_error(e);
           std::process::exit(1);
       },
   };

   for stream in listener.incoming() {
       match stream {
           Ok(s) => handle_client(s),
           Err(e) => handle_error(e),
       }
   }
}

#[derive(Debug)]
struct Command {
    name: String,
    payload: jsosso::Json,
}

impl Default for Command {
    fn default() -> Command {
        Command {
            name: "ADD".to_string(),
            payload: Json::Null,
        }
    }
}

fn handle_client(mut stream: TcpStream) {
    println!("connected!");

    stream.set_read_timeout(Some(time::Duration::new(3, 0))).unwrap();

    let mut bs = vec![];
    loop {
        bs = {
            let input = bs.chain(&mut stream);
            match handle_input(input) {
                Ok(v) => v,
                Err(ParseError::IOError(e)) if e.kind() == ErrorKind::WouldBlock => {
                     eprintln!("would block on read: {:?}", e);
                     return;
                },
                Err(e) => {
                    eprintln!("error on parse: {:?}", e);
                    return;
                },
            }
        }; // drop input

        match stream.write(&"OK".as_bytes().to_vec()) {
            Ok(2) => {
                stream.flush().expect("flush failed");
                continue;
            },
            Ok(n) => {
                eprintln!("error: cannot write 2 bytes: {}", n);
                break;
            },
            Err(e) if e.kind() == ErrorKind::Interrupted => break,
            Err(e) if e.kind() == ErrorKind::WouldBlock  => {
                eprintln!("would block on write: {:?}", e);
                thread::sleep(time::Duration::new(0, 1000));
            },
            Err(e) => {
                eprintln!("error on write: {:?}", e);
                break;
            },
        }
    }
}

fn handle_input<R: Read>(mut stream: R) -> ParseResult<Vec<u8>> {
    // this parses a command of the form:
    // <command>\n\n<json>
    let parse_command = |s: &mut pacosso::Stream<R>| -> ParseResult<Command> {
        let mut v = Vec::new();

        s.skip_whitespace()?;

        loop {
            let b = s.any_byte()?;
            if b == b'\n' {
                break;
            }
            v.push(b);
        }
        s.byte(b'\n')?;

        let cmd = match str::from_utf8(&v) {
            Ok(c) => c,
            Err(_) => return s.fail("utf8 error", Command::default()),
        };

        let j = parse(s)?;

        s.skip_whitespace()?;
        s.byte(3)?;

        Ok(Command {
            name: cmd.to_string(),
            payload: j,
        })
    };

    let mut p = pacosso::Stream::new(Opts::default()
                                         .set_infinite_stream(),
                                         &mut stream);
    let cmd = parse_command(&mut p)?;
    handle_command(&cmd);
    p.drain()
}

fn handle_command(cmd: &Command) {
    println!("Command: '{}' ", cmd.name);
    println!("Payload: '{:?}' ", cmd.payload);
    println!("========");
}

fn handle_error(e: io::Error) {
    eprintln!("error: {:?}", e);
}
