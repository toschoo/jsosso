use super::*;
use std::str;
use rand;

/// Generates a random Json value.
///
/// Example:
///
/// ```
/// use std::io::Cursor;
/// use jsosso::{Json};
/// use jsosso::parsing::*;
/// use jsosso::arbitrary::*;
/// use pacosso::{Stream, Opts};
///
/// for _ in 0 .. 10 {
///     let original = make_arbitrary();
///     let mut v = Vec::new();
///
///     assert!(match original.serialize(&mut v) {
///         Ok(_)  => true,
///         Err(_) => false,
///     });
///
///     let mut input = Cursor::new(v);
///     let mut s = Stream::new(Opts::default()
///                .set_buf_size(1024)
///                .set_buf_num(5),
///                &mut input);
///
///     let mycopy = match parse(&mut s) {
///         Ok(j) => j,
///         Err(e) => panic!("unexpected error: {:?} at {}", e, s.position()),
///     };
///
///     assert_eq!(original, mycopy);
/// }
/// 
/// ```
pub fn make_arbitrary() -> Json {
    make_value(0, 0, 0)
}

/// Generates a random Json value with at most 'n' elements.
/// For n = 0, the function behaves exactly like `make_arbitrary()`.
pub fn make_n_arbitrary(n: usize) -> Json {
    make_value(0, n, 0)
}

fn make_value(level: usize, max: usize, have: usize) -> Json {

    let x = rand::random::<u8>()%100;

    if x < 5 {
        return Json::Null;
    }

    if x < 15 {
        return make_boolean(); 
    }

    if x < 30 {
        return make_number();
    }

    if x < 50 {
       return make_string();
    }

    if x < 75 {
       return make_array(level, max, have);
    }

    make_object(level, max, have)
}

fn make_boolean() -> Json {
    let x = rand::random::<u8>()%2;
    Json::Boolean(x == 0)
}

fn make_number() -> Json {
    Json::Number(rand::random::<f64>())
}

fn make_string() -> Json {
    Json::String(random_string())
}

fn random_string() -> String {
    let x = (rand::random::<u8>()%25)+2;

    let mut v = Vec::new();
    for _ in 0 .. x {
        let b = rand::random::<u8>()%42;
        let c = b+48;
        if c == b'\\' {
           v.push(b'.');
        } else {
           v.push(c);
        }
    }

    let s = match str::from_utf8(&v) {
        Ok(x) => x,
        Err(e) => panic!("internal error: {:?}", e), // cannot happen
    };

    s.to_string()
}

fn make_array(level: usize, max: usize, have: usize) -> Json {
    if level > 2 {
        return Json::Null;
    }

    let mut n = have;

    let x = rand::random::<usize>()%100;

    let mut v = Vec::with_capacity(x);
    for _ in 0 .. x {
        if n >= max {
            break;
        }
        let j = make_value(level+1, max, n);
        v.push(j); 
        n += 1;
    }
    
    Json::Array(v)
}

fn make_object(level: usize, max: usize, have: usize) -> Json {
    if level > 2 {
        return Json::Null;
    }

    let mut n = have;

    let x = rand::random::<u8>()%25;

    let mut m = HashMap::new();
    for _ in 0 .. x {
        if n >= max {
            break;
        }
        let s = random_string();
        let j = make_value(level+1, max, n);
        let _ = m.insert(s, j);
        n += 1;
    }

    Json::Object(Box::new(m))
}


