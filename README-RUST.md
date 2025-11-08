# SharpDPAPI-RS (Rust Port)

This is a Rust port of [SharpDPAPI](https://github.com/GhostPack/SharpDPAPI), a C# tool for DPAPI (Data Protection API) operations originally developed by [@harmj0y](https://twitter.com/harmj0y).

## About This Port

SharpDPAPI-RS is a work-in-progress Rust implementation that aims to provide the same DPAPI functionality as the original C# version, with the benefits of Rust's memory safety and cross-platform capabilities.

### Current Status

This is an **initial port** with the following implementation status:

#### ✅ Completed
- Basic project structure and build system
- Command-line argument parsing
- Core domain logic (Info, ArgumentParser, CommandCollection)
- Helper utilities (hex conversion, GUID parsing, etc.)
- Cryptographic primitives (AES, 3DES, SHA1, SHA512, HMAC, PBKDF2)
- DPAPI blob parsing and structure definitions
- Module organization matching original C# structure

#### 🚧 Partial Implementation
- DPAPI blob decryption (basic implementation)
- Key derivation (Microsoft's custom DPAPI algorithm)
- Certificate operations (structure only)

#### ❌ Not Yet Implemented
- Windows-specific API calls (CryptUnprotectData, LSA secrets, etc.)
- Token impersonation for SYSTEM elevation
- Complete command implementations:
  - masterkeys
  - credentials
  - vaults
  - certificates
  - rdg
  - keepass
  - triage
  - backupkey
  - search
  - sccm
  - and others
- Full certificate private key decryption
- Remote server operations
- RPC calls for backup keys
- WMI operations for SCCM

## Building

### Prerequisites

- Rust 1.70 or later
- Windows SDK (for Windows-specific features)
- Cargo

### Build Instructions

```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test
```

The compiled binary will be located at:
- Debug: `target/debug/sharp-dpapi`
- Release: `target/release/sharp-dpapi`

## Usage

```bash
sharp-dpapi [command] [options]
```

### Available Commands

*Note: Most commands are not yet fully implemented in this Rust port*

- `masterkeys` - Triage user masterkey files
- `credentials` - Triage user Credential files
- `vaults` - Triage user Vault files
- `rdg` - Triage RDCMan.settings files
- `certificates` - Triage certificate private keys
- `keepass` - Triage KeePass ProtectedUserKey.bin files
- `triage` - Run all user triage commands
- `machinemasterkeys` - Triage machine masterkey files
- `machinecredentials` - Triage machine Credential files
- `machinevaults` - Triage machine Vaults
- `machinetriage` - Run all machine triage commands
- `backupkey` - Retrieve domain DPAPI backup key
- `search` - Search for DPAPI blobs
- `blob` - Describe/decrypt a DPAPI blob
- `ps` - Describe/decrypt a PSCredential .xml file
- `sccm` - Triage SCCM NAA credentials

## Differences from C# Version

### Architectural Differences

1. **Error Handling**: Uses Rust's `Result<T, E>` instead of exceptions
2. **Memory Safety**: Leverages Rust's ownership system for safer memory handling
3. **Windows API**: Windows API calls require different bindings (using `windows-rs` crate)
4. **Cryptography**: Uses pure Rust crypto libraries instead of .NET's crypto APIs

### Limitations

1. **Windows-Only Features**: Many features require Windows-specific APIs:
   - CryptUnprotectData
   - LSA secret retrieval
   - Token impersonation
   - WMI operations

2. **Platform Support**: While the core crypto and parsing logic could work cross-platform, DPAPI itself is Windows-specific

3. **Third-Party Dependencies**: Uses well-audited Rust crypto crates:
   - `aes`, `des` for encryption
   - `sha1`, `sha2` for hashing
   - `hmac` for HMAC operations
   - `pbkdf2` for key derivation

## Development Roadmap

### Phase 1: Core Functionality ✅ (Current)
- [x] Project structure
- [x] Basic crypto operations
- [x] DPAPI blob parsing
- [x] Helper utilities

### Phase 2: Windows API Integration 🚧
- [ ] Implement CryptUnprotectData wrapper
- [ ] LSA secret retrieval
- [ ] Token impersonation
- [ ] Registry operations

### Phase 3: Command Implementation 📋
- [ ] Masterkeys command
- [ ] Credentials command
- [ ] Vaults command
- [ ] Certificates command
- [ ] Triage commands

### Phase 4: Advanced Features 📋
- [ ] Remote server operations
- [ ] RPC backup key retrieval
- [ ] SCCM integration
- [ ] Search functionality

## Contributing

This is a port of an existing tool. Contributions are welcome, especially:

- Windows API integrations
- Command implementations
- Test coverage
- Documentation improvements
- Cross-compilation support

## Security Notice

⚠️ **This tool is designed for authorized security testing and research only.**

- DPAPI credential extraction requires appropriate authorization
- Backup key retrieval requires domain administrator privileges
- Machine triage requires SYSTEM/administrative privileges
- Use only in environments where you have explicit permission

## License

This Rust port maintains the same BSD 3-Clause license as the original SharpDPAPI project.

## Credits

- **Original SharpDPAPI**: [@harmj0y](https://twitter.com/harmj0y) and the GhostPack team
- **DPAPI Research**: [@gentilkiwi](https://twitter.com/gentilkiwi) (Mimikatz)
- **Rust Port**: Created as part of the C#-to-Rust migration initiative

## References

- [Original SharpDPAPI](https://github.com/GhostPack/SharpDPAPI)
- [Mimikatz](https://github.com/gentilkiwi/mimikatz)
- [Operational Guidance for Offensive DPAPI Abuse](https://blog.harmj0y.net/redteaming/operational-guidance-for-offensive-user-dpapi-abuse/)
- [DPAPI Documentation](https://docs.microsoft.com/en-us/windows/win32/api/dpapi/)

## Version

Current version: **1.20.0-rust** (Initial Port)

Original C# version: **1.12.0**

---

**Note**: This is an initial port and many features are not yet fully functional. See the [Current Status](#current-status) section for details on what's implemented.
