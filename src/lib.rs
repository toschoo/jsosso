use std::io::{self, Read, Write};
use std::collections::HashMap;
use std::str;
use pacosso::{Stream, ParseResult, ParseError};

#[derive(Debug)]
pub enum Json {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    Array(Vec<Json>),
    Object(Box<HashMap<String, Json>>),
}

#[derive(Debug)]
struct Human {
    size: usize,
    line: u64,
    ind: String,
}

impl Json {
    pub fn serialize<W: Write> (&self, w: &mut W) -> io::Result<usize> {
        let mut h = Human {
            size: 0,
            line: 0,
            ind: "".to_string(),
        };
        let s = self.write_jvalue(w, &mut h)?;
        w.flush()?;
        Ok(h.size)
    }

    fn write_jvalue<W: Write> (&self, w: &mut W, h: &mut Human) -> io::Result<()> {
        match self {
            Json::Null => return self.write_jnull(w, h),
            Json::Boolean(t) => return self.write_jboolean(*t, w, h),
            Json::Number(n) => return self.write_jnumber(*n, w, h),
            Json::String(s) => return self.write_jstring(s, w, h),
            Json::Array(a) => return self.write_jarray(a, w, h),
            Json::Object(o) => return self.write_jobject(o, w, h),
        }
    }

    fn write_indent<W: Write>(&self, w: &mut W, h: &mut Human) -> io::Result<()> {
        if h.ind.len() > 0 {
            let v = h.ind.bytes().collect::<Vec<u8>>();
            self.write(w, h, &v)?;
        }
        Ok(()) 
    }

    fn write<W: Write>(&self, w: &mut W, h: &mut Human, b: &[u8]) -> io::Result<()> {
        let mut s = 0;
        while s < b.len() {
            s += w.write(&b[s..])?;
        }
        h.size += s;
        h.line += s as u64;
        Ok(())
    }

    fn write_jnull<W: Write> (&self, w: &mut W, h: &mut Human) -> io::Result<()> {
        self.write(w, h, b"null")
    }

    fn write_jboolean<W: Write> (&self, t: bool, w: &mut W, h: &mut Human) -> io::Result<()> {
        if t {
            return self.write(w, h, b"true");
        } else {
            return self.write(w, h, b"false");
        }
    }

    fn write_jnumber<W: Write> (&self, n: f64, w: &mut W, h: &mut Human) -> io::Result<()> {
        let t = format!("{}", n).bytes().collect::<Vec<u8>>();
        self.write(w, h, &t)
    }

    fn write_jstring<W: Write> (&self, s: &str, w: &mut W, h: &mut Human) -> io::Result<()> {
        self.write(w, h, b"\"")?;
        let mut v = Vec::new();
        for b in s.bytes().collect::<Vec<u8>>() {
            match b {
                b'"' => {
                    v.push(b'\\'); v.push(b);
                },
                b'\\' => {
                    v.push(b'\\'); v.push(b);
                },
                b'/' => {
                    v.push(b'\\'); v.push(b);
                },
                b'\n' => {
                    v.push(b'\\'); v.push(b'n');
                },
                b'\r' => {
                    v.push(b'\\'); v.push(b'r');
                },
                b'\t' => {
                    v.push(b'\\'); v.push(b't');
                },
                8 => { // backspace
                    v.push(b'\\'); v.push(b'b');
                },
                12 => { // formfeed
                    v.push(b'\\'); v.push(b'f');
                },
                _ => v.push(b),
            }
        }
        self.write(w, h, &v)?;
        self.write(w, h, b"\"")
    }

    fn write_jarray<W: Write> (&self, a: &Vec<Json>, w: &mut W, h: &mut Human) -> io::Result<()> {
        let mut first = true;
        self.write(w, h, b"[\n")?;
        h.ind += "  ";
        h.line = 0;
        for o in a {
           if first {
               first = false;
               self.write_indent(w, h)?;
           } else {
               if h.line >= 120 {
                   self.write(w, h, b",\n")?;
                   h.line = 0;
                   self.write_indent(w, h)?;
               } else {
                   self.write(w, h, b", ")?;
               }
           }
           o.write_jvalue(w, h)?;
        }
        h.ind.pop();
        h.ind.pop();
        self.write(w, h, b"\n]\n")
    }

    fn write_jobject<W: Write> (&self, o: &HashMap<String, Json>, w: &mut W, h: &mut Human) -> io::Result<()> {
        let mut first = true;
        self.write(w, h, b"{\n")?;
        h.ind += "  ";
        for (k, v) in o {
           if first {
               first = false;
           } else {
               self.write(w, h, b",\n")?;
           }
           self.write_indent(w, h)?;
           self.write_jstring(k, w, h)?;
           self.write(w, h, b": ")?;
           v.write_jvalue(w, h)?;
        }
        h.ind.pop();
        h.ind.pop();
        self.write(w, h, b"\n}\n")
    }
}

pub fn parse<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    s.skip_whitespace()?;
    jvalue(s)
}

pub fn jvalue<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {
    /*
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

fn fail<R: Read>(s: &mut Stream<R>, msg: String) -> ParseResult<Json> {
    s.fail(&msg, Json::Null)
}

fn jarray<R: Read>(s: &mut Stream<R>) -> ParseResult<Json> {

    s.byte(b'[')?;
    s.skip_whitespace()?;

    let mut v = Vec::new();

    let c = s.peek_byte()?;
    if c == b']' {
        return Ok(Json::Array(v));
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
            if !e.is_expected_token() {
                return Err(e);
            }
        },
    }

    match s.one_of_bytes(&[b'e', b'E']) {
        Ok(()) => exp = true,
        Err(e) => {
            if !e.is_expected_token() {
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
                        "duplicated key '{}' in object", k.clone()))),
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

    let mut v: Vec<u8> = Vec::new();
    s.byte('"' as u8)?;
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
      Err(_) => return Err(ParseError::Failed("unicode error".to_string())),
    }
}

fn convert_ascii(n: u8) -> ParseResult<u16> {
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
        _ => return Err(ParseError::Failed(format!("hexadecimal expected, have: {}", n))),
    }
}

fn utf16bytes<R: Read>(s: &mut Stream<R>) -> ParseResult<u16> {
    let bs = s.get_bytes(4)?;
    let mut u = 0u16;
    let mut x = 3u32;
    let h = 16u16;
    for b in bs {
        let i = convert_ascii(b.to_ascii_lowercase())?;
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
            _ => return Err(ParseError::Failed("unexpected result".to_string())),
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
                       "unknown escape sequence {}", c))),
       }
       Ok(())
}

#[cfg(test)]
mod test;
