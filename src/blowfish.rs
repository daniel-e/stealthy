extern crate rand;
extern crate libc;

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

    pub fn new() -> Blowfish { Blowfish::from_key(Blowfish::new_key()).unwrap() }

    pub fn from_key(key: Vec<u8>) -> Option<Blowfish> {

        if key.len() != KEY_LEN {
            return None;
        }

        Some(Blowfish {
            schedule: Box::new(BF_KEY {
                    p: [0; 18], 
                    s: [0; 4 * 256],
                }),
            key: key.clone()
        })
    }

    pub fn key(&self) -> Vec<u8> {
        self.key.clone()
    }

    /// Creates a random key.
    fn new_key() -> Vec<u8> {
        let mut key: Vec<u8> = vec![];;
        for _ in 0..KEY_LEN {
            key.push(rand::random::<u8>());  // TODO crypto rng
        }
        key
    }

    fn setup_key(&mut self) {
        unsafe {
            let k = self.key.clone();
            BF_set_key(&mut *(self.schedule), k.len() as libc::c_uint, k.as_ptr());
        }
    }

    fn padding(data: Vec<u8>) -> Vec<u8> {

        let mut r = data.clone();
        // PKCS#7 padding
        let padval = (8 - r.len() % 8) as u8;
        for _ in 0..padval {
            r.push(padval);
        }
        r
    }

    fn new_iv(len: usize) -> Vec<u8> {

        let mut iv: Vec<u8> = vec![];
        for _ in 0..len {
            iv.push(rand::random::<u8>()); // TODO crypto rng
        }
        iv
    }

    pub fn encrypt(&mut self, data: &Vec<u8>) -> EncryptionResult {

        self.setup_key();

        let plain = Blowfish::padding(data.clone());
        let iv    = Blowfish::new_iv(IV_LEN);

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

        EncryptionResult {
            iv: v,
            ciphertext: cipher,
        }
    }

    fn remove_padding(data: Vec<u8>) -> Option<Vec<u8>> {
    
        if data.len() < 8 {
            return None;
        }

        let mut plain  = data.clone();
        let     padval = plain.pop().unwrap();

        if padval > 8 {
            return None;
        }

        for _ in 0..(padval - 1) {
            let val = plain.pop();
            if val.is_none() || val.unwrap() != padval {
                return None;
            }
        }
        Some(plain)
    }

    fn set_key(&mut self, key: Vec<u8>) {

        self.key = key.clone();
    }

    pub fn decrypt(&mut self, e: EncryptionResult, key: Vec<u8>) -> Option<Vec<u8>> {

        self.set_key(key);
        self.setup_key();

        let iv     = e.iv.clone();
        let cipher = e.ciphertext.clone();
        let plain  = cipher.clone();

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

    fn print_u8_vector(v: Vec<u8>, s: &str) {

        print!("{}", s);
        for i in 0..v.len() {
            print!("{:02x} ", v[i]);
        }
        println!("");
    }

    #[test]
    fn test_encryption() {

        // use cargo test -- --nocapture to see output of print
        let mut b = super::Blowfish::new();
        let k = b.key().clone();

        println!("--------------------------------------");
        let v = "123456789".to_string().into_bytes();
        print_u8_vector(v.clone(), "plaintext: ");

        let r = b.encrypt(&v);

        print_u8_vector(r.iv.clone(), "iv       : ");
        print_u8_vector(r.ciphertext.clone(), "cipher   : ");

        let p = b.decrypt(r, k).unwrap();

        print_u8_vector(p, "decrypted: ");
        println!("--------------------------------------");
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
}
