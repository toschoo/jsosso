use std::io::{self, Write};
use std::collections::HashMap;
use std::str;
use super::*;

#[derive(Debug)]
struct Human {
    size: usize,
    line: u64,
    ind: String,
}

impl Json {
    /// Serializes the Json value into `Writer` 'w'.
    ///
    /// Example:
    /// ```
    /// use std::str;
    /// use std::io::{self};
    /// use jsosso::Json;
    ///
    /// let jdoc = Json::String("hello world!".to_string());
    /// let mut v: Vec<u8> = Vec::new();
    /// jdoc.serialize(&mut v);
    /// assert!(match str::from_utf8(&v) {
    ///     Ok(s) if s == "\"hello world!\"" => true,
    ///     _ => false,
    /// });
    /// ```
    pub fn serialize<W: Write> (&self, w: &mut W) -> io::Result<usize> {
        let mut h = Human {
            size: 0,
            line: 0,
            ind: "".to_string(),
        };

        self.write_jvalue(w, &mut h)?;
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
        self.write(w, h, b"\n")?;
        self.write_indent(w,h)?;
        self.write(w, h, b"]\n")
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
        self.write(w, h, b"\n")?;
        self.write_indent(w,h)?;
        self.write(w, h, b"}\n")
    }
}
