use std::io::prelude::*;
use std::io;
use std::net::TcpStream;
use std::thread;
use std::time;
use jsosso::arbitrary::{make_n_arbitrary};

fn main() {
    let nap = time::Duration::new(1, 0);
    let mut stream = match TcpStream::connect("127.0.0.1:6049") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {:?}", e);
            std::process::exit(1);
        },
    };

    // what we are doing here looks much more like UDP than TCP
    stream.set_nodelay(true).unwrap();

    loop {
        println!("sleeping");
        thread::sleep(nap);
        let v = match build_message() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: {:?}", e);
                std::process::exit(1);
            },
        };
        match stream.write(&v) {
            Ok(s) => {
                println!("{} bytes written", s);
            },
            Err(e) => {
                eprintln!("error: {:?}", e);
                std::process::exit(1);
            },
        }
    }
}

fn build_message() -> io::Result<Vec<u8>> {
    let j = make_n_arbitrary(10);
    let mut v = Vec::new();
    let s = "Add\n\n".as_bytes().to_vec();
    v.extend_from_slice(&s[..]);
    let mut jv = Vec::new();
    j.serialize(&mut io::Cursor::new(&mut jv))?;
    v.extend_from_slice(&jv[..]);
    println!("message: {:?}", v);
    Ok(v)
}
