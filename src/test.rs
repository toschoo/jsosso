use super::*;
use std::io::Cursor;
use std::fs::File;
use pacosso::Opts;

fn approx_eq(a: f64, b: f64) -> bool {
    println!("{} >= {} && {} <= {}", a, b - 0.1, a, b + 0.1);
    a >= b - 0.1 && a <= b + 0.1
}

#[test]
fn test_hello_world() {
    let v: Vec<u8> = "\"hello world\"".to_string().bytes().collect();
    let mut input = Cursor::new(v);
    println!("{}", Opts::default().buf_size);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(Json::String(x)) if x == "hello world" => true,
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_escape() {
    let v: Vec<u8> = r#""linebreak: '\n', another: '\r\n', tab: '\t'""#.to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(Json::String(s)) => {
            println!("{}", s);
            println!("linebreak: '\n', another: '\r\n', tab: '\t'");
            let r = if s == "linebreak: '\n', another: '\r\n', tab: '\t'" {
                true
            } else {
                false 
            };
            r
        },
        Ok(_) => panic!("unexpected value"),
        Err(e) => panic!("error: {:?}", e),
    });
}

#[test]
fn test_unicode() {
    let v: Vec<u8> = r#""\uD834\uDD1E \u006d\u0075\u0073\udd1eic""#.to_string().bytes().collect(); // , 0xDD1E, 0x0069, 0x0063, 0xD834].
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(Json::String(s)) => {
            println!("{}", s);
            let r = if s == "𝄞 mus�ic" {
                true
            } else {
                false 
            };
            r
        },
        Ok(_) => panic!("unexpected value"),
        Err(e) => panic!("error: {:?}", e),
    });
}

#[test]
fn test_jobject() {
    let v: Vec<u8> = r#"{
       "greetings": "hello world",
       "name": "world",
       "first name": "hello"
    }"#.to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    let o = match parse(&mut s) {
        Ok(Json::Object(v)) => v,
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    assert!(match o.get("greetings") {
      Some(Json::String(s)) if s == "hello world" => true,
      Some(v) => panic!("unexpected value {:?}", v),
      None => panic!("greetings not foudn"),
    });

    assert!(match o.get("name") {
      Some(Json::String(s)) if s == "world" => true,
      Some(v) => panic!("unexpected value {:?}", v),
      None => panic!("greetings not foudn"),
    });

    assert!(match o.get("first name") {
      Some(Json::String(s)) if s == "hello" => true,
      Some(v) => panic!("unexpected value {:?}", v),
      None => panic!("greetings not foudn"),
    });
}

#[test]
fn test_jarray() {
    let v: Vec<u8> = r#"[
       "hello", "world", null,
       "hello", false, "ilja", true]
    }"#.to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    let o = match parse(&mut s) {
        Ok(Json::Array(v)) => v,
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    assert!(match o[..] {
        [Json::String(ref a),
         Json::String(ref b),
         Json::Null         ,
         Json::String(ref c),
         Json::Boolean(false),
         Json::String(ref d), 
         Json::Boolean(true)] if a == "hello" &&
                                 b == "world" &&
                                 c == "hello" &&
                                 d == "ilja"  => true,
        _ => panic!("unexpected value {:?}", o),
    });
}

#[test]
fn test_jnumber() {
    let v: Vec<u8> = r#"[
       1, 1.5, 0.2, 0.1e3, 1.0E5, 500.123, -9.0, 3.1415926]
    }"#.to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    let o = match parse(&mut s) {
        Ok(Json::Array(v)) => v,
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) => panic!("unexpected error: {:?}", e),
    };

    assert!(match o[..] {
        [Json::Number(a),
         Json::Number(b),
         Json::Number(c),
         Json::Number(d),
         Json::Number(e),
         Json::Number(f),
         Json::Number(g),
         Json::Number(h)] if approx_eq(a, 1.0)     &&
                             approx_eq(b, 1.5)     &&
                             approx_eq(c, 0.2)     &&
                             approx_eq(d, 0.1e3)   &&
                             approx_eq(e, 1.0e5)   &&
                             approx_eq(f, 500.123) &&
                             approx_eq(g, -9.0)    &&
                             approx_eq(h, 3.1415925) => true,
        _ => panic!("unexpected value {:?}", o),
    });
}

