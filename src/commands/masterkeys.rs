// Masterkeys command - Triage and decrypt DPAPI masterkeys

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

use crate::lib_dpapi::Helpers;

pub struct MasterkeysCommand;

impl MasterkeysCommand {
    pub fn execute(arguments: &HashMap<String, String>) -> Result<()> {
        println!("\r\n[*] Action: User DPAPI Masterkey File Triage\r\n");

        let mut backup_key_bytes: Option<Vec<u8>> = None;
        let password = arguments.get("/password").map(|s| s.as_str());
        let ntlm = arguments.get("/ntlm").map(|s| s.as_str());
        let credkey = arguments.get("/credkey").map(|s| s.as_str());
        let computer_name = arguments.get("/server").map(|s| s.as_str());
        let target = arguments.get("/target").map(|s| s.as_str());
        let sid = arguments.get("/sid").map(|s| s.as_str());
        let hashes = arguments.contains_key("/hashes");
        let rpc = arguments.contains_key("/rpc");

        // Handle PVK backup key
        if let Some(pvk) = arguments.get("/pvk") {
            if Path::new(pvk).exists() {
                println!("[*] Loading DPAPI backup key from file: {}", pvk);
                backup_key_bytes = Some(fs::read(pvk)?);
            } else {
                println!("[*] Decoding DPAPI backup key from base64");
                backup_key_bytes = Some(
                    general_purpose::STANDARD.decode(pvk)
                        .map_err(|e| anyhow!("Failed to decode backup key: {}", e))?
                );
            }
        }

        // Validation
        if (password.is_some() || ntlm.is_some() || credkey.is_some())
            && target.is_some()
            && sid.is_none()
        {
            return Err(anyhow!(
                "[X] When using /password, /ntlm, or /credkey with /target:X, a /sid:X (domain user SID) is required!"
            ));
        }

        if !(password.is_some() || ntlm.is_some() || credkey.is_some())
            && backup_key_bytes.is_none()
            && !rpc
            && !hashes
        {
            return Err(anyhow!(
                "[X] A /pvk:BASE64 domain DPAPI backup key, /rpc, /password, /ntlm, /credkey, or /hashes must be supplied!"
            ));
        }

        // Find masterkey files
        let masterkey_files = if let Some(target_path) = target {
            find_masterkey_files_target(target_path)?
        } else {
            find_masterkey_files_local(computer_name)?
        };

        println!("[*] Found {} masterkey files\r\n", masterkey_files.len());

        if hashes {
            // Display as hashes for cracking
            println!("[*] Will dump user masterkey hashes\r\n");
            for mk_file in &masterkey_files {
                if let Ok(hash) = extract_masterkey_hash(mk_file) {
                    println!("{}", hash);
                }
            }
        } else {
            // Try to decrypt masterkeys
            let masterkey_cache: HashMap<String, String> = HashMap::new();

            for mk_file in &masterkey_files {
                println!("[*] Found MasterKey : {}", mk_file);

                if let Some(ref _backup_key) = backup_key_bytes {
                    // TODO: Implement actual masterkey decryption with backup key
                    println!("    [!] Backup key decryption not yet fully implemented");
                } else if password.is_some() {
                    println!("    [!] Password-based decryption not yet fully implemented");
                } else if ntlm.is_some() {
                    println!("    [!] NTLM-based decryption not yet fully implemented");
                } else if credkey.is_some() {
                    println!("    [!] Credkey-based decryption not yet fully implemented");
                } else if rpc {
                    println!("    [!] RPC-based decryption not yet fully implemented");
                }
            }

            if !masterkey_cache.is_empty() {
                println!("\r\n[*] User master key cache:\r\n");
                for (guid, key) in &masterkey_cache {
                    println!("{}:{}", guid, key);
                }
            }
        }

        Ok(())
    }
}

fn find_masterkey_files_local(computer_name: Option<&str>) -> Result<Vec<String>> {
    let mut files = Vec::new();

    let base_path = if let Some(server) = computer_name {
        format!("\\\\{}\\C$\\Users\\", server)
    } else {
        #[cfg(windows)]
        {
            format!("{}\\Users\\", std::env::var("SystemDrive").unwrap_or_else(|_| "C:".to_string()))
        }
        #[cfg(not(windows))]
        {
            return Err(anyhow!("Local user enumeration only available on Windows"));
        }
    };

    // Find user directories
    if let Ok(entries) = fs::read_dir(&base_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().unwrap().to_string_lossy();

                // Skip system directories
                if dir_name == "Public" || dir_name == "Default" || dir_name == "Default User" || dir_name == "All Users" {
                    continue;
                }

                // Look for masterkey files
                let protect_path = path.join("AppData").join("Roaming").join("Microsoft").join("Protect");
                if let Ok(protect_entries) = fs::read_dir(&protect_path) {
                    for protect_entry in protect_entries.flatten() {
                        let sid_path = protect_entry.path();
                        if sid_path.is_dir() {
                            if let Ok(mk_entries) = fs::read_dir(&sid_path) {
                                for mk_entry in mk_entries.flatten() {
                                    let mk_path = mk_entry.path();
                                    if mk_path.is_file() {
                                        if let Some(filename) = mk_path.file_name() {
                                            let filename_str = filename.to_string_lossy();
                                            // Masterkey files are GUIDs
                                            if Helpers::is_guid(&filename_str) {
                                                files.push(mk_path.to_string_lossy().to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(files)
}

fn find_masterkey_files_target(target: &str) -> Result<Vec<String>> {
    let target_path = Path::new(target);
    let mut files = Vec::new();

    if target_path.is_file() {
        // Single masterkey file
        files.push(target.to_string());
    } else if target_path.is_dir() {
        // Directory of masterkey files
        for entry in fs::read_dir(target_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename_str = filename.to_string_lossy();
                    if Helpers::is_guid(&filename_str) {
                        files.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(files)
}

fn extract_masterkey_hash(mk_file: &str) -> Result<String> {
    // Read masterkey file and extract hash for cracking
    let mk_data = fs::read(mk_file)?;

    if mk_data.len() < 100 {
        return Err(anyhow!("Masterkey file too small"));
    }

    // Parse masterkey file structure (simplified)
    // TODO: Implement full masterkey parsing

    // Extract GUID from filename
    let path = Path::new(mk_file);
    let guid = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    // Placeholder hash format
    Ok(format!("{}:$DPAPImk$...", guid))
}
