use std::io::prelude::*;
use std::io;
use std::io::ErrorKind;
use std::net::TcpStream;
use std::thread;
use std::time;
use std::env;
use jsosso::arbitrary::{make_n_arbitrary};

fn main() {

    let acks = check_args();

    let nap = time::Duration::new(1, 0);
    let mut stream = match TcpStream::connect("127.0.0.1:6049") {
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
        v.push(3);
        println!("message: {:?}", v);

        match stream.write(&v) {
            Ok(s) => {
                stream.flush().unwrap();
                println!("{} bytes written", s);
                thread::sleep(time::Duration::new(0, 1000));
            },
            Err(e) => {
                eprintln!("error: {:?}", e);
                std::process::exit(1);
            },
        }

        if !acks {
            continue;
        }

        let mut ok = vec![0; 2];
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

fn check_args() -> bool {
    let args: Vec<String> = env::args().collect();
    let parse = |s: &str| -> bool {
        match s {
            "with-ack" => return true,
            "-with-ack" => return true,
            "--with-ack" => return true,
            "-w" => return true,
            "-a" => return true,
            "-ack" => return true,
            "--ack" => return true,
            _ => return false,
        }
    };

    if args.len() > 1 {
        return parse(&args[1]);
    }
    false
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
