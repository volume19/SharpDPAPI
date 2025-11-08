// Enhanced Windows API interop with actual implementations
// This module provides Rust equivalents of Windows API calls

#[cfg(windows)]
use std::ptr;
#[cfg(windows)]
use std::mem;

use anyhow::{Result, anyhow};

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum CryptAlg {
    CalgMd2 = 32769,
    CalgMd4 = 32770,
    CalgMd5 = 32771,
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

pub struct Interop;

impl Interop {
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

    #[cfg(windows)]
    pub fn crypt_unprotect_data(encrypted_data: &[u8], entropy: Option<&[u8]>) -> Result<Vec<u8>> {
        use winapi::um::dpapi::{CryptUnprotectData, CRYPTOAPI_BLOB};
        use winapi::um::winbase::LocalFree;

        unsafe {
            let mut data_in = CRYPTOAPI_BLOB {
                cbData: encrypted_data.len() as u32,
                pbData: encrypted_data.as_ptr() as *mut u8,
            };

            let mut entropy_blob = if let Some(ent) = entropy {
                Some(CRYPTOAPI_BLOB {
                    cbData: ent.len() as u32,
                    pbData: ent.as_ptr() as *mut u8,
                })
            } else {
                None
            };

            let mut data_out = CRYPTOAPI_BLOB {
                cbData: 0,
                pbData: ptr::null_mut(),
            };

            let result = CryptUnprotectData(
                &mut data_in,
                ptr::null_mut(),
                entropy_blob.as_mut().map(|e| e as *mut _).unwrap_or(ptr::null_mut()),
                ptr::null_mut(),
                ptr::null_mut(),
                0,
                &mut data_out,
            );

            if result == 0 {
                return Err(anyhow!("CryptUnprotectData failed"));
            }

            let decrypted = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize).to_vec();
            LocalFree(data_out.pbData as *mut _);

            Ok(decrypted)
        }
    }

    #[cfg(not(windows))]
    pub fn crypt_unprotect_data(_encrypted_data: &[u8], _entropy: Option<&[u8]>) -> Result<Vec<u8>> {
        Err(anyhow!("CryptUnprotectData is only available on Windows"))
    }
}
