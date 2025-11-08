// Blob command - Describe and decrypt DPAPI blobs

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

use crate::lib_dpapi::{Dpapi, Helpers};

pub struct BlobCommand;

impl BlobCommand {
    pub fn execute(arguments: &HashMap<String, String>) -> Result<()> {
        println!("\r\n[*] Action: Describe DPAPI blob\r\n");

        let unprotect = arguments.contains_key("/unprotect");
        if unprotect {
            println!("[*] Using CryptUnprotectData() for decryption.\r\n");
        }

        // Get the target blob
        let blob_bytes = if let Some(target) = arguments.get("/target") {
            let target = target.trim_matches('"').trim_matches('\'');

            if Path::new(target).exists() {
                println!("[*] Reading blob from file: {}", target);
                fs::read(target)?
            } else {
                println!("[*] Decoding blob from base64");
                general_purpose::STANDARD.decode(target)
                    .map_err(|e| anyhow!("Failed to decode base64: {}", e))?
            }
        } else {
            return Err(anyhow!("[X] A /target:<BASE64 | file.bin> must be supplied!"));
        };

        println!("[*] Blob size: {} bytes\r\n", blob_bytes.len());

        // Parse the blob
        let blob = Dpapi::describe_blob(&blob_bytes)?;

        // Display blob information
        println!("    guidMasterKey    : {}", blob.mk_guid);
        println!("    size             : {}", blob_bytes.len());
        println!("    flags            : 0x{:08X}", blob.flags);
        println!("    algHash/algCrypt : {} / {}", blob.alg_hash, blob.alg_crypt);
        println!("    description      : {}", blob.description);
        println!("    salt             : {}", Helpers::bytes_to_hex(&blob.salt));
        println!("    hmac             : {}", Helpers::bytes_to_hex(&blob.hmac));

        // Try to decrypt
        let mut masterkeys = HashMap::new();

        // Collect {GUID}:SHA1 masterkeys from arguments
        for (key, value) in arguments.iter() {
            if !key.starts_with('/') && Helpers::is_guid(key) {
                masterkeys.insert(key.clone(), value.clone());
            }
        }

        // Load from file if specified
        if let Some(mkfile) = arguments.get("/mkfile") {
            let file_keys = Helpers::parse_masterkey_file(mkfile)?;
            masterkeys.extend(file_keys);
        }

        // Get entropy if specified
        let entropy = if let Some(ent_hex) = arguments.get("/entropy") {
            Some(Helpers::hex_to_bytes(ent_hex)?)
        } else {
            None
        };

        // Try decryption
        if unprotect {
            #[cfg(windows)]
            {
                use crate::lib_dpapi::interop::Interop;
                match Interop::crypt_unprotect_data(&blob_bytes, entropy.as_deref()) {
                    Ok(decrypted) => {
                        println!("\n[+] Successfully decrypted with CryptUnprotectData()");
                        display_decrypted_data(&decrypted);
                    }
                    Err(e) => {
                        println!("\n[!] CryptUnprotectData failed: {}", e);
                    }
                }
            }
            #[cfg(not(windows))]
            {
                println!("[!] CryptUnprotectData is only available on Windows");
            }
        } else if !masterkeys.is_empty() {
            match Dpapi::decrypt_blob(&blob, &masterkeys, entropy.as_deref()) {
                Ok(decrypted) => {
                    println!("\n[+] Successfully decrypted with masterkey {}", blob.mk_guid);
                    display_decrypted_data(&decrypted);
                }
                Err(e) => {
                    println!("\n[!] Decryption failed: {}", e);
                    println!("[!] Masterkey {} may be needed", blob.mk_guid);
                }
            }
        } else {
            println!("\n[!] No decryption method available");
            println!("[!] Provide masterkeys, /mkfile, or use /unprotect");
        }

        Ok(())
    }
}

fn display_decrypted_data(data: &[u8]) {
    // Check if data is likely Unicode text
    if is_unicode_text(data) {
        match from_utf16le(data) {
            Ok(text) => {
                println!("    dec(blob)        : {}", text.trim_end_matches('\0'));
                return;
            }
            Err(_) => {}
        }
    }

    // Check if it's ASCII text
    if data.iter().all(|&b| (b >= 0x20 && b <= 0x7E) || b == 0x0A || b == 0x0D || b == 0x09 || b == 0x00) {
        if let Ok(text) = String::from_utf8(data.to_vec()) {
            println!("    dec(blob)        : {}", text.trim_end_matches('\0'));
            return;
        }
    }

    // Otherwise display as hex
    let hex_data = data.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ");
    println!("    dec(blob)        : {}", hex_data);
}

fn is_unicode_text(data: &[u8]) -> bool {
    if data.len() < 2 || data.len() % 2 != 0 {
        return false;
    }

    // Check for null-terminated UTF-16LE patterns
    let mut null_count = 0;
    for chunk in data.chunks_exact(2) {
        if chunk[0] == 0 && chunk[1] == 0 {
            null_count += 1;
        }
    }

    // If more than 20% are nulls in the right positions, likely UTF-16
    null_count > data.len() / 10
}

fn from_utf16le(data: &[u8]) -> Result<String, std::string::FromUtf16Error> {
    let utf16_data: Vec<u16> = data
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    String::from_utf16(&utf16_data)
}
