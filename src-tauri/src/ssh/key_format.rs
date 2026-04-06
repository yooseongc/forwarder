use std::borrow::Cow;

use aes::cipher::{block_padding::NoPadding, BlockDecryptMut, KeyIvInit};
use anyhow::{bail, Context, Result};
use hmac::Mac;
use russh_keys::{
    ec,
    key::{KeyPair, SignatureHash},
    protocol,
};
use sha2::Digest;

type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type HmacSha1 = hmac::Hmac<sha1::Sha1>;
type HmacSha256 = hmac::Hmac<sha2::Sha256>;

// ── Format detection ──

#[derive(Debug, PartialEq)]
enum KeyFormat {
    OpenSsh,
    RsaPem,
    Pkcs8,
    Pkcs8Enc,
    EcPem,
    PuttyV2,
    PuttyV3,
    PublicKey,
    Unknown,
}

fn detect_format(data: &str) -> KeyFormat {
    let trimmed = data.trim_start();
    if trimmed.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----") {
        KeyFormat::OpenSsh
    } else if trimmed.starts_with("-----BEGIN RSA PRIVATE KEY-----") {
        KeyFormat::RsaPem
    } else if trimmed.starts_with("-----BEGIN ENCRYPTED PRIVATE KEY-----") {
        KeyFormat::Pkcs8Enc
    } else if trimmed.starts_with("-----BEGIN PRIVATE KEY-----") {
        KeyFormat::Pkcs8
    } else if trimmed.starts_with("-----BEGIN EC PRIVATE KEY-----") {
        KeyFormat::EcPem
    } else if trimmed.starts_with("PuTTY-User-Key-File-2:") {
        KeyFormat::PuttyV2
    } else if trimmed.starts_with("PuTTY-User-Key-File-3:") {
        KeyFormat::PuttyV3
    } else if trimmed.starts_with("---- BEGIN SSH2 PUBLIC KEY ----")
        || trimmed.starts_with("ssh-")
        || trimmed.starts_with("ecdsa-")
    {
        KeyFormat::PublicKey
    } else {
        KeyFormat::Unknown
    }
}

// ── Public API ──

pub fn decode_key(data: &str, passphrase: Option<&str>) -> Result<KeyPair> {
    match detect_format(data) {
        KeyFormat::EcPem => decode_ec_pem(data),
        KeyFormat::PuttyV2 => decode_ppk(data, passphrase, 2),
        KeyFormat::PuttyV3 => decode_ppk(data, passphrase, 3),
        KeyFormat::PublicKey => {
            bail!("The selected file is a public key. Please select the private key file instead.")
        }
        KeyFormat::Unknown => {
            bail!("Unrecognized key file format")
        }
        // OpenSSH, RSA PEM, PKCS#8 — handled by russh-keys
        _ => russh_keys::decode_secret_key(data, passphrase)
            .context("Failed to decode private key"),
    }
}

// ── SEC1 EC PEM ──

fn decode_ec_pem(data: &str) -> Result<KeyPair> {
    use sec1::der::Decode;

    let (_, doc) = sec1::pem::decode_vec(data.as_bytes())
        .map_err(|e| anyhow::anyhow!("Failed to decode EC PEM: {e}"))?;
    let ec_key = sec1::EcPrivateKey::from_der(&doc)
        .context("Failed to parse EC private key DER")?;

    let oid = ec_key
        .parameters
        .and_then(|p| p.named_curve())
        .ok_or_else(|| anyhow::anyhow!("EC key missing curve OID"))?;

    const SECP256R1: sec1::der::asn1::ObjectIdentifier =
        sec1::der::asn1::ObjectIdentifier::new_unwrap("1.2.840.10045.3.1.7");
    const SECP384R1: sec1::der::asn1::ObjectIdentifier =
        sec1::der::asn1::ObjectIdentifier::new_unwrap("1.3.132.0.34");

    let (algo, expected_len): (&[u8], usize) = if oid == SECP256R1 {
        (b"ecdsa-sha2-nistp256", 32)
    } else if oid == SECP384R1 {
        (b"ecdsa-sha2-nistp384", 48)
    } else {
        bail!("Unsupported EC curve (OID: {oid}) — only P-256 and P-384 are supported");
    };

    let scalar = strip_leading_zero(ec_key.private_key, expected_len);
    let key = ec::PrivateKey::new_from_secret_scalar(algo, scalar)
        .context("Failed to construct EC key")?;
    Ok(KeyPair::EC { key })
}