#[test]
fn test_fail_unclosed_array() {
    let v: Vec<u8> = r#"["unclosed array""#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_eof() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_empty() {
    let v: Vec<u8> = r#""#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_eof() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_unquoted_key() {
    let v: Vec<u8> = r#"
        {unqoted_key: "keys must be quoted"}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_extra_comma_in_array() {
    let v: Vec<u8> = r#"
        ["extra comma",]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_extra_double_comma_in_array() {
    let v: Vec<u8> = r#"
        ["double comma",,]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_missing_value() {
    let v: Vec<u8> = r#"
        [, "<-- missing value"]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

// this one passes, because we have a well-formed jvalue before the error
#[test]
fn test_comma_outside_array() {
    let v: Vec<u8> = r#"
        ["comma after array"],"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

// same thing
#[test]
fn test_double_closed_array() {
    let v: Vec<u8> = r#"
        ["comma after array"]]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_extra_comma_in_object() {
    let v: Vec<u8> = r#"
        {"extra": "comma",}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_illegal_expression() {
    let v: Vec<u8> = r#"
        {"illegal expression": 1+2}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_leading_zero() {
    let v: Vec<u8> = r#"
        {"leading zero": 012}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_illegal_hex() {
    let v: Vec<u8> = r#"
        {"illegal hex": 0x12}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_illegal_escape() {
    let v: Vec<u8> = r#"
        {"illegal escape": \x12}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_unqoted_string() {
    let v: Vec<u8> = r#"
        [\naked]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

// deep nesting - should we protect?
#[test]
fn test_pass_too_deep() {
    let v: Vec<u8> = r#"
        [[[[[[[[[[[[[[[[[[[["Too deep"]]]]]]]]]]]]]]]]]]]]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_missing_colon() {
    let v: Vec<u8> = r#"
        {"missing" "colon"}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_double_colon() {
    let v: Vec<u8> = r#"
        {"double":: "colon"}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_comma_instead_colon() {
    let v: Vec<u8> = r#"
        {"comma", "colon"}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_colon_instead_comma() {
    let v: Vec<u8> = r#"
        ["colon": "comma"]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_mismatched_brackets() {
    let v: Vec<u8> = r#"
        {"colon": "comma"]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_bad_boolean() {
    let v: Vec<u8> = r#"
        {"bool": truth}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_single_quote() {
    let v: Vec<u8> = r#"
        ['single quotes']"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_empty_exponent() {
    let v: Vec<u8> = r#"
        [0e+]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_fail_plus_minus_exponent() {
    let v: Vec<u8> = r#"
        [0e+-1]"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(v) => panic!("unexpected value: {:?}", v),
        Err(e) if e.is_expected_token() => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_pass_empty_string() {
    let v: Vec<u8> = r#"
        """#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_pass_empty_object() {
    let v: Vec<u8> = r#"
        {}"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_pass_empty_array() {
    let v: Vec<u8> = r#"
        []"#
        .to_string().bytes().collect();
    let mut input = Cursor::new(v);
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?}", e),
    });
}

#[test]
fn test_pass_from_file_pattern1() {
    let mut input = match File::open("rsc/test/pass1.json") {
        Ok(f) => f,
        Err(e) => panic!("can't read file: {:?}", e),
    };
    let mut s = Stream::new(Opts::default()
               .set_buf_size(8)
               .set_buf_num(3),
               &mut input);

    assert!(match parse(&mut s) {
        Ok(_) => true,
        Err(e) => panic!("unexpected error: {:?} at {}", e, s.position()),
    });
}
