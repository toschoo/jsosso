use std::fs::File;
use std::time::Instant;
use std::ffi::OsString;
use std::io::{self, Cursor};
use pacosso::{Stream, Opts};
use jsosso::parsing::{parse};
use jsosso::arbitrary::make_n_arbitrary;

const US: f64 = 1_000_000.0;

fn main() {
    let mut d: f64 = 0.0;
    let mut sz = 0;


    let mut input = io::Cursor::new("hello world".as_bytes().to_vec());

    for _ in 0 .. 1000 {
        let t = Instant::now();
        Stream::new(Opts::default().set_buf_size(8), &mut input).succeed().unwrap();
        d += t.elapsed().as_secs_f64() * US;
    }
    d /= 1000.0;
    println!("Duration stream (buf size:    8): {:05}us", d as i64);

    d = 0.0;
    for _ in 0 .. 1000 {
        let t = Instant::now();
        Stream::new(Opts::default().set_buf_size(128), &mut input).succeed().unwrap();
        d += t.elapsed().as_secs_f64() * US;
    }
    d /= 1000.0;
    println!("Duration stream (buf size:  128): {:05}us", d as i64);

    d = 0.0;
    for _ in 0 .. 1000 {
        let t = Instant::now();
        Stream::new(Opts::default().set_buf_size(1024), &mut input).succeed().unwrap();
        d += t.elapsed().as_secs_f64() * US;
    }
    d /= 1000.0;
    println!("Duration stream (buf size: 1024): {:05}us", d as i64);

    d = 0.0;
    for _ in 0 .. 1000 {
        let t = Instant::now();
        Stream::new(Opts::default(), &mut input).succeed().unwrap();
        d += t.elapsed().as_secs_f64() * US;
    }
    d /= 1000.0;
    println!("Duration stream (buf size: 8192): {:05}us", d as i64);

    println!("");
    // oeis
    d = 0.0;
    for _ in 0 .. 1000 {
        let (x, l) = run_with_file("rsc/test/oeis.json".into());
        d += x;
        sz = l;
    }
    d /= 1000.0;
    let k = sz / d as u64;
    println!("Duration oeis  (size: {:06}): {:05}us = {:05}MB/s", sz, d as i64, k as i64);

    // pass1
    d = 0.0;
    for _ in 0 .. 1000 {
        let (x, l) = run_with_file("rsc/test/pass1.json".into());
        d += x;
        sz = l;
    }
    d /= 1000.0;
    let k = sz / d as u64;
    println!("Duration pass1 (size: {:06}): {:05}us = {:05}MB/s", sz, d as i64, k as i64);

    // strings
    d = 0.0;
    let mut v: Vec<u8> = r#"
        "The paper by Kaoru Motose starts as follows:
        \"Let q be a prime divisor of a Mersenne number 2^p-1 where p is prime. Then p is the order of 2 (mod q).
        Thus p is a divisor of q - 1 and q > p. This shows that there exist infinitely many prime numbers.\"
        - Pieter Moree, Oct 14 2004""#
        .to_string().bytes().collect();
    for _ in 0 .. 1000 {
        d += run_with_text(&mut v);
    }
    d /= 1000.0;
    sz = v.len() as u64;
    let k = sz as f64 / d;
    println!("Duration text  (size: {:06}): {:05}us = {:05}MB/s", sz, d as i64, k as i64);

    // numbers
    d = 0.0;
    let mut v: Vec<u8> = r#"
        [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59,
	 61, 67, 71, 73, 79, 83, 89, 97, 101, 103, 107, 109, 113,
	 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179, 181,
	 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251,
	 257, 263, 269, 271]
        "#
        .to_string().bytes().collect();
    for _ in 0 .. 1000 {
        d += run_with_text(&mut v);
    }
    d /= 1000.0;
    sz = v.len() as u64;
    let k = sz as f64 / d;
    println!("Duration nums  (size: {:06}): {:05}us = {:05}MB/s", sz, d as i64, k as i64);

    // arbitrary
    for _ in 0 .. 100 {
        let (x, l) = parse_and_serialize("rsc/test/arbitrary.json".into());
        d += x;
        sz = l;
    }
    d /= 100.0;
    let k = sz as f64 / d;
    println!("Duration arb.  (size: {:06}): {:05}us = {:05}MB/s", sz, d as i64, k as i64);

    // 10 random
    let (x, l) = run_with_random(100);
    d = x;
    let k = l as f64 / d;
    println!("Duration rand. (size: {:06}): {:05}us = {:05}MB/s", l, d as i64, k as i64);
}

