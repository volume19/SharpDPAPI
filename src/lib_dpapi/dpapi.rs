// DPAPI blob parsing and decryption

use anyhow::{Result, anyhow};
use std::collections::HashMap;

use super::{Crypto, Helpers};

/// DPAPI (Data Protection API) operations for parsing and decrypting protected data.
///
/// The Dpapi struct provides methods for working with Windows DPAPI encrypted blobs,
/// including parsing blob structure and decrypting data using masterkeys.
pub struct Dpapi;

/// Represents a parsed DPAPI blob structure.
///
/// A DPAPI blob contains encrypted data protected by a masterkey. This structure
/// holds all the metadata and encrypted content from a DPAPI-protected blob.
///
/// # Fields
/// - `version`: DPAPI blob version (typically 1)
/// - `provider_guid`: GUID of the crypto provider
/// - `mk_version`: Masterkey version number
/// - `mk_guid`: GUID of the masterkey used for encryption
/// - `flags`: Encryption flags
/// - `description`: Optional description string
/// - `alg_crypt`: Encryption algorithm ID (e.g., 26115 for 3DES, 26128 for AES-256)
/// - `alg_crypt_len`: Length of encrypted key
/// - `salt`: Random salt for key derivation
/// - `alg_hash`: Hash algorithm ID (e.g., 32772 for SHA1, 32782 for SHA-512)
/// - `hmac`: HMAC for integrity verification
/// - `ciphertext`: Encrypted data payload
/// - `sign`: Digital signature (if present)
#[derive(Debug)]
pub struct DpapiBlob {
    pub version: u32,
    pub provider_guid: String,
    pub mk_version: u32,
    pub mk_guid: String,
    pub flags: u32,
    pub description: String,
    pub alg_crypt: i32,
    pub alg_crypt_len: i32,
    pub salt: Vec<u8>,
    pub alg_hash: i32,
    pub hmac: Vec<u8>,
    pub ciphertext: Vec<u8>,
    pub sign: Vec<u8>,
}

