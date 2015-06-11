extern crate rand;
extern crate libc;

use std::ptr;

#[repr(C)]
struct BIO_METHOD;

#[repr(C)]
struct BIO;

#[repr(C)]
struct EVP_PKEY;

#[repr(C)]
struct PEM_PASSWORD_CALLBACK;

#[repr(C)]
pub struct RSA;

#[link(name = "crypto")]
extern {
    // https://www.openssl.org/docs/crypto/BIO_s_mem.html
    fn BIO_s_mem() -> *mut BIO_METHOD;

    // https://www.openssl.org/docs/crypto/BIO_new.html
    fn BIO_new(typ: *mut BIO_METHOD) -> *mut BIO;

    // http://linux.die.net/man/3/bio_puts
    fn BIO_write(b: *mut BIO, buf: *const u8, len: libc::c_int) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/pem.html
    fn PEM_read_bio_PUBKEY(
        bp: *mut BIO, 
        x: *mut *mut EVP_PKEY, 
        cb: *mut PEM_PASSWORD_CALLBACK, u: *mut libc::c_void) -> *mut EVP_PKEY;

    // https://www.openssl.org/docs/crypto/pem.html
    fn PEM_read_bio_PrivateKey(
        bp: *mut BIO, 
        x: *mut *mut EVP_PKEY, 
        cb: *mut PEM_PASSWORD_CALLBACK, u: *mut libc::c_void) -> *mut EVP_PKEY;

    // https://www.openssl.org/docs/crypto/EVP_PKEY_set1_RSA.html
    fn EVP_PKEY_get1_RSA(pkey: *mut EVP_PKEY) -> *mut RSA;

    // https://www.openssl.org/docs/crypto/RSA_public_encrypt.html
    fn RSA_public_encrypt(
        flen: libc::c_int,
        from: *mut u8,
        to: *mut u8,
        rsa: *mut RSA,
        padding: libc::c_int) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/RSA_public_encrypt.html
    pub fn RSA_private_decrypt(
        flen: libc::c_int,
        from: *mut u8,
        to: *mut u8,
        rsa: *mut RSA,
        padding: libc::c_int) -> libc::c_int;

    // https://www.openssl.org/docs/crypto/RSA_size.html
    fn RSA_size(rsa: *const RSA) -> libc::c_int;
}

const RSA_PKCS1_OAEP_PADDING: libc::c_int = 4;   // openssl/rsa.h


enum CryptOperation {
    Encrypt,
    Decrypt
}

enum KeyType {
    PublicKey,
    PrivateKey
}

pub struct RSAenc {
    rsapub: *mut RSA,
    rsapriv: *mut RSA,
}

impl RSAenc {

    fn pem(pem: String, kt: KeyType) -> Option<*mut RSA> {

        // TODO BIO_new_mem_buf()
        unsafe {
            let bio_method = BIO_s_mem();
            if bio_method.is_null() {
                return None;
            }

            let bio: *mut BIO = BIO_new(bio_method);
            if bio.is_null() {
                return None;
            }

            if BIO_write(bio, pem.as_ptr(), pem.len() as libc::c_int) != pem.len() as libc::c_int {
                return None;
            }

            let r = match kt {
                KeyType::PublicKey => {
                    PEM_read_bio_PUBKEY(bio, ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
                }

                KeyType::PrivateKey => {
                    PEM_read_bio_PrivateKey(bio, ptr::null_mut(), ptr::null_mut(), ptr::null_mut())
                }
            };
            if r.is_null() {
                return None;
            }

            let rsa = EVP_PKEY_get1_RSA(r);
            if rsa.is_null() {
                return None;
            }

            Some(rsa)
        }
    }


    pub fn rsa_pubkey(pem: String) -> Option<*mut RSA> {

        RSAenc::pem(pem, KeyType::PublicKey)
    }

    pub fn rsa_privkey(pem: String) -> Option<*mut RSA> { // TODO duplicated code

        RSAenc::pem(pem, KeyType::PrivateKey)
    }


    fn crypt(&mut self, msg: Vec<u8>, op: CryptOperation) -> Option<Vec<u8>> {

        // TODO check message size
        // https://www.openssl.org/docs/crypto/RSA_public_encrypt.html

        unsafe {
            let siz = RSA_size(self.rsapub);

            let mut to: Vec<u8> = vec![];
            for _ in 0..siz {
                to.push(0);
            }

            let ret = match op {
                CryptOperation::Encrypt => {
                    RSA_public_encrypt(
                        msg.len() as libc::c_int, msg.as_ptr() as *mut u8, to.as_ptr() as *mut u8, 
                        self.rsapub, RSA_PKCS1_OAEP_PADDING)
                    }

                CryptOperation::Decrypt => {
                    RSA_private_decrypt(
                        msg.len() as libc::c_int, msg.as_ptr() as *mut u8, to.as_ptr() as *mut u8, 
                        self.rsapriv, RSA_PKCS1_OAEP_PADDING)
                    }
            };

            match ret {
                -1 => { None }
                _  => {
                    to.truncate(ret as usize);
                    Some(to)
                }
            }
        }
    }


    pub fn encrypt(&mut self, msg: Vec<u8>) -> Option<Vec<u8>> {

        self.crypt(msg, CryptOperation::Encrypt)
    }

    pub fn decrypt(&mut self, cipher: Vec<u8>) -> Option<Vec<u8>> {

        self.crypt(cipher, CryptOperation::Decrypt)
    }

    pub fn new(pubkey: String, privkey: String) -> RSAenc {

        // TODO error handling
        let rsapub = RSAenc::rsa_pubkey(pubkey).unwrap();
        let rsapriv = RSAenc::rsa_privkey(privkey).unwrap();

        RSAenc {
            rsapub: rsapub,
            rsapriv: rsapriv
        }
    }

}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

use std::fs::File;
use std::io::Read;

pub fn read_file(fname: &str) -> Option<String> {
    let mut r = File::open(fname);
    match r {
        Ok(mut file) => {
            let mut s = String::new();
            match file.read_to_string(&mut s) {
                Ok(siz) => { Some(s) }
                Err(e) => { None }
            }
        }

        Err(e) => { None }
    }
}

use std::env;

#[test]
fn test_encryption() {
    // use cargo test -- --nocapture to see output of print

    println!("path {}", env::current_dir().unwrap().display());

    let pubkey = read_file("testdata/rsa_pub.pem").unwrap();
    let privkey = read_file("testdata/rsa_priv.pem").unwrap();

    let mut rsa = super::RSAenc::new(pubkey, privkey);
    let plain   = "hello".to_string();

    let cipher = rsa.encrypt(plain.clone().into_bytes()).unwrap();

    assert!(cipher != plain.clone().into_bytes());
    assert!(cipher.len() >= 256);

    let p = String::from_utf8(rsa.decrypt(cipher).unwrap()).unwrap();
    assert_eq!(p, plain);
}
}
