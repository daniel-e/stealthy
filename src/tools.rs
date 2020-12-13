use crypto::sha1::Sha1;
use crypto::digest::Digest;

use std::io::Read;
use std::io::Write;
use rand::{thread_rng, Rng};
use std::fs::{File, OpenOptions};

#[allow(dead_code)]
pub fn log_to_file(s: String) {
    match OpenOptions::new().append(true).create(true).open("/tmp/stealthy.log") {
        Ok(mut f) => {
            f.write_all(s.as_bytes()).expect("Cannot write into file.");
        },
        _ => panic!("Cannot open log file.")
    }
}

pub fn without_dirs(fname: &str) -> String {

    let mut parts: Vec<&str> = fname.split("/").collect();
    parts
        .pop()
        .expect("expected one element in vector")
        .to_string()
}

pub fn decode_uptime(t: i64) -> String {

    let days = t / 86400;
    if days > 0 {
        if days > 1 {
            format!("{} days ({} seconds)", days, t)
        } else {
            format!("{} day ({} seconds)", days, t)
        }
    } else {
        format!("{} seconds", t)
    }
}

pub fn write_data(fname: &str, data: Vec<u8>) -> bool {
    match File::create(fname) {
        Ok(mut f) => {
            f.write_all(&data).is_ok()
        },
        _ => false
    }
}

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

// public key encryption is deactivated at the moment
#[allow(dead_code)]
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

pub fn rot13(c: char) -> char {
    let x = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    let y = "NOPQRSTUVWXYZABCDEFGHIJKLMnopqrstuvwxyzabcdefghijklm";
    x.find(c).map_or(' ', |p| y.chars().nth(p).expect("ROT13 error"))
}

pub fn random_str(n: usize) -> String {
    let chars = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = thread_rng();
    String::from_utf8(
        (0..n).map(|_| { chars[rng.gen::<usize>() % chars.len()] }).collect()
    ).unwrap()
}

// public key encryption is deactivated at the moment
#[allow(dead_code)]
pub fn sha1(s: &[u8]) -> String {
    let mut h = Sha1::new();
    h.input(s);
    insert_delimiter(&h.result_str())
}
