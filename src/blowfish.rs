extern crate rand;
extern crate libc;

use self::rand::{OsRng, Rng};
use std::iter;

#[repr(C)]
struct BF_KEY {
    p: [libc::c_uint; 18],
    s: [libc::c_uint; 4 * 256]
}

#[link(name = "crypto")]
extern {
    fn BF_set_key(
        key: *mut BF_KEY, 
        len: libc::c_uint, // typically 16 bytes (128 bit)
        data: *const u8
    );

    // https://www.openssl.org/docs/crypto/blowfish.html
    fn BF_cbc_encrypt(
        plaintext: *const u8,  // plaintext must be a multiple of 8 bytes
        cipher: *mut u8,       // buffer must be as long as the plaintext
        length: libc::c_long,  // length of the plaintext
        schedule: *mut BF_KEY, // the key
        ivec: *mut u8,         // iv, 8 bytes
        enc: libc::c_long      // whether encryption BF_ENCRYPT or decryption BF_DECRYPT
    );
}

const BF_ENCRYPT: libc::c_long = 1; // values taken from header file
const BF_DECRYPT: libc::c_long = 0;


pub struct EncryptionResult {
    pub iv: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

pub struct Blowfish {
    schedule: Box<BF_KEY>,
    key: Vec<u8>
}

pub const KEY_LEN: usize = 16;
pub const IV_LEN: usize = 8;

impl Blowfish {

    /// Returns a new instance of Blowfish with a random key.
    pub fn new() -> Option<Blowfish> { 
        match Blowfish::new_key() {
            Some(k) => Blowfish::from_key(k),
            _ => None
        }
    }

    /// Returns a new instance of Blowfish with the given key.
    pub fn from_key(key: Vec<u8>) -> Option<Blowfish> {
        match key.len() {
            KEY_LEN => 
                Some(Blowfish {
                    schedule: Box::new(BF_KEY {
                        p: [0; 18], 
                        s: [0; 4 * 256],
                    }),
                    key: key
                }),
            _ => None
        }
    }

    /// Returns the current key used by this instance.
    pub fn key(&self) -> Vec<u8> {
        self.key.clone()
    }

    /// Returns cryptographically secure pseudorandom numbers for
    /// keys and initialization vectors.
    fn random_u8(n: usize) -> Option<Vec<u8>> {
        match OsRng::new() {
            Ok(mut r) => Some(r.gen_iter::<u8>().take(n).collect()),
            _ => None
        }
    }

    /// Generates a new key.
    fn new_key() -> Option<Vec<u8>> {
        Blowfish::random_u8(KEY_LEN)
    }

    /// Generates a new initialization vector.
    fn new_iv() -> Option<Vec<u8>> {
        Blowfish::random_u8(IV_LEN)
    }

    /// Initializes the configured key.
    fn setup_key(&mut self) {
        unsafe {
            let k = self.key.clone();
            BF_set_key(&mut *(self.schedule), k.len() as libc::c_uint, k.as_ptr());
        }
    }

    /// Returns a new vector padded via PKCS#7.
    fn padding(data: &Vec<u8>) -> Vec<u8> {

        let padval = 8 - data.len() % 8;
        data.iter().map(|&x| x).chain(iter::repeat(padval as u8).take(padval)).collect()
    }

    fn remove_padding(data: Vec<u8>) -> Option<Vec<u8>> {
    
        let mut plain = data.clone();
        if plain.len() < 8 {
            return None;
        }

        match plain.pop().unwrap() {
            padval @ 0 ... 8 => {
                for _ in 0..(padval - 1) {
                    if plain.pop().unwrap() != padval {
                        return None;
                    }
                }
                Some(plain)
            }
            _ => None
        }
    }

    pub fn encrypt(&mut self, data: &Vec<u8>) -> Option<EncryptionResult> {

        self.setup_key();
        let plain = Blowfish::padding(data);

        match Blowfish::new_iv() {
            Some(iv) => {
                // we need a copy of the iv because it is modified by
                // BF_cbc_encrypt
                let v = iv.clone();
                let cipher = plain.clone();

                unsafe {
                    BF_cbc_encrypt(
                        plain.as_ptr(), 
                        cipher.as_ptr() as *mut u8,
                        plain.len() as libc::c_long, 
                        &mut *(self.schedule), 
                        iv.as_ptr() as *mut u8,
                        BF_ENCRYPT as libc::c_long
                    );
                }

                Some(EncryptionResult {
                    iv: v,
                    ciphertext: cipher,
                })
            }
            _ => None
        }
    }

    fn set_key(&mut self, key: Vec<u8>) {

        self.key = key.clone();
    }

    pub fn decrypt(&mut self, e: EncryptionResult, key: Vec<u8>) -> Option<Vec<u8>> {

        self.set_key(key);
        self.setup_key();

        let iv = e.iv.clone();
        let cipher = e.ciphertext.clone();
        let plain = cipher.clone();

        unsafe {
            BF_cbc_encrypt(
                cipher.as_ptr(),
                plain.as_ptr() as *mut u8,
                cipher.len() as libc::c_long,
                &mut *(self.schedule),
                iv.as_ptr() as *mut u8,
                BF_DECRYPT as libc::c_long
            );
        }

        Blowfish::remove_padding(plain)
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    #[test]
    fn test_encryption() {

        let mut b = super::Blowfish::new().unwrap();
        let k = b.key();
        let v = "123456789".to_string().into_bytes();
        let r = b.encrypt(&v).unwrap();
        let p = b.decrypt(r, k).unwrap();
        assert_eq!(v, p);

        // check that two instances use different keys and different IVs
        // and that the ciphertext differs for the same plaintext
        let mut b1 = super::Blowfish::new().unwrap();
        let mut b2 = super::Blowfish::new().unwrap();
        let k1 = b1.key();
        let k2 = b2.key();
        assert!(k1 != k2);
        let c1 = b1.encrypt(&v).unwrap();
        let c2 = b2.encrypt(&v).unwrap();
        assert!(c1.iv != c2.iv);
        assert!(c1.ciphertext != c2.ciphertext);
        let p1 = b.decrypt(c1, k1).unwrap();
        let p2 = b.decrypt(c2, k2).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_from_key() {

        let mut b = super::Blowfish::from_key(vec![0]);
        assert!(!b.is_some());
        let k = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        b = super::Blowfish::from_key(k.clone());
        assert!(b.is_some());
        assert_eq!(b.unwrap().key, k);
    }
     #[test]
    fn test_padding() {

        let a = vec![1, 2, 3, 5];
        assert_eq!(super::Blowfish::padding(&a), vec![1, 2, 3, 5, 4, 4, 4, 4]);
        let b = vec![];
        assert_eq!(super::Blowfish::padding(&b), vec![8 ,8, 8, 8, 8, 8, 8, 8]);
        let c = vec![1, 2, 3, 4, 5, 6, 7, 8];
        assert_eq!(super::Blowfish::padding(&c), vec![1 ,2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8]);
        let d = vec![1, 2, 3, 4, 5, 6, 7];
        assert_eq!(super::Blowfish::padding(&d), vec![1 ,2, 3, 4, 5, 6, 7, 1]);
    }
}
