//use std::fs::File;
//use std::io::Read;

/*
mod fileio {

use std::fs::File;
use std::io::Read;

pub fn read_file(fname: &str) -> Option<String> {
    let r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_siz) => { Some(s) }
                _        => { None    }
            }
        }

        _ => { None }
    }
}
}
*/

/*
pub fn to_hex(v: Vec<u8>) -> String {

    let mut s = String::new();
    for i in v {
        s.push_str(&format!("{:02X}", i));
    }
    s
}
*/

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

/*
    #[test]
    fn test_to_hex() {
        
        let v: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        assert_eq!("0001090A0F10", super::to_hex(v));
    }
*/

}