impl Dpapi {
    /// Parse and describe a DPAPI blob from raw bytes.
    ///
    /// # Arguments
    /// * `blob` - Raw DPAPI blob bytes to parse
    ///
    /// # Returns
    /// A `DpapiBlob` structure containing parsed metadata and encrypted data
    ///
    /// # Errors
    /// Returns an error if the blob is too small or has invalid structure
    ///
    /// # Examples
    /// ```ignore
    /// let blob_data = fs::read("encrypted.bin")?;
    /// let dpapi_blob = Dpapi::describe_blob(&blob_data)?;
    /// println!("Masterkey GUID: {}", dpapi_blob.mk_guid);
    /// ```
    pub fn describe_blob(blob: &[u8]) -> Result<DpapiBlob> {
        if blob.len() < 24 {
            return Err(anyhow!("Blob too small to be valid DPAPI blob"));
        }

        let mut offset = 0;

        // Version
        let version = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Provider GUID
        let provider_bytes = &blob[offset..offset + 16];
        let provider_guid = Self::format_guid(provider_bytes);
        offset += 16;

        let _blob_start = offset;

        // Master key version
        let mk_version = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Master key GUID
        let mk_guid_bytes = &blob[offset..offset + 16];
        let mk_guid = Self::format_guid(mk_guid_bytes);
        offset += 16;

        // Flags
        let flags = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Description length
        let descr_len = i32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]) as usize;
        offset += 4;

        // Description (UTF-16LE)
        let description = if descr_len > 0 && offset + descr_len <= blob.len() {
            String::from_utf16_lossy(
                &blob[offset..offset + descr_len]
                    .chunks(2)
                    .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                    .collect::<Vec<u16>>(),
            )
            .trim_end_matches('\0')
            .to_string()
        } else {
            String::new()
        };
        offset += descr_len;

        // Algorithm (crypt)
        let alg_crypt = i32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Algorithm length
        let alg_crypt_len = i32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Salt
        let salt_len = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]) as usize;
        offset += 4;

        let salt = if salt_len > 0 && offset + salt_len <= blob.len() {
            blob[offset..offset + salt_len].to_vec()
        } else {
            Vec::new()
        };
        offset += salt_len + 4; // +4 for strong structure

        // Hash algorithm
        let alg_hash = i32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // Hash length
        let _hash_len = i32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]);
        offset += 4;

        // HMAC
        let hmac_len = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]) as usize;
        offset += 4;

        let hmac = if hmac_len > 0 && offset + hmac_len <= blob.len() {
            blob[offset..offset + hmac_len].to_vec()
        } else {
            Vec::new()
        };
        offset += hmac_len;

        // Ciphertext
        let cipher_len = u32::from_le_bytes([
            blob[offset],
            blob[offset + 1],
            blob[offset + 2],
            blob[offset + 3],
        ]) as usize;
        offset += 4;

        let ciphertext = if cipher_len > 0 && offset + cipher_len <= blob.len() {
            blob[offset..offset + cipher_len].to_vec()
        } else {
            Vec::new()
        };
        offset += cipher_len;

        // Signature
        let sign_len = if offset + 4 <= blob.len() {
            u32::from_le_bytes([
                blob[offset],
                blob[offset + 1],
                blob[offset + 2],
                blob[offset + 3],
            ]) as usize
        } else {
            0
        };
        offset += 4;

        let sign = if sign_len > 0 && offset + sign_len <= blob.len() {
            blob[offset..offset + sign_len].to_vec()
        } else {
            Vec::new()
        };

        Ok(DpapiBlob {
            version,
            provider_guid,
            mk_version,
            mk_guid,
            flags,
            description,
            alg_crypt,
            alg_crypt_len,
            salt,
            alg_hash,
            hmac,
            ciphertext,
            sign,
        })
    }

    /// Decrypt a DPAPI blob using masterkeys
    pub fn decrypt_blob(
        blob: &DpapiBlob,
        masterkeys: &HashMap<String, String>,
        entropy: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        // Get the masterkey for this blob
        let mk_hex = masterkeys
            .get(&blob.mk_guid)
            .ok_or_else(|| anyhow!("Masterkey {} not found", blob.mk_guid))?;

        let key_bytes = Helpers::hex_to_bytes(mk_hex)?;

        // Derive the session key
        let derived_key = Crypto::derive_key(&key_bytes, &blob.salt, blob.alg_hash, entropy)?;

        // Take the appropriate number of bytes for the algorithm
        let key_len = (blob.alg_crypt_len / 8) as usize;
        let final_key = &derived_key[..key_len.min(derived_key.len())];

        // Decrypt the ciphertext
        let decrypted = Crypto::decrypt_blob(&blob.ciphertext, final_key, blob.alg_crypt)?;

        Ok(decrypted)
    }

    /// Format GUID from bytes
    fn format_guid(bytes: &[u8]) -> String {
        if bytes.len() != 16 {
            return String::from("{invalid-guid}");
        }

        format!(
            "{{{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}}}",
            bytes[3], bytes[2], bytes[1], bytes[0],
            bytes[5], bytes[4],
            bytes[7], bytes[6],
            bytes[8], bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }

    /// Describe and decrypt a DPAPI certificate private key
    pub fn describe_dpapi_cert_private_key(
        _file_name: &str,
        dpapi_blob: &[u8],
        master_keys: &HashMap<String, String>,
        _entropy: Option<&[u8]>,
        _unprotect: bool,
    ) -> Result<(String, Vec<u8>)> {
        let blob = Self::describe_blob(dpapi_blob)?;

        let message = format!(
            "\n    Provider GUID    : {}\n\
            Master Key GUID  : {}\n\
            Description      : {}\n\
            algCrypt         : {} (keyLen {})\n\
            algHash          : {} ({})\n\
            Salt             : {}\n\
            HMAC             : {}\n",
            blob.provider_guid,
            blob.mk_guid,
            blob.description,
            blob.alg_crypt,
            blob.alg_crypt_len,
            blob.alg_hash,
            blob.alg_hash,
            Helpers::bytes_to_hex(&blob.salt),
            Helpers::bytes_to_hex(&blob.hmac)
        );

        // Try to decrypt
        let decrypted = match Self::decrypt_blob(&blob, master_keys, None) {
            Ok(data) => data,
            Err(e) => {
                println!("    [!] Failed to decrypt: {}", e);
                Vec::new()
            }
        };

        Ok((message, decrypted))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_guid() {
        let bytes = [
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e,
            0x0f, 0x10,
        ];
        let guid = Dpapi::format_guid(&bytes);
        assert!(guid.starts_with('{'));
        assert!(guid.ends_with('}'));
    }
}
