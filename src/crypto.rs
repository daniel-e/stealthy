use std::fs::File;
use std::io::Read;

use super::{blowfish, rsa};

pub trait Encryption : Send + Sync {
    fn encrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String>;
    fn decrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String>;
}

pub struct SymmetricEncryption {
    algorithm: blowfish::Blowfish
}

pub struct AsymmetricEncryption {
    pub_key: String,
    priv_key: String
}

// ---------------------------------

impl SymmetricEncryption {

    pub fn new(hexkey: &String) -> Result<SymmetricEncryption, String> {

        Ok(SymmetricEncryption {
            algorithm: try!(blowfish::Blowfish::from_key(try!(from_hex(hexkey.clone()))))
        })
    }
}

impl Encryption for SymmetricEncryption {

    /// Encrypts the given data stored in a vector and returns the concatenated
    /// IV and ciphertext.
    fn encrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String> {

        self.algorithm.encrypt(v)
    }

    /// Decrypts the given daa stored in a vector and returns the plaintext.
    fn decrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String> {

        // TODO String -> &str in result
        self.algorithm.decrypt(v)
    }
}

// ---------------------------------

impl AsymmetricEncryption {

    pub fn new(pubkey_file: &str, privkey_file: &str) -> Option<AsymmetricEncryption> {

        match read_file(pubkey_file) {
            Some(pubkey) => {
                match read_file(privkey_file) {
                    Some(privkey) => Some(AsymmetricEncryption {
                            pub_key: pubkey,
                            priv_key: privkey
                        }),
                    _ => None
                }
            }
            _ => None
        }
    }
}

// ---------------------------------

impl Encryption for AsymmetricEncryption {

    fn encrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String> {
        // 1. generate a random key and encrypt with blowfish -> ciphertext1, iv, key
        // 2. encrypt iv + key with public key -> ciphertext2
        // 3. return ciphertext2 + ciphertext1

        match blowfish::Blowfish::new() {
            Err(e) => Err(e),
            Ok(blowfish) => {
                match blowfish.encrypt(v) {
                    Ok(ci) => {
                        let mut r = rsa::RSAenc::new(self.pub_key.clone(), self.priv_key.clone());
                        let cipher_of_key = r.encrypt(blowfish.key());

                        match cipher_of_key {
                            Some(cok) => {
                                let mut c = String::new();
                                c.push_str(&to_hex(ci));  // TODO use more efficient encoding
                                c.push_str(":");
                                c.push_str(&to_hex(cok));
                                Ok(c.into_bytes())
                            }
                            _ => { Ok(vec![]) } // TODO error handling
                        }
                    }
                    _ => Err("todo".to_string())
                }
            }
        }
    }

    fn decrypt(&self, v: &Vec<u8>) -> Result<Vec<u8>, String> {

        match split(v) {
            Some((ciphertext, cipher_key)) => {
                let mut r = rsa::RSAenc::new(self.pub_key.clone(), self.priv_key.clone());
                match r.decrypt(cipher_key) {
                    Some(raw_key) => {
                        match blowfish::Blowfish::from_key(raw_key) {
                            Ok(b) => b.decrypt(&ciphertext),
                            _ => Err("todo err".to_string())
                        }
                    }
                    _ => Err("todo".to_string())
                }
            }
            _ => Err("todo".to_string())
        }
    }
}

// ------------------------------------------------------------------

fn split(v: &Vec<u8>) -> Option<(Vec<u8>, Vec<u8>)> {
    
    match String::from_utf8(v.clone()) {
        Ok(cipher) => {
            let dec: Vec<&str> = cipher.split(':').collect();
            if dec.len() != 2 {
                return None;
            }
            match from_hex(dec[0].to_string()) {
                Ok(ciphertext) => {
                    match from_hex(dec[1].to_string()) {
                        Ok(cipher_iv_key) => Some((ciphertext, cipher_iv_key)),
                        _ => None
                    }
                }
                _ => None
            }
        }
        _ => None
    }
}

pub fn from_hex(s: String) -> Result<Vec<u8>, String> {

    let bytes = s.into_bytes();

    if bytes.len() % 2 != 0 {
        return Err("Length of hexadecimal string is not a multiple of 2.".to_string());
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
                _ => { return Err("Invalid character in hexadecimal string.".to_string()); }
            }
            p += 1;
        }
        v.push(b);
    }
    Ok(v)
}

pub fn to_hex(v: Vec<u8>) -> String {

    let mut s = String::new();
    for i in v {
        s.push_str(&format!("{:02x}", i));
    }
    s
}

pub fn read_file(fname: &str) -> Option<String> {

    let r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(_) => { Some(s) }
                _ => { None }
            }
        }
        _ => { None }
    }
}


// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    #[test]
    fn test_from_hex() {
        
        let mut r = super::from_hex("0".to_string());
        assert!(r.is_err());

        r = super::from_hex("0001090A0F10".to_string());
        assert!(r.is_ok());

        let o: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        let v = r.unwrap();
        assert!(v.len() == 6);
        assert_eq!(o, v);
    }

    #[test]
    fn test_to_hex() {
        
        let v: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        assert_eq!("0001090a0f10", super::to_hex(v));
    }

    // --------------------------------------------------------------
 
    use super::{Encryption, AsymmetricEncryption};

    #[test]
    fn test_asymmetric_encryption() {
        
        let a = AsymmetricEncryption::new("testdata/rsa_pub.pem", "testdata/rsa_priv.pem");
        assert!(a.is_some());

        let b = AsymmetricEncryption::new("testdata/rsa_pub.pem", "abc");
        assert!(b.is_none());

    }

    #[test]
    fn test_asymmetric_encrypt_decrypt() {
        
        let a = AsymmetricEncryption::new("testdata/rsa_pub.pem", "testdata/rsa_priv.pem");
        assert!(a.is_some());
        match a {
            Some(a) => {
                let plain  = "hello".to_string().into_bytes();
                let cipher = a.encrypt(&plain).unwrap();
                let p      = a.decrypt(&cipher).unwrap();
                assert_eq!(plain, p);
            }
            _ => { }
        }
    }
}
