// Windows API interop definitions
// This module provides Rust equivalents of Windows API calls and structures

#[cfg(windows)]
use windows::Win32::Security::Cryptography::*;

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CryptAlg {
    CalgMd2 = 32769,
    CalgMd4 = 32770,
    CalgMd5 = 32771,
    CalgSha = 32772,
    CalgSha1 = 32772,
    CalgMac = 32773,
    CalgRsaSign = 9216,
    CalgDssSign = 8704,
    CalgNoSign = 8192,
    CalgRsaKeyx = 41984,
    CalgDes = 26113,
    Calg3des112 = 26121,
    Calg3des = 26115,
    CalgDesx = 26116,
    CalgRc2 = 26114,
    CalgRc4 = 26625,
    CalgSeal = 26626,
    CalgAes128 = 26126,
    CalgAes192 = 26127,
    CalgAes256 = 26128,
    CalgAes = 26129,
    CalgSha256 = 32780,
    CalgSha384 = 32781,
    CalgSha512 = 32782,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum KerbEtype {
    DesCbcCrc = 1,
    DesCbcMd4 = 2,
    DesCbcMd5 = 3,
    Des3CbcMd5 = 5,
    Des3CbcSha1 = 7,
    DsaWithSha1CmsOid = 9,
    Md5WithRsaEncryption = 10,
    Sha1WithRsaEncryption = 11,
    Rc2CbcEnvOid = 12,
    RsaesOaepEnvOid = 13,
    Des3CbcSha1Kd = 16,
    Aes128CtsHmacSha196 = 17,
    Aes256CtsHmacSha196 = 18,
    Rc4Hmac = 23,
    Rc4HmacExp = 24,
    CamelliaCtsCmacSha196 = 25,
    Camellia256CtsCmacSha196 = 26,
    Subkey = 65,
}

pub struct Interop;

impl Interop {
    #[cfg(windows)]
    pub fn crypt_unprotect_data(encrypted_data: &[u8], entropy: Option<&[u8]>) -> Option<Vec<u8>> {
        // Windows-specific DPAPI call
        // This would use CryptUnprotectData from Windows API
        // Placeholder for now
        None
    }

    #[cfg(not(windows))]
    pub fn crypt_unprotect_data(_encrypted_data: &[u8], _entropy: Option<&[u8]>) -> Option<Vec<u8>> {
        None
    }

    /// Get algorithm name from ID
    pub fn get_alg_name(alg: i32) -> &'static str {
        match alg {
            32772 => "CALG_SHA1",
            32780 => "CALG_SHA_256",
            32781 => "CALG_SHA_384",
            32782 => "CALG_SHA_512",
            26113 => "CALG_DES",
            26115 => "CALG_3DES",
            26128 => "CALG_AES_256",
            _ => "UNKNOWN",
        }
    }
}
