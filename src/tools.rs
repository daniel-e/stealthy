extern crate term;

use std::fs::File;
use std::io::Read;

pub fn println_colored(msg: String, color: term::color::Color) {

    let mut t = term::stdout().unwrap();
    t.fg(color).unwrap();
    (write!(t, "{}", msg)).unwrap();
    t.reset().unwrap();
    (write!(t, "\n")).unwrap();
}

pub fn string_from_cstr(cstr: *const u8) -> String {

	let mut v: Vec<u8> = vec![];
	let mut i = 0;
	loop { unsafe {
		let c = *cstr.offset(i);
		if c == 0 { break; } else { v.push(c); }
		i += 1;
	}}
	String::from_utf8(v).unwrap()
}

pub fn push_slice(v: &mut Vec<u8>, arr: &[u8]) {
    for i in arr { 
        v.push(*i) 
    }
}

pub fn push_val(dst: &mut Vec<u8>, val: u64, n: usize) {
    let mut v = val;
    let mask = 0xff as u64;
    for _ in 0..n {
        let x: u8 = (v & mask) as u8;
        dst.push(x);
        v = v >> 8;
    }
}

pub fn pop_val(src: &mut Vec<u8>, n: usize) -> Option<u64> {
    let mut r: u64 = 0;
    if src.len() < n {
        return None;
    }
    for i in 0..n {
        r = r << 8;
        r = r + (src[n - 1 - i] as u64);
        src.remove(n - 1 - i); // TODO performance
    }
    Some(r)
}

pub fn read_file(fname: &str) -> Option<String> {
    let r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_siz) => { Some(s) }
                Err(_) => { None }
            }
        }

        Err(_) => { None }
    }
}

pub fn to_hex(v: Vec<u8>) -> String {

    let mut s = String::new();
    for i in v {
        s.push_str(&format!("{:02X}", i));
    }
    s
}

pub fn from_hex(s: String) -> Option<Vec<u8>> {

    let bytes = s.into_bytes();

    if bytes.len() % 2 != 0 {
        return None
    }

    let mut v: Vec<u8> = vec![];
    let mut p: usize = 0;
    while p < bytes.len() {
        let mut b: u8 = 0;
        for _ in 0..2 {
            b = b << 4;
            let val = bytes[p];
            match val {
                b'A'...b'F' => b += val - b'A' + 10,
                b'a'...b'f' => b += val - b'a' + 10,
                b'0'...b'9' => b += val - b'0',
                _ => { return None; }
            }
            p += 1;
        }
        v.push(b);
    }

    Some(v)
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::{push_slice, push_val, pop_val};

    #[test]
    fn test_push_slice() {
        let mut v: Vec<u8> = Vec::new();
        push_slice(&mut v, &[1, 2, 3]);
        assert_eq!(v.len(), 3);
        assert_eq!(v, vec![1, 2, 3]);
    }

    #[test]
    fn test_push_val() {
        let mut v: Vec<u8> = Vec::new();

        push_val(&mut v, 123, 2);
        assert_eq!(v, vec![123, 0]);

        v.clear();
        push_val(&mut v, 23 * 256 + 78, 2);
        assert_eq!(v, vec![78, 23]);
    }

    #[test]
    fn test_pop_val() {
        let mut v: Vec<u8> = vec![1, 2, 3];
        
        let mut i = pop_val(&mut v, 4);
        assert!(!i.is_some());

        v.clear();
        push_val(&mut v, 17 * 256 + 19, 2);
        push_val(&mut v, 34, 1);
        i = pop_val(&mut v, 2);
        assert_eq!(i.unwrap(), 17 * 256 + 19);
        assert_eq!(v.len(), 1);

        i = pop_val(&mut v, 1);
        assert_eq!(i.unwrap(), 34);
        assert_eq!(v.len(), 0);
    }

    #[test]
    fn test_to_hex() {
        
        let v: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        assert_eq!("0001090A0F10", super::to_hex(v));
    }

    #[test]
    fn test_from_hex() {
        
        let mut r = super::from_hex("0".to_string());
        assert!(!r.is_some());

        r = super::from_hex("0001090A0F10".to_string());
        assert!(r.is_some());

        let o: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        let v = r.unwrap();
        assert!(v.len() == 6);
        assert_eq!(o, v);
    }
}
