extern crate rand;
extern crate libc;

use self::rand::{OsRng, Rng};
use std::{iter, ptr};

/*
#[repr(C)]
struct BIO;

#[repr(C)]
struct EvpPkey;

#[repr(C)]
struct PemPasswordCallback;
*/
//#[repr(C)]
//pub struct RSA_;

pub enum BIO {}
pub enum EvpPkey {}
pub enum PemPasswordCallback {}
pub enum RSA_ {}

#[link(name = "crypto")]
extern {
    // http://linux.die.net/man/3/bio_new_mem_buf
    fn BIO_new_mem_buf(buf: *const libc::c_void, len: libc::c_int) -> *mut BIO;

    // https://www.openssl.org/docs/crypto/BIO_new.html
    fn BIO_free(bio: *mut BIO) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/pem.html
    fn PEM_read_bio_PUBKEY(
        bp: *mut BIO, 
        x: *mut *mut EvpPkey, 
        cb: *mut PemPasswordCallback, u: *mut libc::c_void) -> *mut EvpPkey;

    // https://www.openssl.org/docs/crypto/pem.html
    fn PEM_read_bio_PrivateKey(
        bp: *mut BIO, 
        x: *mut *mut EvpPkey, 
        cb: *mut PemPasswordCallback, u: *mut libc::c_void) -> *mut EvpPkey;

    // https://www.openssl.org/docs/crypto/EvpPkey_set1_RSA.html
    fn EVP_PKEY_get1_RSA(pkey: *mut EvpPkey) -> *mut RSA_;

    // https://www.openssl.org/docs/crypto/EvpPkey_new.html
    fn EVP_PKEY_free(pkey: *mut EvpPkey) -> libc::c_void;

    // https://www.openssl.org/docs/crypto/RSA_public_encrypt.html
    fn RSA_public_encrypt(
        flen: libc::c_int,
        from: *mut u8,
        to: *mut u8,
        rsa: *mut RSA_,
        padding: libc::c_int) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/RSA_public_encrypt.html
    pub fn RSA_private_decrypt(
        flen: libc::c_int,
        from: *mut u8,
        to: *mut u8,
        rsa: *mut RSA_,
        padding: libc::c_int) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/RSA_size.html
    fn RSA_size(rsa: *const RSA_) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/RSA_new.html
    fn RSA_free(rsa: *mut RSA_);

    // https://www.openssl.org/docs/crypto/rand.html
    fn RAND_seed(buf: *const libc::c_void, len: libc::c_int);
}

const RSA_PKCS1_OAEP_PADDING: libc::c_int = 4;   // openssl/rsa.h


enum KeyType {
    PublicKey,
    PrivateKey
}

pub struct RSA {
    rsapub: *mut RSA_,
    rsapriv: *mut RSA_,
}

impl Drop for RSA {

    fn drop(&mut self) {
        unsafe {
            RSA_free(self.rsapub);
            RSA_free(self.rsapriv);
        }
    }
}

impl RSA {

    fn pem(pem: &String, kt: KeyType) -> Result<*mut RSA_, &'static str> {

        unsafe {
            // the point to pem must be valid until BIO is freed.
            let bio: *mut BIO = BIO_new_mem_buf(pem.as_ptr() as *const libc::c_void, pem.len() as libc::c_int);
            if bio.is_null() {
                return Err("Could not initialize bio.");
            }

            let r = match kt {
                KeyType::PublicKey => 
                    PEM_read_bio_PUBKEY(bio, ptr::null_mut(), ptr::null_mut(), ptr::null_mut()),
                KeyType::PrivateKey =>
                    PEM_read_bio_PrivateKey(bio, ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
            };

            if BIO_free(bio) == 0 {
                return Err("Could not free bio.");
            } else if r.is_null() {
                return Err("Could not initialize key.");
            }

            let rsa = EVP_PKEY_get1_RSA(r);
            EVP_PKEY_free(r);

            match rsa.is_null() {
                false => Ok(rsa),
                true => Err("Could not get RSA structure.")
            }
        }
    }


