use std::fs::File;
use std::io::Read;

pub fn read_bin_file(fname: &str) -> Result<Vec<u8>, String> {

    let r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut v : Vec<u8> = vec![];
            match file.read_to_end(&mut v) {
                Ok(_) => Ok(v),
                _ => Err(format!("Could not read file {}", fname))
            }

        }
        _ => Err(format!("Could not open file '{}' for reading.", fname))
    }
}

pub fn read_file(fname: &str) -> Result<String, &'static str> {

    let r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_) => Ok(s),
                _     => Err("Could not read file")
            }
        }
        _ => Err("Could not open file for reading.")
    }
}

pub fn insert_delimiter(s: &str) -> String {
    match s.is_empty() {
        true  => String::from(""),
        false => {
            let (head, tail) = s.split_at(2);
            let r = insert_delimiter(tail);
            match r.is_empty() {
                true  => String::from(head),
                false => String::from(head) + ":" + &r
            }
        }
    }
}