/// Strip a leading 0x00 byte that ASN.1 integer encoding may add
/// when the high bit of the scalar is set.
fn strip_leading_zero(data: &[u8], expected_len: usize) -> &[u8] {
    if data.len() == expected_len + 1 && data[0] == 0 {
        &data[1..]
    } else {
        data
    }
}

// ── PuTTY PPK ──

struct PpkData {
    key_type: String,
    encryption: String,
    public_blob: Vec<u8>,
    private_blob: Vec<u8>,
    mac_hex: String,
    comment: String,
    // PPK v3 Argon2 params
    argon2_memory: Option<u32>,
    argon2_passes: Option<u32>,
    argon2_parallelism: Option<u32>,
    argon2_salt: Option<Vec<u8>>,
}

fn parse_ppk_text(data: &str) -> Result<PpkData> {
    let mut key_type = String::new();
    let mut encryption = String::new();
    let mut comment = String::new();
    let mut public_b64 = String::new();
    let mut private_b64 = String::new();
    let mut mac_hex = String::new();
    let mut argon2_memory = None;
    let mut argon2_passes = None;
    let mut argon2_parallelism = None;
    let mut argon2_salt = None;

    #[derive(PartialEq)]
    enum Section {
        Header,
        Public,
        Private,
    }
    let mut section = Section::Header;
    let mut remaining_lines: usize = 0;

    for line in data.lines() {
        if section == Section::Public && remaining_lines > 0 {
            public_b64.push_str(line.trim());
            remaining_lines -= 1;
            if remaining_lines == 0 {
                section = Section::Header;
            }
            continue;
        }
        if section == Section::Private && remaining_lines > 0 {
            private_b64.push_str(line.trim());
            remaining_lines -= 1;
            if remaining_lines == 0 {
                section = Section::Header;
            }
            continue;
        }

        if let Some(val) = line
            .strip_prefix("PuTTY-User-Key-File-2: ")
            .or_else(|| line.strip_prefix("PuTTY-User-Key-File-3: "))
        {
            key_type = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Encryption: ") {
            encryption = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Comment: ") {
            comment = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Public-Lines: ") {
            remaining_lines = val.trim().parse().context("Invalid Public-Lines")?;
            section = Section::Public;
        } else if let Some(val) = line.strip_prefix("Private-Lines: ") {
            remaining_lines = val.trim().parse().context("Invalid Private-Lines")?;
            section = Section::Private;
        } else if let Some(val) = line.strip_prefix("Private-MAC: ") {
            mac_hex = val.trim().to_string();
        } else if let Some(val) = line.strip_prefix("Key-Derivation: ") {
            let kdf = val.trim();
            if kdf != "Argon2id" && kdf != "Argon2i" && kdf != "Argon2d" {
                bail!("Unsupported PPK key derivation: {kdf}");
            }
        } else if let Some(val) = line.strip_prefix("Argon2-Memory: ") {
            argon2_memory = Some(val.trim().parse().context("Invalid Argon2-Memory")?);
        } else if let Some(val) = line.strip_prefix("Argon2-Passes: ") {
            argon2_passes = Some(val.trim().parse().context("Invalid Argon2-Passes")?);
        } else if let Some(val) = line.strip_prefix("Argon2-Parallelism: ") {
            argon2_parallelism = Some(val.trim().parse().context("Invalid Argon2-Parallelism")?);
        } else if let Some(val) = line.strip_prefix("Argon2-Salt: ") {
            argon2_salt = Some(hex::decode(val.trim()).context("Invalid Argon2-Salt hex")?);
        }
    }

    if key_type.is_empty() {
        bail!("Missing key type in PPK file");
    }
    if public_b64.is_empty() || private_b64.is_empty() {
        bail!("Missing public or private key data in PPK file");
    }

    use base64::Engine;
    let public_blob = base64::engine::general_purpose::STANDARD
        .decode(&public_b64)
        .context("Invalid base64 in PPK public key")?;
    let private_blob = base64::engine::general_purpose::STANDARD
        .decode(&private_b64)
        .context("Invalid base64 in PPK private key")?;

    Ok(PpkData {
        key_type,
        encryption,
        public_blob,
        private_blob,
        mac_hex,
        comment,
        argon2_memory,
        argon2_passes,
        argon2_parallelism,
        argon2_salt,
    })
}

