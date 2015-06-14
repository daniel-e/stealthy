extern crate libc;

use std::ptr;

#[repr(C)]
struct BIO;

#[repr(C)]
struct EVP_PKEY;

#[repr(C)]
struct PEM_PASSWORD_CALLBACK;

#[repr(C)]
pub struct RSA_;

#[link(name = "crypto")]
extern {
    // http://linux.die.net/man/3/bio_new_mem_buf
    fn BIO_new_mem_buf(buf: *const libc::c_void, len: libc::c_int) -> *mut BIO;

    // https://www.openssl.org/docs/crypto/BIO_new.html
    fn BIO_free(bio: *mut BIO) -> libc::c_int;

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
    fn EVP_PKEY_get1_RSA(pkey: *mut EVP_PKEY) -> *mut RSA_;

    // https://www.openssl.org/docs/crypto/EVP_PKEY_new.html
    fn EVP_PKEY_free(pkey: *mut EVP_PKEY) -> libc::c_void;

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

pub struct RSA {
    // TODO free memory on desctructor
    rsapub: *mut RSA_,
    rsapriv: *mut RSA_,
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


    fn crypt(&self, msg: &[u8], op: CryptOperation) -> Result<Vec<u8>, &'static str> {

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
                -1 => Err("Encryption or decryption with RSA failed."),
                _  => {
                    to.truncate(ret as usize);
                    Ok(to)
                }
            }
        }
    }


    pub fn encrypt(&self, msg: &[u8]) -> Result<Vec<u8>, &'static str> {
        self.crypt(msg, CryptOperation::Encrypt)
    }

    pub fn decrypt(&self, cipher: &[u8]) -> Result<Vec<u8>, &'static str> {
        self.crypt(cipher, CryptOperation::Decrypt)
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
