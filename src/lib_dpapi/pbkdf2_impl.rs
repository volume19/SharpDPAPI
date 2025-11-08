// PBKDF2 implementation wrapper

use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;
use sha2::Sha256;
use anyhow::Result;

pub struct Pbkdf2;

impl Pbkdf2 {
    /// PBKDF2 with SHA1
    pub fn derive_sha1(password: &[u8], salt: &[u8], iterations: u32, output: &mut [u8]) -> Result<()> {
        pbkdf2_hmac::<Sha1>(password, salt, iterations, output);
        Ok(())
    }

    /// PBKDF2 with SHA256
    pub fn derive_sha256(password: &[u8], salt: &[u8], iterations: u32, output: &mut [u8]) -> Result<()> {
        pbkdf2_hmac::<Sha256>(password, salt, iterations, output);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pbkdf2_sha1() {
        let password = b"password";
        let salt = b"salt";
        let mut output = [0u8; 20];
        Pbkdf2::derive_sha1(password, salt, 1000, &mut output).unwrap();
        assert_ne!(output, [0u8; 20]);
    }
}
