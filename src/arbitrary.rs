use super::*;
use std::str;
use rand;

pub fn make_arbitrary() -> Json {
    make_value(0)
}

fn make_value(level: usize) -> Json {

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
       return make_array(level);
    }

    make_object(level)
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

fn make_array(level: usize) -> Json {
    if level > 2 {
        return Json::Null;
    }

    let x = rand::random::<usize>()%100;

    let mut v = Vec::with_capacity(x);
    for _ in 0 .. x {
        let j = make_value(level+1);
        v.push(j); 
    }
    
    Json::Array(v)
}

fn make_object(level: usize) -> Json {
    if level > 2 {
        return Json::Null;
    }

    let x = rand::random::<u8>()%25;

    let mut m = HashMap::new();
    for _ in 0 .. x {
        let s = random_string();
        let j = make_value(level+1);
        let _ = m.insert(s, j);
    }

    Json::Object(Box::new(m))
}