fn decode_ppk(data: &str, passphrase: Option<&str>, version: u8) -> Result<KeyPair> {
    let ppk = parse_ppk_text(data)?;
    let passphrase_bytes = passphrase.unwrap_or("").as_bytes();

    let private_blob = if ppk.encryption == "none" {
        let mac_key = derive_mac_key_unencrypted(version);
        verify_mac(&ppk, &ppk.private_blob, version, &mac_key)?;
        ppk.private_blob
    } else if ppk.encryption == "aes256-cbc" {
        if passphrase.is_none() || passphrase == Some("") {
            bail!("Key is encrypted but no passphrase provided");
        }
        decrypt_and_verify_ppk(&ppk, passphrase_bytes, version)?
    } else {
        bail!("Unsupported PPK encryption: {}", ppk.encryption);
    };

    build_keypair(&ppk.key_type, &ppk.public_blob, &private_blob)
}

/// Derive the MAC key for unencrypted PPK files.
fn derive_mac_key_unencrypted(version: u8) -> Vec<u8> {
    if version == 2 {
        // PPK v2: SHA-1("putty-private-key-file-mac-key") — no passphrase for unencrypted
        let mut h = sha1::Sha1::new();
        h.update(b"putty-private-key-file-mac-key");
        h.finalize().to_vec()
    } else {
        // PPK v3 unencrypted: zero-length MAC key
        Vec::new()
    }
}

fn decrypt_and_verify_ppk(
    ppk: &PpkData,
    passphrase: &[u8],
    version: u8,
) -> Result<Vec<u8>> {
    let (enc_key, iv, mac_key) = if version == 2 {
        let (enc_key, iv) = derive_encryption_keys_v2(passphrase);
        let mut h = sha1::Sha1::new();
        h.update(b"putty-private-key-file-mac-key");
        h.update(passphrase);
        let mac_key = h.finalize().to_vec();
        (enc_key, iv, mac_key)
    } else {
        derive_keys_v3(ppk, passphrase)?
    };

    let mut encrypted = ppk.private_blob.clone();
    let decryptor =
        Aes256CbcDec::new_from_slices(&enc_key, &iv).context("Failed to create AES decryptor")?;
    let decrypted = decryptor
        .decrypt_padded_mut::<NoPadding>(&mut encrypted)
        .map_err(|e| anyhow::anyhow!("AES decryption failed: {e}"))?;
    let decrypted = decrypted.to_vec();

    verify_mac(ppk, &decrypted, version, &mac_key)?;

    Ok(decrypted)
}

/// PPK v2 encryption key derivation (SHA-1 based).
/// Returns (32-byte AES key, 16-byte IV).
fn derive_encryption_keys_v2(passphrase: &[u8]) -> (Vec<u8>, Vec<u8>) {
    let mut hash0 = sha1::Sha1::new();
    hash0.update([0u8, 0, 0, 0]);
    hash0.update(passphrase);
    let h0 = hash0.finalize();

    let mut hash1 = sha1::Sha1::new();
    hash1.update([0u8, 0, 0, 1]);
    hash1.update(passphrase);
    let h1 = hash1.finalize();

    let mut key = Vec::with_capacity(40);
    key.extend_from_slice(&h0);
    key.extend_from_slice(&h1);
    key.truncate(32);

    let iv = vec![0u8; 16]; // PPK v2 uses zero IV
    (key, iv)
}

/// PPK v3 key derivation (Argon2id).
/// Returns (32-byte AES key, 16-byte IV, 32-byte MAC key).
fn derive_keys_v3(ppk: &PpkData, passphrase: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let salt = ppk
        .argon2_salt
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Missing Argon2 salt"))?;
    let memory = ppk
        .argon2_memory
        .ok_or_else(|| anyhow::anyhow!("Missing Argon2 memory"))?;
    let passes = ppk
        .argon2_passes
        .ok_or_else(|| anyhow::anyhow!("Missing Argon2 passes"))?;
    let parallelism = ppk
        .argon2_parallelism
        .ok_or_else(|| anyhow::anyhow!("Missing Argon2 parallelism"))?;

    let params = argon2::Params::new(memory, passes, parallelism, Some(80))
        .map_err(|e| anyhow::anyhow!("Invalid Argon2 params: {e}"))?;
    let argon2 = argon2::Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    // Derive 80 bytes: 32 (AES key) + 16 (IV) + 32 (MAC key)
    let mut derived = vec![0u8; 80];
    argon2
        .hash_password_into(passphrase, salt, &mut derived)
        .map_err(|e| anyhow::anyhow!("Argon2 key derivation failed: {e}"))?;

    let enc_key = derived[0..32].to_vec();
    let iv = derived[32..48].to_vec();
    let mac_key = derived[48..80].to_vec();

    Ok((enc_key, iv, mac_key))
}

