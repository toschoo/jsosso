use std::io::prelude::*;
use std::io;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::thread;
use std::time;
use std::env;
use jsosso::arbitrary::{make_n_arbitrary};

// command line arguments
#[derive(Debug)]
struct Config {
    acks: bool,
    port: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            acks: false,
            port: "6049".to_string(),
        }
    }
}

fn usage(name: &str) -> String {
   format!(r"Usage {}: [OPTION]
            -a:
            --acks: expect acks from server,
            -p <port>:
            --port <port>: connect to port 'port'.
            "
       , name)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut cfg = Config::default();
    check_args(&args[1..],
               &mut cfg)
               .expect(
                 &usage(
                   &args[0]));

    // take a nap between the messages
    let nap = time::Duration::new(1, 0);

    let mut ok = vec![0; 2]; // only with acks

    let addr = format!("127.0.0.1:{}", cfg.port);
    let mut stream = match TcpStream::connect(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        },
    };

    loop {
        println!("sleeping");
        thread::sleep(nap);
        let mut v = match build_message() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: {:?}", e);
                std::process::exit(1);
            },
        };

        v.push(3); // message separator

        println!("message: {:?}", v);

        match stream.write(&v) {
            Ok(s) => {
                stream.flush().expect("cannot flush: ");
                println!("{} bytes written", s);
                thread::sleep(time::Duration::new(0, 1000));
            },
            Err(e) => {
                eprintln!("error: {:?}", e);
                std::process::exit(1);
            },
        }

        // if we are not expecting acks continue with the next message
        if !cfg.acks {
            continue;
        }

        // otherwise wait for ack
        loop {
            match stream.read(&mut ok) {
                Ok(s) => {
                    println!("{} bytes read", s);
                    break;
                },
                Err(e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    eprintln!("error: {:?}", e);
                    std::process::exit(1);
                },
            }
        }
    }
}

// parse command line arguments
fn check_args(args: &[String], cfg: &mut Config) -> Result<(), String> {
    let parse = |args: &[String], cfg: &mut Config| -> Result<(), String>{
        let mut arg = None;
        for s in args {
            match arg {
                Some(0) => {
                   cfg.port = s.to_string();
                   arg = None;
                   continue;
                },
                Some(n) => return Err(format!("unknown option number {}", n)),
                _ => match s.as_ref() {
                       // acks
                       "-a" => cfg.acks = true,
                       "--a" => cfg.acks = true,
                       "-ack" => cfg.acks = true,
                       "--ack" => cfg.acks = true,

                       // port
                       "port" => arg = Some(0),
                       "-port" => arg = Some(0),
                       "--port" => arg = Some(0),
                       "--p" => arg = Some(0),
                       "-p" => arg = Some(0),

                       // error
                       _ => return Err(format!("unknown option '{}'", s)),
                }
            }
        }
        Ok(())
    };

    if args.len() > 0 {
        return parse(args, cfg);
    }
    Ok(())
}

fn build_message() -> io::Result<Vec<u8>> {
    let j = make_n_arbitrary(10);
    let mut v = Vec::new();
    let s = "Add\n\n".as_bytes().to_vec();
    v.extend_from_slice(&s[..]);
    let mut jv = Vec::new();
    j.serialize(&mut io::Cursor::new(&mut jv))?;
    v.extend_from_slice(&jv[..]);
    Ok(v)
}
