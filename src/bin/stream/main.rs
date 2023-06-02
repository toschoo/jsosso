use std::net::{TcpListener, TcpStream};
use std::io;
use std::str;
use std::time;
use pacosso::{Opts, ParseResult};
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

    // this parses a command of the form:
    // <command>\n\n<json>
    let parse_command = |s: &mut pacosso::Stream<TcpStream>| -> ParseResult<Command> {
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

        Ok(Command {
            name: cmd.to_string(),
            payload: j,
        })
    };

    // a parser acting on a tcp stream.
    // Note that, this way, we cannot use the socket
    // for writing because that would imply another mutual borrow.
    // There is another demo that shows how to do that.
    stream.set_read_timeout(Some(time::Duration::new(3, 0))).unwrap();
    let mut p = pacosso::Stream::new(Opts::default()
                                         .set_stream()
                                         .set_buf_size(8192)
                                         .set_buf_num(5),
                                         &mut stream);
    loop {
        match parse_command(&mut p) {
            Err(e) => {
                eprintln!("error: {:?}", e);
                break;
            },
            Ok(cmd) => {
                println!("handling command");
                handle_command(&cmd);
            },
        }
    }
}

fn handle_command(cmd: &Command) {
    println!("Command: '{}' ", cmd.name);
    println!("Payload: '{:?}' ", cmd.payload);
    println!("========");
}

fn handle_error(e: io::Error) {
    eprintln!("error: {:?}", e);
}
