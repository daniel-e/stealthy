pub mod blowfish;
pub mod rsa;
pub mod tools;

pub struct Encryption {
    key: String
}

impl Encryption {

    pub fn new(key: &String) -> Encryption {
        Encryption {
            key: key.clone()
        }
    }

    fn blowfish(&self) -> blowfish::Blowfish {

        let k = tools::from_hex(self.key.clone());
        if k.is_none() {
            println!("Unable to initialize the crypto key.");
        }
        blowfish::Blowfish::from_key(k.unwrap()).unwrap()
    }

    pub fn encrypt(&self, v: Vec<u8>) -> Vec<u8> {

        let mut b = self.blowfish();
        let er    = b.encrypt(v);
        let mut r = er.iv;

        for i in er.ciphertext {
            r.push(i);
        }
        r
    }

    pub fn decrypt(&self, v: Vec<u8>) -> Option<Vec<u8>> {

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


/*
pub struct Encryption {
    blowfish: blowfish::Blowfish,
    rsa: rsa::RSAenc
}

impl Encryption {

    pub fn new(pubkey_pem: String, privkey_pem: String) -> Encryption {

        Encryption {
            rsa: rsa::RSAenc::new(Some(pubkey_pem), Some(privkey_pem)),
            blowfish: blowfish::Blowfish::new()
        }
    }

    pub fn encrypt(&mut self, msg: Vec<u8>) -> Option<String> {

        // encrypt message with blowfish
        let mut r = self.blowfish.encrypt(msg);  // cipher + iv
        let k = self.blowfish.key();         // symmetric key

        // encrypt the symmetric encryption with RSA
        let key_cipher = self.rsa.encrypt(k);
        if !key_cipher.is_some() {
            return None;
        }

        //Some(tools::to_hex(r.iv) + ":" + tools::to_hex(r.ciphertext) + ":" + tools::to_hex(key_cipher.unwrap()))
        None
    }
}

// ------------------------------------------------------------------------
// TESTS
// ------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::tools;

    #[test]
    fn test_bla() {

    }

    
    }
*/



/*
fn init_encryption() -> Option<Encryption> {

    // TODO hard coded
    let pubkey_file = "/home/dz/Dropbox/github/icmpmessaging-rs/testdata/rsa_pub.pem";
    let privkey_file = "/home/dz/Dropbox/github/icmpmessaging-rs/testdata/rsa_priv.pem";

    let pubkey = crypto::tools::read_file(pubkey_file);
    let privkey = crypto::tools::read_file(privkey_file);

    match pubkey.is_some() && privkey.is_some() {
        false => {
            println!("Could not read all required keys.");
            None
        }
        true  => { Some(Encryption::new(pubkey.unwrap(), privkey.unwrap())) }
    }
}
*/


