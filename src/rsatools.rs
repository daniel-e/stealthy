extern crate rustc_serialize as serialize;
use self::serialize::base64::*;

pub fn key_as_der(pem: &String) -> Vec<u8> {
    
    let mut s = String::new();
    for i in pem.split("\n")
            .skip_while(|x| *x != "-----BEGIN PUBLIC KEY-----")
            .skip_while(|x| *x == "-----BEGIN PUBLIC KEY-----")
            .take_while(|x| *x != "-----END PUBLIC KEY-----") {
        s.push_str(i);
    }
    (&s).from_base64().unwrap()
}

