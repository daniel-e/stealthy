mod blowfish;
mod rsa;
pub mod tools;


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
