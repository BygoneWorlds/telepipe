//! Utility functions for Serial.

use std::io::{Read, Write};
use std::io;

use super::Serial;

pub fn read_ascii_len<R: Read>(len: u32, mut src: R) -> io::Result<String> {
    use encoding::all::ASCII;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;
    let mut r = vec![0u8; len as usize];
    try!(src.read(&mut r));
    // up to first null
    let mut end = len as usize;
    {
        for (i, c) in r.iter().enumerate() {
            if *c == 0 {
                debug!("truncating string repr by buffer {:?}", r);
                end = i;
                break;
            }
        }
    }
    debug!("decoding string truncated to {} from {}", end, len);
    match ASCII.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode ascii: {:?}", e)))
    }
}

pub fn write_ascii_len<W: Write>(s: &str, len: usize, mut dst: W) -> io::Result<()> {
    use encoding::all::ASCII;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let r = match ASCII.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode ASCII: {:?}", e)))
    };

    let padding: isize = len as isize - r.len() as isize;
    if padding < 0 {
        warn!("ascii string too long, truncating \"{}\" to fit", s);
        try!(dst.write_all(&r[..len]));
        Ok(())
    } else {
        debug!("encoding string padded to {} from {}", len, len as isize - padding);
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

pub fn read_utf16<R: Read>(mut src: R) -> io::Result<String> {
    use encoding::all::UTF_16LE;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;

    let mut buf = Vec::new();
    // Keep reading until EOF
    loop {
        let mut r_buf = [0u8; 2];
        let bytes_read = try!(src.read(&mut r_buf));
        if bytes_read != 2 {
            break;
        }
        if r_buf[0] == 0 && r_buf[1] == 0 {
            break;
        }
        buf.push(r_buf[0]);
        buf.push(r_buf[1]);
    }

    if buf.len() == 0 {
        return Ok("".to_string())
    }

    let r = match UTF_16LE.decode(&buf[..], Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to decode utf16: {:?}", e)))
    };
    Ok(r)
}

pub fn write_utf16<W: Write>(s: &str, mut dst: W) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let mut r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };
    r.push(0);
    r.push(0);
    try!(dst.write_all(&r[..]));
    Ok(())
}

pub fn read_utf16_len<R: Read>(len: usize, mut src: R) -> io::Result<String> {
    use encoding::all::UTF_16LE;
    use encoding::DecoderTrap::Replace;
    use encoding::Encoding;

    let real_len = len * 2;

    let mut r = vec![0u8; real_len as usize];
    try!(src.read(&mut r));
    // up to first 2 nulls
    let mut nulls = 0;
    let mut end = r.len();
    for (i, ch) in r.iter().enumerate() {
        if *ch == 0 {
            nulls += 1;
            if nulls == 2 {
                end = i;
                break
            }
        } else {
            nulls = 0;
        }
    }
    if end == 1 {
        return Ok("".to_string())
    }
    match UTF_16LE.decode(&r[..end], Replace) {
        Ok(s) => Ok(s),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Unable decode utf16: {:?}", e)))
    }
}

pub fn write_utf16_len(s: &str, len: usize, dst: &mut Write) -> io::Result<()> {
    use encoding::all::UTF_16LE;
    use encoding::EncoderTrap::Replace;
    use encoding::Encoding;

    let real_len = len * 2;

    let r = match UTF_16LE.encode(s, Replace) {
        Ok(s) => s,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Unable to encode utf16: {:?}", e)))
    };

    let padding: isize = real_len as isize - r.len() as isize;
    if padding < 0 {
        warn!("utf16 string too long, truncating \"{}\" to fit", s);
        try!(dst.write_all(&r[..real_len]));
        Ok(())
    } else {
        try!(dst.write_all(&r[..]));
        try!(dst.write_all(&vec![0u8; padding as usize]));
        Ok(())
    }
}

pub fn read_array<T: Serial + Default, R: Read>(len: usize, mut src: R) -> io::Result<Vec<T>> {
    let mut r = Vec::with_capacity(len);
    for _ in 0..len {
        r.push(try!(T::deserialize(&mut src)));
    }
    Ok(r)
}

pub fn write_array<T: Serial + Default, W: Write>(sl: &[T], len: usize, mut dst: W) -> io::Result<()> {
    if sl.len() > len as usize {
        warn!("Slice is larger than desired length, writing truncated");
        for i in sl.iter().take(len) {
            try!(i.serialize(&mut dst));
        }
        Ok(())
    } else {
        let padding = len as i32 - sl.len() as i32;
        for i in sl.iter() {
            try!(i.serialize(&mut dst));
        }
        for _ in 0..padding {
            try!(T::default().serialize(&mut dst));
        }
        Ok(())
    }
}
