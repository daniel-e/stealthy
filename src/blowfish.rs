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


pub struct Blowfish {
    key: Vec<u8>
}

pub const KEY_LEN: usize = 16;
pub const IV_LEN: usize = 8;
pub const BLOCKSIZE: usize = 8;

type ResultVec = Result<Vec<u8>, &'static str>;

impl Blowfish {

    /// Returns a new instance of Blowfish with a random key.
    pub fn new() -> Result<Blowfish, &'static str> { 
        Blowfish::from_key(try!(Blowfish::new_key()))
    }

    /// Returns a new instance of Blowfish with the given key.
    pub fn from_key(key: Vec<u8>) -> Result<Blowfish, &'static str> {
        match key.len() {
            KEY_LEN => 
                Ok(Blowfish {
                    key: key
                }),
            _ => Err("Invalid key length.")
        }
    }

    /// Returns the current key used by this instance.
    pub fn key(&self) -> Vec<u8> {
        self.key.clone()
    }

    /// Returns cryptographically secure pseudorandom numbers for
    /// keys and initialization vectors.
    fn random_u8(n: usize) -> ResultVec {
        match OsRng::new() {
            Ok(mut r) => Ok(r.gen_iter::<u8>().take(n).collect()),
            _ => Err("Could not get OsRng.")
        }
    }

    /// Generates a new key.
    fn new_key() -> ResultVec {
        Blowfish::random_u8(KEY_LEN)
    }

    /// Generates a new initialization vector.
    fn new_iv() -> ResultVec {
        Blowfish::random_u8(IV_LEN)
    }

    /// Returns a new vector padded via PKCS#7.
    fn padding(data: &[u8]) -> Vec<u8> {

        let padval = BLOCKSIZE - data.len() % BLOCKSIZE;
        data.iter().cloned().chain(iter::repeat(padval as u8).take(padval)).collect()
    }

    /// Removes the PKCS#7 padding.
    fn remove_padding(data: &[u8]) -> ResultVec {
    
        if data.len() < BLOCKSIZE {
            return Err("Length of ciphertext is smaller than the block size.");
        }

        let padval = *data.last().unwrap();
        if padval as usize > BLOCKSIZE {
            return Err("Invalid padding value.");
        }

        let valid = data.iter().rev().take(padval as usize).all(|x| *x == padval);
        match valid {
            false => Err("Invalid padding."),
            true  => Ok(data.iter().take(data.len() - padval as usize).cloned().collect()),
        }
    }

    /// Function for encryption and decryption.
    fn crypt(&self, src: &[u8], iv: &[u8], key: Vec<u8>, mode: libc::c_long) -> Vec<u8> {

        let mut schedule = Box::new(BF_KEY {
                p: [0; 18], 
                s: [0; 4 * 256],
        });

        unsafe {
            BF_set_key(&mut *schedule, key.len() as libc::c_uint, key.as_ptr());
        }

        let result = src.clone();

        // We need to create a copy from the IV because it is modified by BF_cbc_encrypt.
        let i = iv.iter().cloned().collect::<Vec<u8>>();

        unsafe {
            BF_cbc_encrypt(
                src.as_ptr(), 
                result.as_ptr() as *mut u8,
                src.len() as libc::c_long, 
                &mut *schedule, 
                i.as_ptr() as *mut u8,
                mode
            );
        }
        result.iter().cloned().collect()
    }


    /// Encrypts the data with the current key and a new IV.
    pub fn encrypt(&self, data: &[u8]) -> ResultVec {

        let iv = try!(Blowfish::new_iv());
        Ok(
            iv.iter().cloned()
                .chain(
                    self.crypt(&Blowfish::padding(data), &iv, self.key.clone(), BF_ENCRYPT)
                        .iter().cloned())
                .collect()
        )
    }

    /// Decrypts the data.
    pub fn decrypt(&self, ciphertext: &[u8]) -> ResultVec {

        if IV_LEN > ciphertext.len() {
            return Err("Ciphertext has invalid length.");
        }

        let (iv, cipher) = ciphertext.split_at(IV_LEN);
        Blowfish::remove_padding(
            &self.crypt(cipher, iv, self.key.clone(), BF_DECRYPT)
        )
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    pub fn to_hex(v: Vec<u8>) -> String {

        let mut s = String::new();
        for i in v {
            s.push_str(&format!("{:02x}", i));
        }
        s
    }

    use ::crypto::{from_hex};
    use super::Blowfish;
    use std::ascii::AsciiExt;

    fn test_to_hex() {
        
        let v: Vec<u8> = vec![0, 1, 9, 10, 15, 16];
        assert_eq!("0001090a0f10", to_hex(v));
    }

    fn encrypt(s: &str, key: &str, iv: &str) -> String {

        let k = from_hex(key.to_string()).unwrap();
        let i = from_hex(iv.to_string()).unwrap();
        let mut b = Blowfish::new().unwrap();
        let src = s.to_string().into_bytes();
        to_hex(b.crypt(&Blowfish::padding(&src), &i, k, super::BF_ENCRYPT))
    }

    fn decrypt(hexcipher: &str, key: &str, iv: &str) -> String {

        let k = from_hex(key.to_string()).unwrap();
        let i = from_hex(iv.to_string()).unwrap();
        let mut b = Blowfish::new().unwrap();
        String::from_utf8(
            Blowfish::remove_padding(
                &b.crypt(
                    &from_hex(hexcipher.to_string()).unwrap(), &i, k, super::BF_DECRYPT
                )
            ).unwrap()
        ).unwrap()
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
    fn test_decryption() {

        assert_eq!(decrypt("a28c37bc94fef20d", "11111111111111111111111111111111", "1111111111111111"), 
            "abcdefg");
        assert_eq!(decrypt("600e966085f3fb7c", "11111111111111111111111111111111", "2222222222222222"), 
            "abcdefg");
        assert_eq!(decrypt("39a79eeec0466eacea99fbb377af2d3f", "11111111111111111111111111111111", "1111111111111111"), 
            "abcdefgh");
    }

    #[test]
    fn test_encryption_decryption() {

        let mut b = Blowfish::new().unwrap();
        let v = "123456789".to_string().into_bytes();
        let r = b.encrypt(&v).unwrap();
        let p = b.decrypt(&r).unwrap();
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
        assert_eq!(c1.len(), 16 + 8);
        assert_eq!(c2.len(), 16 + 8);
        assert!(c1 != c2);
        let p1 = b1.decrypt(&c1).unwrap();
        let p2 = b2.decrypt(&c2).unwrap();
        assert_eq!(p1, p2);
        assert_eq!(p1, v);

        let x = vec![1, 2, 3];
        assert!(b1.decrypt(&x).is_err());
    }

    #[test]
    fn test_from_key() {

        let mut b = Blowfish::from_key(vec![0]);
        assert!(b.is_err());
        let k = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        b = super::Blowfish::from_key(k.clone());
        assert!(b.is_ok());
        assert_eq!(b.unwrap().key, k);
    }

     #[test]
    fn test_padding() {

        let a = vec![1, 2, 3, 5];
        let pa = Blowfish::padding(&a);
        assert_eq!(pa, vec![1, 2, 3, 5, 4, 4, 4, 4]);
        assert_eq!(Blowfish::remove_padding(&pa).unwrap(), a);

        let b = vec![];
        let pb = Blowfish::padding(&b);
        assert_eq!(pb, vec![8 ,8, 8, 8, 8, 8, 8, 8]);
        assert_eq!(Blowfish::remove_padding(&pb).unwrap(), b);

        let c = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let pc = Blowfish::padding(&c);
        assert_eq!(pc, vec![1 ,2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8]);
        assert_eq!(Blowfish::remove_padding(&pc).unwrap(), c);

        let d = vec![1, 2, 3, 4, 5, 6, 7];
        let pd = Blowfish::padding(&d);
        assert_eq!(pd, vec![1 ,2, 3, 4, 5, 6, 7, 1]);
        assert_eq!(Blowfish::remove_padding(&pd).unwrap(), d);

        assert!(Blowfish::remove_padding(&[1, 2, 3, 4, 5, 6, 7]).is_err());
        assert!(Blowfish::remove_padding(&[8, 8 ,8 ,8 ,8 ,8 ,8 ,8]).is_ok());
        assert!(Blowfish::remove_padding(&[8, 8 ,8 ,8 ,8 ,8 ,8 ,9]).is_err());
        assert!(Blowfish::remove_padding(&[8, 7 ,7 ,7 ,7 ,7 ,7 ,7]).is_ok());
    }
}
