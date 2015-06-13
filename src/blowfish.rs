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

    /// Sets a new encryptio / decryption key for this instance.
    fn set_key(&mut self, key: Vec<u8>) {
        self.key = key;
    }

    /// Returns a new vector padded via PKCS#7.
    fn padding(data: &Vec<u8>) -> Vec<u8> {

        let padval = 8 - data.len() % 8;
        data.iter().map(|&x| x).chain(iter::repeat(padval as u8).take(padval)).collect()
    }

    /// Removes the PKCS#7 padding.
    fn remove_padding(data: Vec<u8>) -> Option<Vec<u8>> {
    
        if data.len() >= 8 {
            let padval = *data.last().unwrap();
            if padval <= 8 {
                if data.iter().rev().take(padval as usize).all(|x| *x == padval) {
                    return Some(data.iter().take(data.len() - padval as usize).map(|&x| x).collect());
                }
            }
        }
        None
    }

    /// Function for encryption or decryption.
    fn crypt(&mut self, src: Vec<u8>, iv: Vec<u8>, mode: libc::c_long) -> Vec<u8> {

        let result = src.clone();
        let i = iv.clone();
        self.setup_key();
        unsafe {
            BF_cbc_encrypt(
                src.as_ptr(), 
                result.as_ptr() as *mut u8,
                src.len() as libc::c_long, 
                &mut *(self.schedule), 
                i.as_ptr() as *mut u8,
                mode
            );
        }
        result
    }

    /// Encrypts the data with the current key and a new IV.
    pub fn encrypt(&mut self, data: &Vec<u8>) -> Option<EncryptionResult> {

        match Blowfish::new_iv() {
            Some(iv) => {
                Some(EncryptionResult {
                    iv: iv.clone(),
                    ciphertext: self.crypt(Blowfish::padding(data), iv, BF_ENCRYPT)
                })
            }
            _ => None
        }
    }

    /// Decrypts the data.
    pub fn decrypt(&mut self, e: EncryptionResult, key: Vec<u8>) -> Option<Vec<u8>> {

        self.set_key(key);
        Blowfish::remove_padding(self.crypt(e.ciphertext, e.iv, BF_DECRYPT))
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use ::crypto::{from_hex, to_hex};
    use super::Blowfish;
    use std::ascii::AsciiExt;

    fn encrypt(s: &str, key: &str, iv: &str) -> String {

        let k = from_hex(key.to_string()).unwrap();
        let i = from_hex(iv.to_string()).unwrap();
        let mut b = Blowfish::from_key(k).unwrap();
        let src = s.to_string().into_bytes();
        to_hex(b.crypt(Blowfish::padding(&src), i, super::BF_ENCRYPT))
    }

    #[test]
    fn test_encryption() {

        // generated on the command line with openssl:
        // echo -n "abcdefg" | openssl enc -bf-cbc -e -K '11111111111111111111111111111111' 
        //    -iv '1111111111111111' -nosalt | xxd -ps
        assert_eq!(encrypt("abcdefg", "11111111111111111111111111111111", "1111111111111111"), 
            "a28c37bc94fef20d");
        assert_eq!(encrypt("abcdefg", "11111111111111111111111111111111", "2222222222222222"), 
            "600e966085f3fb7c");
        assert_eq!(encrypt("abcdefgh", "11111111111111111111111111111111", "1111111111111111"), 
            "39a79eeec0466eacea99fbb377af2d3f");
    }

    #[test]
    fn test_encryption_decryption() {

        let mut b = Blowfish::new().unwrap();
        let k = b.key();
        let v = "123456789".to_string().into_bytes();
        let r = b.encrypt(&v).unwrap();
        let p = b.decrypt(r, k).unwrap();
        assert_eq!(v, p);

        // check that two instances use different keys and different IVs
        // and that the ciphertext differs for the same plaintext
        let mut b1 = Blowfish::new().unwrap();
        let mut b2 = Blowfish::new().unwrap();
        let k1 = b1.key();
        let k2 = b2.key();
        assert!(k1 != k2);
        let c1 = b1.encrypt(&v).unwrap();
        let c2 = b2.encrypt(&v).unwrap();
        assert_eq!(c1.ciphertext.len(), 16);
        assert_eq!(c2.ciphertext.len(), 16);
        assert!(c1.iv != c2.iv);
        assert!(c1.ciphertext != c2.ciphertext);
        let p1 = b.decrypt(c1, k1).unwrap();
        let p2 = b.decrypt(c2, k2).unwrap();
        assert_eq!(p1, p2);
    }

    #[test]
    fn test_from_key() {

        let mut b = Blowfish::from_key(vec![0]);
        assert!(!b.is_some());
        let k = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        b = super::Blowfish::from_key(k.clone());
        assert!(b.is_some());
        assert_eq!(b.unwrap().key, k);
    }

     #[test]
    fn test_padding() {

        let a = vec![1, 2, 3, 5];
        let pa = Blowfish::padding(&a);
        assert_eq!(pa, vec![1, 2, 3, 5, 4, 4, 4, 4]);
        assert_eq!(Blowfish::remove_padding(pa).unwrap(), a);

        let b = vec![];
        let pb = Blowfish::padding(&b);
        assert_eq!(pb, vec![8 ,8, 8, 8, 8, 8, 8, 8]);
        assert_eq!(Blowfish::remove_padding(pb).unwrap(), b);

        let c = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let pc = Blowfish::padding(&c);
        assert_eq!(pc, vec![1 ,2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8]);
        assert_eq!(Blowfish::remove_padding(pc).unwrap(), c);

        let d = vec![1, 2, 3, 4, 5, 6, 7];
        let pd = Blowfish::padding(&d);
        assert_eq!(pd, vec![1 ,2, 3, 4, 5, 6, 7, 1]);
        assert_eq!(Blowfish::remove_padding(pd).unwrap(), d);
    }
}
