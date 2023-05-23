use std::io::Read;
use std::collections::HashMap;
use std::str;
use super::*;

/// Parses the first complete Json value in stream 's'
/// and returns it as Enum 'Json' on success and a parse error otherwise.
/// For details on how to create and manage streams, please refer to [`pacosso`].
///
/// [`pacosso`]: https://github.com/toschoo/pacosso
///
/// Example:
///
/// ```
///    use std::io::Cursor;
///    use jsosso::{Json};
///    use jsosso::parsing::*;
///    use pacosso::{Stream, ParseError, Opts};
/// 
///    let v: Vec<u8> = "\"hello world\"".to_string().bytes().collect();
///    let mut input = Cursor::new(v);
///    let mut s = Stream::new(Opts::default()
///               .set_buf_size(8)
///               .set_buf_num(3),
///               &mut input
///    );
///
///    assert!(match parse(&mut s) {
///        Ok(Json::String(x)) if x == "hello world" => true,
///        Ok(v) => panic!("unexpected value: {:?}", v),
///        Err(e) => panic!("unexpected error: {:?}", e),
///    });
/// ```
pub fn parse<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    s.skip_whitespace()?;
    jvalue(s)
}

fn jvalue<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    /* This looks nice, but it is less efficient and would force us
       to use a buffer that basically is big enough to contain the whole jvalue.
 
    let v = [jstring, jobject, jarray, jnil, jboolean, jnumber]; 
    s.choice(&v)
    */

    let ch = s.peek_byte()?;
    match ch {
      b'"' => jstring(s),
      b'{' => jobject(s),
      b'[' => jarray(s),
      b'n' => jnil(s),
      b't' => jboolean(s),
      b'f' => jboolean(s),
      _    => jnumber(s),
    }
}

// A simple error generator for ParseResult<Json>
fn fail<R: Read>(s: &mut Stream<R>, msg: String) -> ParseResult<Json> {
    s.fail(&msg, Json::Null)
}

fn jnil<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    s.string("null")?;
    Ok(Json::Null)
}

fn jboolean<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {

    let c = s.any_byte()?;

    if c == b'f' {
       s.string("alse")?;
       return Ok(Json::Boolean(false));
    } else if c == b't' {
       s.string("rue")?;
       return Ok(Json::Boolean(true));
    }
    fail(s, "boolean value expected".to_string())
}

// jnumber is extremely inefficient. Definitely needs review.
fn jnumber<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    let mut zero = false;
    let mut exp  = false;
    let mut v = Vec::new();

    match s.byte(b'-') {
        Ok(()) => v.push(b'-'),
        Err(e) => {
            if !e.is_expected_token() {
                return Err(e);
            }
        },
    }

    match s.byte(b'0') {
        Ok(()) => {
            zero = true;
            v.push(b'0');
        },
        Err(e) => {
            if !e.is_expected_token() {
                return Err(e);
            }
        },
    }

    if !zero {
        let ds = s.digits()?;
        for d in ds {
            v.push(d);
        }
    }

    match s.byte(b'.') {
        Ok(()) => jfrac(s, &mut v)?,
        Err(e) => {
            if !e.is_expected_token() && !e.is_eof() {
                return Err(e);
            }
        },
    }

    match s.one_of_bytes(&[b'e', b'E']) {
        Ok(()) => exp = true,
        Err(e) => {
            if !e.is_expected_token() && !e.is_eof() {
                return Err(e);
            }
        },
    }

    if exp {
        jexp(s, &mut v)?;
    }

    let x = match str::from_utf8(&v) {
        Ok(x) => x,
        Err(e) => return fail(s, format!("internal error: {:?}", e)),
    };

    match x.parse::<f64>() {
        Ok(f) => return Ok(Json::Number(f)),
        Err(e) => return fail(s, format!("internal error: {:?}", e)),
    }
}

fn jfrac<R: Read>(s: &mut Stream<R>, v: &mut Vec<u8>) -> ParseResult<()> {
     v.push(b'.');
     let ds = s.digits()?; 
     for d in ds {
         v.push(d);
     }
     Ok(())
}

fn jexp<R: Read>(s: &mut Stream<R>, v: &mut Vec<u8>) -> ParseResult<()> {
     v.push(b'e');
     let c = s.peek_byte()?;
     if c == b'-' {
         s.byte(b'-')?;
         v.push(b'-');
     } else if c == b'+' {
         s.byte(b'+')?;
     }
     let ds = s.digits()?; 
     for d in ds {
         v.push(d);
     }
     Ok(())
}

fn jarray<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {

    s.byte(b'[')?;
    s.skip_whitespace()?;

    let mut v = Vec::new();

    // empty array
    match s.byte(b']') {
        Ok(()) =>  return Ok(Json::Array(v)),
        Err(e) =>  if !e.is_expected_token() {
                       return Err(e);
        },
    }

    loop {
        s.skip_whitespace()?;
        let e = jvalue(s)?;
        v.push(e);   
        s.skip_whitespace()?;
        match s.byte(b',') {
          Ok(()) => continue,
          _ => break,
        }
    }

    s.byte(b']')?;

    Ok(Json::Array(v))
}

fn jobject<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {

    s.byte(b'{')?;

    let m = keyvalues(s)?;

    s.byte(b'}')?;

    Ok(Json::Object(Box::new(m)))
}