fn verify_mac(
    ppk: &PpkData,
    decrypted_private: &[u8],
    version: u8,
    mac_key: &[u8],
) -> Result<()> {
    if ppk.mac_hex.is_empty() {
        if ppk.encryption != "none" {
            bail!("Encrypted PPK file is missing MAC — file may be corrupted");
        }
        return Ok(());
    }

    let expected_mac = hex::decode(&ppk.mac_hex).context("Invalid MAC hex")?;

    // MAC covers: key_type || encryption || comment || public_blob || private_blob (decrypted)
    // Each field is length-prefixed (uint32 big-endian).
    let mut mac_data = Vec::new();
    write_ppk_string(&mut mac_data, ppk.key_type.as_bytes());
    write_ppk_string(&mut mac_data, ppk.encryption.as_bytes());
    write_ppk_string(&mut mac_data, ppk.comment.as_bytes());
    write_ppk_string(&mut mac_data, &ppk.public_blob);
    write_ppk_string(&mut mac_data, decrypted_private);

    if version == 2 {
        let mut hmac =
            HmacSha1::new_from_slice(mac_key).context("Failed to create HMAC-SHA1")?;
        hmac.update(&mac_data);
        hmac.verify_slice(&expected_mac)
            .context("MAC verification failed — wrong passphrase or corrupted key")?;
    } else {
        let mut hmac =
            HmacSha256::new_from_slice(mac_key).context("Failed to create HMAC-SHA256")?;
        hmac.update(&mac_data);
        hmac.verify_slice(&expected_mac)
            .context("MAC verification failed — wrong passphrase or corrupted key")?;
    }

    Ok(())
}

fn write_ppk_string(buf: &mut Vec<u8>, data: &[u8]) {
    buf.extend_from_slice(&(data.len() as u32).to_be_bytes());
    buf.extend_from_slice(data);
}

// ── Build KeyPair from PPK blobs ──

fn build_keypair(key_type: &str, public_blob: &[u8], private_blob: &[u8]) -> Result<KeyPair> {
    match key_type {
        "ssh-ed25519" => build_ed25519(private_blob),
        "ssh-rsa" => build_rsa(public_blob, private_blob),
        t if t.starts_with("ecdsa-sha2-") => build_ecdsa(key_type, private_blob),
        _ => bail!("Unsupported PPK key type: {key_type}"),
    }
}

fn build_ed25519(private_blob: &[u8]) -> Result<KeyPair> {
    // Private blob: string private_key (64 bytes: secret || public, or 32 bytes secret)
    let mut reader = SshReader::new(private_blob);
    let secret_bytes = reader.read_string()?;

    let secret = if secret_bytes.len() == 64 {
        &secret_bytes[..32]
    } else if secret_bytes.len() == 32 {
        secret_bytes
    } else {
        bail!(
            "Invalid Ed25519 private key length: {}",
            secret_bytes.len()
        );
    };

    let secret_arr: [u8; 32] = secret
        .try_into()
        .context("Ed25519 secret key must be 32 bytes")?;
    Ok(KeyPair::Ed25519(ed25519_dalek::SigningKey::from_bytes(
        &secret_arr,
    )))
}

