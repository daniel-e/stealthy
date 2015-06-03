use std::fs::File;
use std::io::Read;

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
