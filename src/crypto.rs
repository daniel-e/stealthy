use std::fs::File;
use std::io::Read;

use blowfish;
use blowfish::EncryptionResult;
use rsa;

pub trait Encryption : Send + Sync {
    fn encrypt(&self, v: &Vec<u8>) -> Vec<u8>;
    fn decrypt(&self, v: Vec<u8>) -> Option<Vec<u8>>;
}


pub struct SymmetricEncryption {
    key: String
}

pub struct AsymmetricEncryption {
    pub_key: String,
    priv_key: String
}

// ---------------------------------

impl SymmetricEncryption {

    pub fn new(key: &String) -> SymmetricEncryption {
        SymmetricEncryption {
            key: key.clone()
        }
    }

    fn blowfish(&self) -> blowfish::Blowfish {

        let k = from_hex(self.key.clone());
        if k.is_none() {
            println!("Unable to initialize the crypto key.");
        }
        blowfish::Blowfish::from_key(k.unwrap()).unwrap()
    }
}

impl Encryption for SymmetricEncryption {

    fn encrypt(&self, v: &Vec<u8>) -> Vec<u8> {

        let mut b = self.blowfish();
        let er    = b.encrypt(v);
        let mut r = er.iv;

        for i in er.ciphertext {
            r.push(i);
        }
        r
    }

    fn decrypt(&self, v: Vec<u8>) -> Option<Vec<u8>> {

        let mut b = self.blowfish();
        let k     = b.key();

        let (iv, cipher) = v.split_at(blowfish::IV_LEN);

        let mut x = Vec::new();
        for i in iv { x.push(*i) }
        let mut y = Vec::new();
        for i in cipher { y.push(*i) }

        let e = blowfish::EncryptionResult {
            iv: x,
            ciphertext: y
        };
        b.decrypt(e, k)
    }
}

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

impl Encryption for AsymmetricEncryption {

    fn encrypt(&self, v: &Vec<u8>) -> Vec<u8> {
        // 1. generate a random key and encrypt with blowfish -> ciphertext1, iv, key
        // 2. encrypt iv + key with public key -> ciphertext2
        // 3. return ciphertext2 + ciphertext1

        let mut blowfish = blowfish::Blowfish::new();
        let er = blowfish.encrypt(v);

        let iv = er.iv;
        let ciphertext1: Vec<u8> = er.ciphertext;
        let key = blowfish.key();

        let mut r = rsa::RSAenc::new(self.pub_key.clone(), self.priv_key.clone());
        let mut data = to_hex(iv);
        data.push_str(":");
        data.push_str(&to_hex(key));
        let ciphertext2: Option<Vec<u8>> = r.encrypt(data.into_bytes());

        match ciphertext2 {
            Some(cipher) => {
                let mut c = String::new();
                c.push_str(&to_hex(ciphertext1));  // TODO use more efficient encoding
                c.push_str(":");
                c.push_str(&to_hex(cipher));
                c.into_bytes()
            }
            _ => { vec![] } // TODO error handling
        }
    }

    fn decrypt(&self, v: Vec<u8>) -> Option<Vec<u8>> {

        match String::from_utf8(v) {
            Ok(cipher) => {
                let dec: Vec<&str> = cipher.split(':').collect();
                if dec.len() != 2 {
                    return None;
                }
                match from_hex(dec[0].to_string()) {
                    Some(ciphertext) => {
                        match from_hex(dec[1].to_string()) {
                            Some(cipher_iv_key) => {
                                let mut r = rsa::RSAenc::new(self.pub_key.clone(), self.priv_key.clone());
                                let raw_iv_key = r.decrypt(cipher_iv_key);
                                if raw_iv_key.is_none() {
                                    return None;
                                }
                                match String::from_utf8(raw_iv_key.unwrap()) {
                                    Ok(hex_str_iv_key) => {
                                        let iv_key: Vec<&str> = hex_str_iv_key.split(':').collect();
                                        if iv_key.len() != 2 {
                                            return None;
                                        }
                                        let hex_str_iv = iv_key[0].to_string();
                                        let hex_str_key = iv_key[1].to_string();
                                        match from_hex(hex_str_iv) {
                                            Some(iv) => {
                                                match from_hex(hex_str_key) {
                                                    Some(key) => {
                                                        let er = EncryptionResult {
                                                            iv: iv,
                                                            ciphertext: ciphertext
                                                        };
                                                        let mut b = blowfish::Blowfish::new();
                                                        b.decrypt(er, key)
                                                    }
                                                    _ => None
                                                }
                                            }
                                            _ => None
                                        }
                                    }
                                    _ => None
                                }
                            }
                            _ => None
                        }
                    }
                    _ => None
                }
            }
            _ => None
        }
    }
 
}

// ------------------------------------------------------------------

fn from_hex(s: String) -> Option<Vec<u8>> {

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

pub fn to_hex(v: Vec<u8>) -> String {

    let mut s = String::new();
    for i in v {
        s.push_str(&format!("{:02X}", i));
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
        assert!(!r.is_some());

        r = super::from_hex("0001090A0F10".to_string());
        assert!(r.is_some());

        let o: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        let v = r.unwrap();
        assert!(v.len() == 6);
        assert_eq!(o, v);
    }

    #[test]
    fn test_to_hex() {
        
        let v: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        assert_eq!("0001090A0F10", super::to_hex(v));
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
    fn test_asymmetric_encryp_decrypt() {
        
        let a = AsymmetricEncryption::new("testdata/rsa_pub.pem", "testdata/rsa_priv.pem");
        assert!(a.is_some());
        match a {
            Some(a) => {
                let plain  = "hello".to_string().into_bytes();
                let cipher = a.encrypt(&plain);
                let p      = a.decrypt(cipher).unwrap();
                assert_eq!(plain, p);
            }
            _ => { }
        }
    }
}