fn build_rsa(public_blob: &[u8], private_blob: &[u8]) -> Result<KeyPair> {
    // Public blob: string "ssh-rsa" || mpint e || mpint n
    let mut pub_reader = SshReader::new(public_blob);
    let _key_type = pub_reader.read_string()?;
    let e = pub_reader.read_string()?;
    let n = pub_reader.read_string()?;

    // Private blob: mpint d || mpint p || mpint q || mpint iqmp
    let mut priv_reader = SshReader::new(private_blob);
    let d = priv_reader.read_string()?;
    let p = priv_reader.read_string()?;
    let q = priv_reader.read_string()?;
    let iqmp = priv_reader.read_string()?;

    let rsa_sk = protocol::RsaPrivateKey {
        public_key: protocol::RsaPublicKey {
            public_exponent: Cow::Borrowed(e),
            modulus: Cow::Borrowed(n),
        },
        private_exponent: Cow::Borrowed(d),
        coefficient: Cow::Borrowed(iqmp),
        prime1: Cow::Borrowed(p),
        prime2: Cow::Borrowed(q),
        comment: Cow::Borrowed(b""),
    };

    KeyPair::new_rsa_with_hash(&rsa_sk, None, SignatureHash::SHA2_256)
        .context("Failed to construct RSA key")
}

fn build_ecdsa(key_type: &str, private_blob: &[u8]) -> Result<KeyPair> {
    let expected_len: usize = match key_type {
        "ecdsa-sha2-nistp256" => 32,
        "ecdsa-sha2-nistp384" => 48,
        "ecdsa-sha2-nistp521" => 66,
        _ => bail!("Unsupported ECDSA curve: {key_type}"),
    };

    // Private blob: mpint private_scalar
    let mut reader = SshReader::new(private_blob);
    let scalar = reader.read_string()?;
    let scalar = strip_leading_zero(scalar, expected_len);

    let key = ec::PrivateKey::new_from_secret_scalar(key_type.as_bytes(), scalar)
        .context("Failed to construct ECDSA key")?;
    Ok(KeyPair::EC { key })
}

// ── Simple SSH wire format reader ──

struct SshReader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SshReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn read_string(&mut self) -> Result<&'a [u8]> {
        if self.pos.saturating_add(4) > self.data.len() {
            bail!("Unexpected end of SSH data (reading length)");
        }
        let len = u32::from_be_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap())
            as usize;
        self.pos += 4;
        let remaining = self.data.len() - self.pos;
        if len > remaining {
            bail!(
                "Unexpected end of SSH data (need {} bytes, have {})",
                len,
                remaining
            );
        }
        let result = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(result)
    }
}

// ── Hex decode helper ──

mod hex {
    use anyhow::{bail, Result};