    fn rsa_pubkey(pem: &String) -> Result<*mut RSA_, &'static str> {
        RSA::pem(pem, KeyType::PublicKey)
    }

    fn rsa_privkey(pem: &String) -> Result<*mut RSA_, &'static str> {
        RSA::pem(pem, KeyType::PrivateKey)
    }

    fn seed_rand() -> Result<(), &'static str> {

        match OsRng::new() {
            Ok(mut r) => {
                let mut seed: [u8; 32] = [0; 32];
                r.fill_bytes(&mut seed);
                unsafe {
                    RAND_seed(
                        seed.as_ptr() as *const libc::c_void,
                        seed.len() as libc::c_int
                    );
                    Ok(())
                }
            }
            _ => Err("Could not get OsRng.")
        }
    }

    fn crypt(f: unsafe extern "C" fn(
                flen: libc::c_int, from: *mut u8, to: *mut u8, rsa: *mut RSA_, padding: libc::c_int) -> libc::c_int, 
             msg: &[u8], 
             key: *mut RSA_) -> Result<Vec<u8>, &'static str> {

        unsafe {
            let siz = RSA_size(key) as usize;
            let mut buf = iter::repeat(0).take(siz).collect::<Vec<u8>>();

            let ret = f(
                msg.len()    as libc::c_int, 
                msg.as_ptr() as *mut u8, 
                buf.as_ptr() as *mut u8, 
                key, 
                RSA_PKCS1_OAEP_PADDING
            );

            match ret {
                -1 => Err("Encryption or decryption with RSA failed."),
                _  => {
                    buf.truncate(ret as usize);
                    Ok(buf)
                }
            }
        }
    }

    pub fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, &'static str> {

        // "rng must be seeded prior to calling this method"
        try!(Self::seed_rand());

        unsafe {
            // For encryption message length must be less than siz - 41
            // for RSA_PKCS1_OAEP_PADDING.
            let siz = RSA_size(self.rsapub) as usize;
            if msg.len() >= siz - 41 {
                return Err("Message too large for RSA_PKCS1_OAEP_PADDING");
            }

            Self::crypt(RSA_public_encrypt, msg, self.rsapub)
        }
    }

    pub fn decrypt(&self, cipher: &[u8]) -> Result<Vec<u8>, &'static str> {
        Self::crypt(RSA_private_decrypt, cipher, self.rsapriv)
    }

    pub fn new(pubkey: &String, privkey: &String) -> Result<RSA, &'static str> {

        Ok(RSA {
            rsapub: try!(RSA::rsa_pubkey(pubkey)),
            rsapriv: try!(RSA::rsa_privkey(privkey))
        })
    }

}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use ::crypto::read_file;
    use ::rsa::RSA;

    #[test]
    fn test_new() {

        let pubkey = read_file("testdata/rsa_pub.pem").unwrap();
        let privkey = read_file("testdata/rsa_priv.pem").unwrap();

        let rsa1 = RSA::new(&"abc".to_string(), &"def".to_string());
        assert!(rsa1.is_err());
        let rsa2 = RSA::new(&pubkey, &"abc".to_string());
        assert!(rsa2.is_err());
        let rsa3 = RSA::new(&"abc".to_string(), &privkey);
        assert!(rsa3.is_err());
        let rsa4 = RSA::new(&pubkey, &privkey);
        assert!(rsa4.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt() {
        // use cargo test -- --nocapture to see output of print

        let pubkey = read_file("testdata/rsa_pub.pem").unwrap();
        let privkey = read_file("testdata/rsa_priv.pem").unwrap();

        let rsa = RSA::new(&pubkey, &privkey).unwrap();
        let plain = "hello".to_string();

        let cipher = rsa.encrypt(&plain.clone().into_bytes()).unwrap();

        assert!(cipher != plain.clone().into_bytes());
        assert!(cipher.len() >= 256);

        let p = String::from_utf8(rsa.decrypt(&cipher).unwrap()).unwrap();
        assert_eq!(p, plain);
    }
}