fn run_with_file(f: OsString) -> (f64, u64) {
    let mut input = match File::open(f) {
        Ok(f) => f,
        Err(e) => panic!("can't read file: {:?}", e),
    };

    let l = match input.metadata() {
        Ok(m) => m.len(),
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    let t = Instant::now();
    let mut s = Stream::new(Opts::default()
                   .set_buf_size(1024)
                   .set_buf_num(3),
                   &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });

    (t.elapsed().as_secs_f64() * US, l)
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

fn parse_and_serialize(f: OsString) -> (f64, u64) {
    let n = f.clone();
    let mut input = match File::open(f) {
        Ok(f) => f,
        Err(e) => panic!("can't read file: {:?}", e),
    };

    let l = match input.metadata() {
        Ok(m) => m.len(),
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    let t = Instant::now();
    let mut s = Stream::new(Opts::default()
                   .set_buf_size(1024)
                   .set_buf_num(3),
                   &mut input);

    let mut v: Vec<u8> = Vec::new();
    let mut output = Cursor::new(&mut v);
    let j = match parse(&mut s) {
        Ok(j) => {
            j.serialize(&mut output).unwrap();
            j
        },
        Err(e) => panic!("unexpected error: {:?} in {:?} at {}", e, n, s.position()),
    };

    let mut input2 = Cursor::new(&mut v);
    let mut k = Stream::new(Opts::default()
                   .set_buf_size(1024)
                   .set_buf_num(3),
                   &mut input2);

    match parse(&mut k) {
        Ok(r) => assert_eq!(j, r),
        Err(e) => panic!("unexpected error: {:?} in {:?} at {}", e, n, s.position()),
    };

    (t.elapsed().as_secs_f64() * US, 2*l)
}

fn run_with_random(n: usize) -> (f64, usize) {
    let v = match n_random(n) {
        Ok(v) => v,
        Err(e) => panic!("cannot generate {} random: {:?}", n, e),
    };
    let l = v.len();
    let t = Instant::now();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default(), &mut input);
    let mut js = Vec::new();
    let mut rs = Vec::new();
    loop {
        match parse(&mut s) {
            Ok(j) => {
                let mut v2 = Vec::new();
                let mut output = Cursor::new(&mut v2);
                j.serialize(&mut output).unwrap();
                js.push(j);
                let mut input2 = Cursor::new(v2);
                let mut s2 = Stream::new(Opts::default(), &mut input2);
                match parse(&mut s2) {
                    Ok(r) => rs.push(r),
                    Err(e) => panic!("parse failed with error: {:?}", e),
                }
            },
            Err(e) if e.is_eof() => break,
            Err(e) => panic!("parse of serialized failed with error: {:?}", e),
        }
    }
    for (j, r) in js.into_iter().zip(rs) {
        // println!("comparing {:?} and {:?}", j, r);
        assert_eq!(j, r);
    }
    
    (t.elapsed().as_secs_f64() * US, 2*l)
}

fn n_random(m: usize) -> io::Result<Vec<u8>> {
    let mut v = Vec::new();
    for _ in 0 .. m {
        let j = make_n_arbitrary(10);
        // println!("{:?}", j);
        let _ = j.serialize(&mut v)?;
        v.push(b' ');
    }
    Ok(v)
}