    pub fn decode(s: &str) -> Result<Vec<u8>> {
        if s.len() % 2 != 0 {
            bail!("Odd-length hex string");
        }
        (0..s.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&s[i..i + 2], 16)
                    .map_err(|_| anyhow::anyhow!("Invalid hex character"))
            })
            .collect()
    }
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;

    // ── Format detection ──

    #[test]
    fn detect_openssh() {
        let data = "-----BEGIN OPENSSH PRIVATE KEY-----\nfoo\n-----END OPENSSH PRIVATE KEY-----";
        assert_eq!(detect_format(data), KeyFormat::OpenSsh);
    }

    #[test]
    fn detect_rsa_pem() {
        let data = "-----BEGIN RSA PRIVATE KEY-----\nfoo\n-----END RSA PRIVATE KEY-----";
        assert_eq!(detect_format(data), KeyFormat::RsaPem);
    }

    #[test]
    fn detect_pkcs8() {
        let data = "-----BEGIN PRIVATE KEY-----\nfoo\n-----END PRIVATE KEY-----";
        assert_eq!(detect_format(data), KeyFormat::Pkcs8);
    }

    #[test]
    fn detect_ec_pem() {
        let data = "-----BEGIN EC PRIVATE KEY-----\nfoo\n-----END EC PRIVATE KEY-----";
        assert_eq!(detect_format(data), KeyFormat::EcPem);
    }

    #[test]
    fn detect_ppk_v2() {
        assert_eq!(
            detect_format("PuTTY-User-Key-File-2: ssh-rsa\nfoo"),
            KeyFormat::PuttyV2
        );
    }

    #[test]
    fn detect_ppk_v3() {
        assert_eq!(
            detect_format("PuTTY-User-Key-File-3: ssh-ed25519\nfoo"),
            KeyFormat::PuttyV3
        );
    }

    #[test]
    fn detect_ssh2_public_key() {
        let data = "---- BEGIN SSH2 PUBLIC KEY ----\nfoo\n---- END SSH2 PUBLIC KEY ----";
        assert_eq!(detect_format(data), KeyFormat::PublicKey);
    }

    #[test]
    fn detect_openssh_public_key() {
        assert_eq!(
            detect_format("ssh-rsa AAAA... user@host"),
            KeyFormat::PublicKey
        );
    }

    #[test]
    fn detect_ecdsa_public_key() {
        assert_eq!(
            detect_format("ecdsa-sha2-nistp256 AAAA... user@host"),
            KeyFormat::PublicKey
        );
    }

    #[test]
    fn detect_unknown() {
        assert_eq!(detect_format("some random data"), KeyFormat::Unknown);
    }

    // ── Error cases ──

    #[test]
    fn public_key_returns_error() {
        let result = decode_key("ssh-rsa AAAAB3NzaC1yc2EAAA user@host", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("public key"));
    }

    #[test]
    fn unknown_format_returns_error() {
        let result = decode_key("not a key at all", None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unrecognized"));
    }

    #[test]
    fn ssh2_public_key_returns_error() {
        let data = "\
---- BEGIN SSH2 PUBLIC KEY ----
Comment: \"ecdsa-key-20240624\"
AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBH2MPXJ0o8KV
KDZ1JbP6BmMITt7HBftkkgXAHLtPaeyJwIHsEFfiwn9TDvb4cWRX3Kl+RfcxN+hX
Ec0xVfKxLOo=
---- END SSH2 PUBLIC KEY ----";
        let result = decode_key(data, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("public key"));
    }

    // ── EC PEM decoding ──

    #[test]
    fn decode_ec_pem_p256() {
        // tests/keys/yooseongc-pkey.pem
        let pem = "\
-----BEGIN EC PRIVATE KEY-----
MHgCAQEEIQCHuMmVPOoWsCg41tCJz1vWr91E599Fz7TrZADAi85p7aAKBggqhkjO
PQMBB6FEA0IABH2MPXJ0o8KVKDZ1JbP6BmMITt7HBftkkgXAHLtPaeyJwIHsEFfi
wn9TDvb4cWRX3Kl+RfcxN+hXEc0xVfKxLOo=
-----END EC PRIVATE KEY-----";
        let result = decode_key(pem, None);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    // ── PPK decoding ──

    #[test]
    fn decode_ppk_v3_unencrypted_ecdsa() {
        // tests/keys/yooseongc-pkey.ppk
        let ppk = "\
PuTTY-User-Key-File-3: ecdsa-sha2-nistp256
Encryption: none
Comment: ecdsa-key-20240624
Public-Lines: 3
AAAAE2VjZHNhLXNoYTItbmlzdHAyNTYAAAAIbmlzdHAyNTYAAABBBH2MPXJ0o8KV
KDZ1JbP6BmMITt7HBftkkgXAHLtPaeyJwIHsEFfiwn9TDvb4cWRX3Kl+RfcxN+hX
Ec0xVfKxLOo=
Private-Lines: 1
AAAAIQCHuMmVPOoWsCg41tCJz1vWr91E599Fz7TrZADAi85p7Q==
Private-MAC: a2d58a46bf5c64fd9d960ab1d898cc326e3a0535566454e4b9b2a45333767948";

        let result = decode_key(ppk, None);
        assert!(result.is_ok(), "Failed: {:?}", result.err());
    }

    // ── Helpers ──

    #[test]
    fn hex_decode_basic() {
        assert_eq!(hex::decode("").unwrap(), Vec::<u8>::new());
        assert_eq!(hex::decode("00ff").unwrap(), vec![0x00, 0xff]);
        assert_eq!(hex::decode("a2d5").unwrap(), vec![0xa2, 0xd5]);
        assert!(hex::decode("0").is_err());
        assert!(hex::decode("zz").is_err());
    }

    #[test]
    fn ssh_reader_basic() {
        let data = [0, 0, 0, 5, b'h', b'e', b'l', b'l', b'o', 0, 0, 0, 3, b'b', b'y', b'e'];
        let mut reader = SshReader::new(&data);
        assert_eq!(reader.read_string().unwrap(), b"hello");
        assert_eq!(reader.read_string().unwrap(), b"bye");
    }

    #[test]
    fn strip_leading_zero_removes_padding() {
        assert_eq!(strip_leading_zero(&[0, 1, 2, 3], 3), &[1, 2, 3]);
        assert_eq!(strip_leading_zero(&[1, 2, 3], 3), &[1, 2, 3]);
        // Don't strip if already correct length
        assert_eq!(strip_leading_zero(&[0, 1, 2], 3), &[0, 1, 2]);
    }
}
