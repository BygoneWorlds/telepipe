//! Message structures in PSO.

use std::io;
use std::io::{Read, Write, Cursor};
use serial::Serial;

use byteorder::{BigEndian as BE, ReadBytesExt, WriteBytesExt};

#[derive(Clone, Debug)]
pub enum Msg {
    Unknown(u8, u8, Vec<u8>)
}

impl Serial for Msg {
    fn serialize<W: Write>(&self, mut w: W) -> Result<(), io::Error> {
        let code;
        let flags;
        let mut cursor = Cursor::new(Vec::new());

        match self {
            &Msg::Unknown(ref c, ref f, ref b) => {
                code = *c;
                flags = *f;
                try!(cursor.write_all(b));
            }
        }

        let mut buf: Vec<u8> = cursor.into_inner();
        let buf_len = buf.len();
        buf.append(&mut vec![0; round_up_remainder(buf_len as u16, 4) as usize]);

        try!(w.write_u8(code));
        try!(w.write_u8(flags));
        try!(w.write_u16::<BE>(round_up(buf.len() as u16, 4)));
        try!(w.write_all(&buf));

        Ok(())
    }

    fn deserialize<R: Read>(mut r: R) -> Result<Msg, io::Error> {
        let code = try!(r.read_u8());
        let flags = try!(r.read_u8());
        let size = try!(r.read_u16::<BE>());
        let mut buf: Vec<u8> = vec![0; size as usize];
        try!(r.read_exact(&mut buf));

        let ret = match code {
            _ => Msg::Unknown(code, flags, buf)
        };

        Ok(ret)
    }
}

#[inline(always)]
fn round_up(val: u16, of: u16) -> u16 {
    if val % of == 0 {
        val
    } else {
        val + of
    }
}

#[inline(always)]
fn round_up_remainder(val: u16, of: u16) -> u16 {
    if val % of == 0 {
        0
    } else {
        of - (val % of)
    }
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use serial::Serial;

    use super::*;

    #[test]
    fn test_msg_serial_size() {
        let msg = Msg::Unknown(0, 0, vec![0; 20]);
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor).unwrap();
        let buf: Vec<u8> = cursor.into_inner();
        assert_eq!(buf.len(), 24);
    }

    #[test]
    fn test_msg_serial_padding() {
        let msg = Msg::Unknown(0, 0, vec![0; 21]);
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor).unwrap();
        let buf: Vec<u8> = cursor.into_inner();
        assert_eq!(buf.len(), 28);

        let msg = Msg::Unknown(0, 0, vec![0; 22]);
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor).unwrap();
        let buf: Vec<u8> = cursor.into_inner();
        assert_eq!(buf.len(), 28);

        let msg = Msg::Unknown(0, 0, vec![0; 23]);
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor).unwrap();
        let buf: Vec<u8> = cursor.into_inner();
        assert_eq!(buf.len(), 28);

        let msg = Msg::Unknown(0, 0, vec![0; 24]);
        let mut cursor = Cursor::new(Vec::new());
        msg.serialize(&mut cursor).unwrap();
        let buf: Vec<u8> = cursor.into_inner();
        assert_eq!(buf.len(), 28);
    }
}