fn keyvalues<R: Read>(s: &mut Stream<R>) -> ParseResult<HashMap<String, Json>> {
    let mut m: HashMap<String, Json> = HashMap::new();

    s.skip_whitespace()?;

    // may be empty
    let c = s.peek_byte()?;
    if c == b'}' {
        return Ok(m);
    }
    
    loop {
        let (k, v) = keyvalue(s)?;
        let _ = match m.insert(k.clone(), v) {
            Some(_) => return Err(ParseError::Failed(format!(
                          "duplicated key '{}' in object", k.clone()),
                          s.position())
                       ),
            _ => true,
        };
        s.skip_whitespace()?;
        match s.byte(b',') {
            Ok(()) => continue,
            Err(e) if e.is_expected_token() => return Ok(m),
            Err(e) => return Err(e),
        }
    }
}

fn keyvalue<R: Read>(s: &mut Stream<R>) -> ParseResult<(String, Json)> {
    s.skip_whitespace()?;
    let k = plain_string(s)?;
    s.skip_whitespace()?;
    s.byte(b':')?;
    s.skip_whitespace()?;
    let v = jvalue(s)?;

    Ok((k, v))
}

fn jstring<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    let x = plain_string(s)?;
    Ok(Json::String(x))
}

fn plain_string<R: Read>(s: &mut Stream<R>) -> ParseResult<String> {

    s.byte('"' as u8)?;
    let mut v: Vec<u8> = Vec::new();
    loop {
       let c = s.any_byte()?;
       if c == b'\\' {
           escape(s, &mut v)?;
           continue;
       }
       if c == b'"' {
           break;
       }
       v.push(c);
    }

    match str::from_utf8(&v) {
      Ok(x) => return Ok(x.to_string()),
      Err(_) => return Err(ParseError::Failed("unicode error".to_string(), s.position())),
    }
}

fn convert_ascii<R: Read>(s: &mut Stream<R>, n: u8) -> ParseResult<u16> {
    match n {
        b'0' => return Ok(0),
        b'1' => return Ok(1),
        b'2' => return Ok(2),
        b'3' => return Ok(3),
        b'4' => return Ok(4),
        b'5' => return Ok(5),
        b'6' => return Ok(6),
        b'7' => return Ok(7),
        b'8' => return Ok(8),
        b'9' => return Ok(9),
        b'a' => return Ok(10),
        b'b' => return Ok(11),
        b'c' => return Ok(12),
        b'd' => return Ok(13),
        b'e' => return Ok(14),
        b'f' => return Ok(15),
        _ => return Err(ParseError::Failed(format!("hexadecimal expected, have: {}", n), s.position())),
    }
}

fn utf16bytes<R: Read>(s: &mut Stream<R>) -> ParseResult<u16> {
    let bs = s.get_bytes(4)?;
    let mut u = 0u16;
    let mut x = 3u32;
    let h = 16u16;
    for b in bs {
        let i = convert_ascii(s, b.to_ascii_lowercase())?;
        u += i * h.pow(x);
        if x > 0 {
            x -= 1;
        }
    }
    Ok(u as u16)
}

fn push_replacement(v: &mut Vec<u8>) {
     let mut buf = [0;3];
     let xs = char::REPLACEMENT_CHARACTER.encode_utf8(&mut buf);
     for x in xs.bytes() {
         v.push(x);
     }
}

fn codepoint<R: Read>(s: &mut Stream<R>, v: &mut Vec<u8>) -> ParseResult<()> {
    let mut w = Vec::with_capacity(2);

    for i in 0 .. 2 {
        let a = utf16bytes(s)?;
        w.push(a);
        let cs: Vec<Result<char, u16>> = char::decode_utf16(w.clone())
                .map(|r| r.map_err(|e| e.unpaired_surrogate()))
                .collect();
        match cs[..] {
            [Ok(c)] => {
               let mut buf = [0; 4];
               let xs = c.encode_utf8(&mut buf);
               for x in xs.bytes() {
                  v.push(x);
               }
               return Ok(());
            },
            [Err(_)] if i == 0 => {
                match s.bytes(&vec![b'\\', b'u']) {
                    Ok(()) => {
                        continue;
                    },
                    Err(e) => {
                         if e.is_expected_token() || e.is_eof() {
                             push_replacement(v);
                             return Ok(());
                         } else {
                             return Err(e);
                         }
                    },
                }
            },
            [Err(_)] => {
                push_replacement(v);
                return Ok(());
            },
            _ => return Err(ParseError::Failed("unexpected result".to_string(),
                                               s.position())
                 ),
        }
    }

    Ok(())
}

fn escape<R: Read>(s: &mut Stream<R>, v: &mut Vec<u8>) -> ParseResult<()> {
       let c = s.any_byte()?;
       match c {
          b'\\' => v.push(b'\\'),
          b'"'  => v.push(b'"'),
          b'/'  => v.push(b'/'),
          b'b'  => v.push(8),  // backspace
          b'f'  => v.push(12), // formfeed
          b'n'  => v.push(b'\n'),
          b'r'  => v.push(b'\r'),
          b't'  => v.push(b'\t'),
          b'u'  => codepoint(s, v)?,
          _     => return Err(ParseError::Failed(format!(
                       "unknown escape sequence {}", c),
                        s.position()
                   )),
       }
       Ok(())
}
