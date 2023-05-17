use std::fs::File;
use std::time::Instant;
use std::ffi::OsString;
use std::io::{self, Cursor};
use pacosso::{Stream, Opts};
use jsosso::{parse};

const US: f64 = 1_000_000.0;

fn main() {
    let _ = parse_and_serialize("rsc/test/oeis.json".into());
    let mut d = 0;
    for _ in 0 .. 1000 {
        d += run_with_file("rsc/test/oeis.json".into()) as i64;
    }
    d /= 1000;
    let k = 2363.0 / d as f64;
    println!("Duration oeis : {:03}us = {:03}MB/s", d, k as i64);

    d = 0;
    for _ in 0 .. 1000 {
        d += run_with_file("rsc/test/pass1.json".into()) as i64;
    }
    d /= 1000;
    let k = 1442.0 / d as f64;
    println!("Duration pass1: {:03}us = {:03}MB/s", d, k as i64);

    d = 0;
    let mut v: Vec<u8> = r#"
        "The paper by Kaoru Motose starts as follows:
        \"Let q be a prime divisor of a Mersenne number 2^p-1 where p is prime. Then p is the order of 2 (mod q).
        Thus p is a divisor of q - 1 and q > p. This shows that there exist infinitely many prime numbers.\"
        - Pieter Moree, Oct 14 2004""#
        .to_string().bytes().collect();
    for _ in 0 .. 1000 {
        d += run_with_text(&mut v) as i64;
    }
    d /= 1000;
    let k = v.len() as f64 / d as f64;
    println!("Duration text : {:03}us = {:03}MB/s", d, k as i64);

    d = 0;
    let mut v: Vec<u8> = r#"
        [2,3,5,7,11,13,17,19,23, 29, 31, 37, 41, 43, 47, 53, 59,
	 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113,
	 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181,
	 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251,
	 257, 263, 269, 271]
        "#
        .to_string().bytes().collect();
    for _ in 0 .. 1000 {
        d += run_with_text(&mut v) as i64;
    }
    d /= 1000;
    let k = v.len() as f64 / d as f64;
    println!("Duration nums : {:03}us = {:03}MB/s", d, k as i64);
}

fn parse_and_serialize(f: OsString) -> io::Result<usize> {
    let mut input = match File::open(f) {
        Ok(f) => f,
        Err(e) => panic!("can't read file: {:?}", e),
    };

    let mut s = Stream::new(Opts::default()
                   .set_buf_size(8)
                   .set_buf_num(3),
                   &mut input);

    match parse(&mut s) {
        Ok(j) => return j.serialize(&mut io::stdout()),
        Err(e) => panic!("unexpected error: {:?}", e),
    }
}

fn run_with_file(f: OsString) -> f64 {
    let mut input = match File::open(f) {
        Ok(f) => f,
        Err(e) => panic!("can't read file: {:?}", e),
    };

    let t = Instant::now();
    let mut s = Stream::new(Opts::default()
                   .set_buf_size(8)
                   .set_buf_num(3),
                   &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });

    t.elapsed().as_secs_f64() * US
}

fn run_with_text(txt: &mut Vec<u8>) -> f64 {

    let t = Instant::now();
    let mut input = Cursor::new(txt);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });

    t.elapsed().as_secs_f64() * US
}
