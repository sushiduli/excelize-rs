//! Workbook encryption / decryption (`crypt.go`).
//!
//! Provides ECMA-376 agile and standard encryption helpers, CFB packaging
//! and the ISO password hashing algorithm used for sheet/workbook protection.

use std::io::{Cursor, Read, Write};

use aes::cipher::KeyInit;
use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockDecrypt, BlockEncrypt};
use aes::{Aes128, Aes192, Aes256};
use base64::Engine as _;
use quick_xml::de::from_reader as xml_from_reader;
use rand::RngCore;
use serde::Deserialize;

use crate::constants::MAX_FIELD_LENGTH;
use crate::errors::{
    ErrPasswordLengthInvalid, ErrUnknownEncryptMechanism, ErrUnsupportedEncryptMechanism,
    ErrUnsupportedHashAlgorithm, ErrWorkbookFileFormat, Result,
};
use crate::lib_util::count_utf16_string;
use crate::options::Options;

// ------------------------------------------------------------------
// Constants
// ------------------------------------------------------------------

const BLOCK_KEY: &[u8] = &[0x14, 0x6e, 0x0b, 0xe7, 0xab, 0xac, 0xd0, 0xd6];
#[allow(dead_code)]
const DIF_SECT: u32 = 0xFFFFFFFC;
#[allow(dead_code)]
const END_OF_CHAIN: u32 = 0xFFFFFFFE;
#[allow(dead_code)]
const FAT_SECT: u32 = 0xFFFFFFFD;
const ITER_COUNT: usize = 50_000;
const PACKAGE_ENCRYPTION_CHUNK_SIZE: usize = 4096;
const PACKAGE_OFFSET: usize = 8;
const _SHEET_PROTECTION_SPIN_COUNT: i32 = 100_000;
const _WORKBOOK_PROTECTION_SPIN_COUNT: i32 = 100_000;

// ------------------------------------------------------------------
// XML encryption info types
// ------------------------------------------------------------------

/// Top-level encryption info container used by ECMA-376 agile encryption.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "encryption")]
pub struct Encryption {
    #[serde(rename = "keyData", default)]
    pub key_data: KeyData,
    #[serde(rename = "dataIntegrity", default)]
    pub data_integrity: DataIntegrity,
    #[serde(rename = "keyEncryptors", default)]
    pub key_encryptors: KeyEncryptors,
}

/// Cryptographic attributes used to encrypt the data.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "keyData")]
pub struct KeyData {
    #[serde(rename = "@saltSize", default)]
    pub salt_size: i32,
    #[serde(rename = "@blockSize", default)]
    pub block_size: i32,
    #[serde(rename = "@keyBits", default)]
    pub key_bits: i32,
    #[serde(rename = "@hashSize", default)]
    pub hash_size: i32,
    #[serde(rename = "@cipherAlgorithm", default)]
    pub cipher_algorithm: String,
    #[serde(rename = "@cipherChaining", default)]
    pub cipher_chaining: String,
    #[serde(rename = "@hashAlgorithm", default)]
    pub hash_algorithm: String,
    #[serde(rename = "@saltValue", default)]
    pub salt_value: String,
}

/// Encrypted copies of the salt/hash values used for integrity checks.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "dataIntegrity")]
pub struct DataIntegrity {
    #[serde(rename = "@encryptedHmacKey", default)]
    pub encrypted_hmac_key: String,
    #[serde(rename = "@encryptedHmacValue", default)]
    pub encrypted_hmac_value: String,
}

/// Collection of key encryptors.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "keyEncryptors")]
pub struct KeyEncryptors {
    #[serde(rename = "keyEncryptor", default)]
    pub key_encryptor: Vec<KeyEncryptor>,
}

/// A single key encryptor entry.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "keyEncryptor")]
pub struct KeyEncryptor {
    #[serde(rename = "@uri", default)]
    pub uri: String,
    #[serde(rename = "encryptedKey")]
    pub encrypted_key: EncryptedKey,
}

/// Encrypted key used to derive the package encryption key.
#[derive(Debug, Default, Deserialize)]
#[serde(rename = "encryptedKey")]
pub struct EncryptedKey {
    #[serde(rename = "@spinCount", default)]
    pub spin_count: i32,
    #[serde(rename = "@encryptedVerifierHashInput", default)]
    pub encrypted_verifier_hash_input: String,
    #[serde(rename = "@encryptedVerifierHashValue", default)]
    pub encrypted_verifier_hash_value: String,
    #[serde(rename = "@encryptedKeyValue", default)]
    pub encrypted_key_value: String,
    // KeyData fields are duplicated on the encryptedKey element in agile
    // encryption info XML.
    #[serde(rename = "@saltSize", default)]
    pub salt_size: i32,
    #[serde(rename = "@blockSize", default)]
    pub block_size: i32,
    #[serde(rename = "@keyBits", default)]
    pub key_bits: i32,
    #[serde(rename = "@hashSize", default)]
    pub hash_size: i32,
    #[serde(rename = "@cipherAlgorithm", default)]
    pub cipher_algorithm: String,
    #[serde(rename = "@cipherChaining", default)]
    pub cipher_chaining: String,
    #[serde(rename = "@hashAlgorithm", default)]
    pub hash_algorithm: String,
    #[serde(rename = "@saltValue", default)]
    pub salt_value: String,
}

// ------------------------------------------------------------------
// Standard encryption header / verifier
// ------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct StandardEncryptionHeader {
    pub flags: u32,
    pub size_extra: u32,
    pub alg_id: u32,
    pub alg_id_hash: u32,
    pub key_size: u32,
    pub provider_type: u32,
    pub reserved1: u32,
    pub reserved2: u32,
    pub csp_name: String,
}

#[derive(Debug, Default)]
pub struct StandardEncryptionVerifier {
    pub salt_size: u32,
    pub salt: Vec<u8>,
    pub encrypted_verifier: Vec<u8>,
    pub verifier_hash_size: u32,
    pub encrypted_verifier_hash: Vec<u8>,
}

/// Internal encryption state used by the standard encryptor.
#[derive(Debug)]
struct EncryptionInfo {
    block_size: usize,
    salt_size: usize,
    encrypted_key_value: Vec<u8>,
    encrypted_verifier_hash_input: Vec<u8>,
    encrypted_verifier_hash_value: Vec<u8>,
    salt_value: Vec<u8>,
    key_bits: u32,
}

// ------------------------------------------------------------------
// Public API
// ------------------------------------------------------------------

/// Decrypt a CFB-encoded workbook package.
///
/// Supports ECMA-376 agile and standard encryption with MD4, MD5,
/// RIPEMD-160, SHA1, SHA256, SHA384 and SHA512 hash algorithms.
pub fn decrypt(raw: &[u8], opts: &Options) -> Result<Vec<u8>> {
    let mut doc = open_cfb(raw)?;
    let (encryption_info_buf, encrypted_package_buf) = extract_part(&mut doc)?;
    let mechanism = encryption_mechanism(&encryption_info_buf)?;
    match mechanism.as_str() {
        "agile" => agile_decrypt(&encryption_info_buf, &encrypted_package_buf, opts),
        "standard" => standard_decrypt(&encryption_info_buf, &encrypted_package_buf, opts),
        _ => Err(Box::new(ErrUnsupportedEncryptMechanism)),
    }
}

/// Encrypt a workbook package with a password using ECMA-376 standard
/// encryption (AES-128).
pub fn encrypt(raw: &[u8], opts: &Options) -> Result<Vec<u8>> {
    let mut encryptor = EncryptionInfo {
        encrypted_verifier_hash_input: vec![0; 16],
        encrypted_verifier_hash_value: vec![0; 32],
        salt_value: vec![0; 16],
        block_size: 16,
        key_bits: 128,
        salt_size: 16,
        encrypted_key_value: Vec::new(),
    };

    let encryption_info_buffer = encryptor.standard_key_encryption(&opts.password)?;
    let mut encrypted_package: Vec<u8> = Vec::with_capacity(8 + raw.len() + 16);
    encrypted_package.extend_from_slice(&(raw.len() as u64).to_le_bytes());
    encrypted_package.extend_from_slice(&encryptor.encrypt(raw));

    let cursor = Cursor::new(Vec::new());
    let mut compound_file = cfb::CompoundFile::create(cursor)?;
    {
        let mut stream = compound_file.create_stream("/EncryptionInfo")?;
        stream.write_all(&encryption_info_buffer)?;
    }
    {
        let mut stream = compound_file.create_stream("/EncryptedPackage")?;
        stream.write_all(&encrypted_package)?;
    }
    compound_file.flush()?;
    Ok(compound_file.into_inner().into_inner())
}

// ------------------------------------------------------------------
// CFB helpers
// ------------------------------------------------------------------

fn open_cfb(raw: &[u8]) -> Result<cfb::CompoundFile<Cursor<Vec<u8>>>> {
    let cursor = Cursor::new(raw.to_vec());
    match cfb::CompoundFile::open(cursor) {
        Ok(doc) => Ok(doc),
        Err(e) => {
            let msg = e.to_string();
            let re = regex::Regex::new(r"FAT has (\d+) entries, but file has only (\d+) sectors")
                .unwrap();
            if let Some(caps) = re.captures(&msg) {
                let fat_entries: usize = caps[1].parse().unwrap_or(0);
                if fat_entries > 0 {
                    let sector_size = 1usize << (raw.get(0x1E).copied().unwrap_or(9) as usize);
                    let required_len = (fat_entries + 1) * sector_size;
                    if raw.len() < required_len {
                        let mut padded = raw.to_vec();
                        padded.resize(required_len, 0);
                        let cursor = Cursor::new(padded);
                        return Ok(cfb::CompoundFile::open(cursor)?);
                    }
                }
            }
            Err(Box::new(e))
        }
    }
}

fn extract_part(doc: &mut cfb::CompoundFile<Cursor<Vec<u8>>>) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut encryption_info_buf = Vec::new();
    let mut encrypted_package_buf = Vec::new();

    if let Ok(mut stream) = doc.open_stream("/EncryptionInfo") {
        stream.read_to_end(&mut encryption_info_buf)?;
    }
    if let Ok(mut stream) = doc.open_stream("/EncryptedPackage") {
        stream.read_to_end(&mut encrypted_package_buf)?;
    }

    if encryption_info_buf.is_empty() || encrypted_package_buf.is_empty() {
        return Err(Box::new(ErrWorkbookFileFormat));
    }
    Ok((encryption_info_buf, encrypted_package_buf))
}

fn encryption_mechanism(buffer: &[u8]) -> Result<String> {
    if buffer.len() < 4 {
        return Err(Box::new(ErrUnknownEncryptMechanism));
    }
    let version_major = u16::from_le_bytes([buffer[0], buffer[1]]);
    let version_minor = u16::from_le_bytes([buffer[2], buffer[3]]);

    if version_major == 4 && version_minor == 4 {
        return Ok("agile".to_string());
    }
    if (2..=4).contains(&version_major) && version_minor == 2 {
        return Ok("standard".to_string());
    }
    if (version_major == 3 || version_major == 4) && version_minor == 3 {
        return Err(Box::new(ErrUnsupportedEncryptMechanism));
    }
    Err(Box::new(ErrUnsupportedEncryptMechanism))
}

// ------------------------------------------------------------------
// ECMA-376 standard encryption
// ------------------------------------------------------------------

fn standard_decrypt(
    encryption_info_buf: &[u8],
    encrypted_package_buf: &[u8],
    opts: &Options,
) -> Result<Vec<u8>> {
    if encryption_info_buf.len() < 12 {
        return Err(Box::new(ErrWorkbookFileFormat));
    }
    let encryption_header_size = u32::from_le_bytes([
        encryption_info_buf[8],
        encryption_info_buf[9],
        encryption_info_buf[10],
        encryption_info_buf[11],
    ]) as usize;
    if 12 + encryption_header_size > encryption_info_buf.len() {
        return Err(Box::new(ErrWorkbookFileFormat));
    }
    let block = &encryption_info_buf[12..12 + encryption_header_size];
    let header = standard_encryption_header(block)?;
    let verifier_block = &encryption_info_buf[12 + encryption_header_size..];
    let algorithm = if matches!(header.alg_id, 0x0000_660E | 0x0000_660F | 0x0000_6610) {
        "AES"
    } else {
        "RC4"
    };
    let verifier = standard_encryption_verifier(algorithm, verifier_block);
    let secret_key = standard_convert_passwd_to_key(&header, &verifier, opts)?;

    let x = &encrypted_package_buf[8..];
    let decrypted = match secret_key.len() {
        16 => decrypt_aes_ecb(
            Aes128::new_from_slice(&secret_key).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
            })?,
            x,
        ),
        24 => decrypt_aes_ecb(
            Aes192::new_from_slice(&secret_key).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
            })?,
            x,
        ),
        32 => decrypt_aes_ecb(
            Aes256::new_from_slice(&secret_key).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
            })?,
            x,
        ),
        _ => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "unsupported standard encryption key size: {}",
                    header.key_size
                ),
            )));
        }
    };
    Ok(decrypted)
}

fn decrypt_aes_ecb<C: BlockDecrypt>(cipher: C, input: &[u8]) -> Vec<u8> {
    let mut decrypted = vec![0u8; input.len()];
    for (src, dst) in input.chunks(16).zip(decrypted.chunks_mut(16)) {
        let mut block = [0u8; 16];
        block[..src.len()].copy_from_slice(src);
        cipher.decrypt_block(GenericArray::from_mut_slice(&mut block));
        dst.copy_from_slice(&block);
    }
    decrypted
}

fn standard_encryption_header(block: &[u8]) -> Result<StandardEncryptionHeader> {
    if block.len() < 32 {
        return Err(Box::new(ErrWorkbookFileFormat));
    }
    let csp_name = String::from_utf8_lossy(&block[32..]).to_string();
    Ok(StandardEncryptionHeader {
        flags: u32::from_le_bytes([block[0], block[1], block[2], block[3]]),
        size_extra: u32::from_le_bytes([block[4], block[5], block[6], block[7]]),
        alg_id: u32::from_le_bytes([block[8], block[9], block[10], block[11]]),
        alg_id_hash: u32::from_le_bytes([block[12], block[13], block[14], block[15]]),
        key_size: u32::from_le_bytes([block[16], block[17], block[18], block[19]]),
        provider_type: u32::from_le_bytes([block[20], block[21], block[22], block[23]]),
        reserved1: u32::from_le_bytes([block[24], block[25], block[26], block[27]]),
        reserved2: u32::from_le_bytes([block[28], block[29], block[30], block[31]]),
        csp_name,
    })
}

fn standard_encryption_verifier(algorithm: &str, blob: &[u8]) -> StandardEncryptionVerifier {
    let salt_size = u32::from_le_bytes([blob[0], blob[1], blob[2], blob[3]]);
    let salt_end = 4 + salt_size as usize;
    let salt = blob[4..salt_end].to_vec();
    let verifier_end = salt_end + 16;
    let encrypted_verifier = blob[salt_end..verifier_end].to_vec();
    let verifier_hash_size = u32::from_le_bytes([
        blob[verifier_end],
        blob[verifier_end + 1],
        blob[verifier_end + 2],
        blob[verifier_end + 3],
    ]);
    let hash_end = match algorithm {
        "RC4" => verifier_end + 4 + 20,
        _ => verifier_end + 4 + 32,
    };
    let encrypted_verifier_hash = blob[verifier_end + 4..hash_end].to_vec();
    StandardEncryptionVerifier {
        salt_size,
        salt,
        encrypted_verifier,
        verifier_hash_size,
        encrypted_verifier_hash,
    }
}

fn standard_convert_passwd_to_key(
    header: &StandardEncryptionHeader,
    verifier: &StandardEncryptionVerifier,
    opts: &Options,
) -> Result<Vec<u8>> {
    let password_buffer = encode_utf16le(&opts.password);
    let mut key = hashing("sha1", &[&verifier.salt, &password_buffer]);
    for i in 0..ITER_COUNT {
        let iterator = create_uint32_le_buffer(i as i32, 4);
        key = hashing("sha1", &[&iterator, &key]);
    }
    let block = 0i32;
    let h_final = hashing("sha1", &[&key, &create_uint32_le_buffer(block, 4)]);
    let cb_required_key_length = (header.key_size / 8) as usize;
    let cb_hash = 20; // SHA1 digest size

    let buf1 = vec![0x36u8; 64];
    let xored = standard_xor_bytes(&h_final, &buf1[..cb_hash]);
    let mut buf1 = Vec::with_capacity(64);
    buf1.extend_from_slice(&xored);
    buf1.extend_from_slice(&vec![0x36u8; 64 - cb_hash]);
    let x1 = hashing("sha1", &[&buf1]);

    let buf2 = vec![0x5cu8; 64];
    let xored = standard_xor_bytes(&h_final, &buf2[..cb_hash]);
    let mut buf2 = Vec::with_capacity(64);
    buf2.extend_from_slice(&xored);
    buf2.extend_from_slice(&vec![0x5cu8; 64 - cb_hash]);
    let x2 = hashing("sha1", &[&buf2]);

    let mut x3 = x1;
    x3.extend_from_slice(&x2);
    Ok(x3[..cb_required_key_length].to_vec())
}

fn standard_xor_bytes(a: &[u8], b: &[u8]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

impl EncryptionInfo {
    fn encrypt(&self, input: &[u8]) -> Vec<u8> {
        let input_bytes = if input.len() % self.block_size == 0 {
            input.len()
        } else {
            input.len() + self.block_size - (input.len() % self.block_size)
        };
        let cipher = Aes128::new_from_slice(&self.encrypted_key_value).expect("valid AES key");
        let mut output = Vec::with_capacity(input_bytes);
        for i in (0..input_bytes).step_by(self.block_size) {
            let mut chunk = [0u8; 16];
            let end = (i + self.block_size).min(input.len());
            chunk[..end - i].copy_from_slice(&input[i..end]);
            cipher.encrypt_block(GenericArray::from_mut_slice(&mut chunk));
            output.extend_from_slice(&chunk);
        }
        output
    }

    fn standard_key_encryption(&mut self, password: &str) -> Result<Vec<u8>> {
        if count_utf16_string(password) == 0 || count_utf16_string(password) > MAX_FIELD_LENGTH {
            return Err(Box::new(ErrPasswordLengthInvalid));
        }
        let mut stream = ByteWriter::new();
        stream.write_uint16(0x0003);
        stream.write_uint16(0x0002);
        stream.write_uint32(0x24);
        stream.write_uint32(0xA4);
        stream.write_uint32(0x24);
        stream.write_uint32(0x00);
        stream.write_uint32(0x660E);
        stream.write_uint32(0x8004);
        stream.write_uint32(0x80);
        stream.write_uint32(0x18);
        stream.write_uint64(0x00);
        let provider_name = "Microsoft Enhanced RSA and AES Cryptographic Provider (Prototype)";
        stream.write_utf16le(provider_name);
        stream.write_uint16(0x00);
        stream.write_uint32(0x10);

        let key_data_salt_value = random_bytes(self.salt_size)?;
        let verifier_hash_input = random_bytes(16)?;
        self.salt_value = key_data_salt_value;
        self.encrypted_key_value = standard_convert_passwd_to_key(
            &StandardEncryptionHeader {
                key_size: self.key_bits,
                ..Default::default()
            },
            &StandardEncryptionVerifier {
                salt: self.salt_value.clone(),
                ..Default::default()
            },
            &Options {
                password: password.to_string(),
                ..Default::default()
            },
        )?;
        let verifier_hash_input_key = hashing("sha1", &[&verifier_hash_input]);
        self.encrypted_verifier_hash_input = self.encrypt(&verifier_hash_input);
        self.encrypted_verifier_hash_value = self.encrypt(&verifier_hash_input_key);

        stream.write_bytes(&self.salt_value);
        stream.write_bytes(&self.encrypted_verifier_hash_input);
        stream.write_uint32(0x14);
        stream.write_bytes(&self.encrypted_verifier_hash_value);
        Ok(stream.into_inner())
    }
}

// ------------------------------------------------------------------
// ECMA-376 agile encryption
// ------------------------------------------------------------------

fn agile_decrypt(
    encryption_info_buf: &[u8],
    encrypted_package_buf: &[u8],
    opts: &Options,
) -> Result<Vec<u8>> {
    let encryption_info = parse_encryption_info(&encryption_info_buf[8..])?;
    let key = convert_passwd_to_key(&opts.password, BLOCK_KEY, &encryption_info)?;
    let encrypted_key = &encryption_info.key_encryptors.key_encryptor[0].encrypted_key;
    let salt_value = base64_decode(&encrypted_key.salt_value)?;
    let encrypted_key_value = base64_decode(&encrypted_key.encrypted_key_value)?;
    let package_key = decrypt_aes_cbc(&key, &salt_value, &encrypted_key_value)?;
    decrypt_package(&package_key, encrypted_package_buf, &encryption_info)
}

fn convert_passwd_to_key(
    passwd: &str,
    block_key: &[u8],
    encryption: &Encryption,
) -> Result<Vec<u8>> {
    let encrypted_key = &encryption.key_encryptors.key_encryptor[0].encrypted_key;
    let salt_value = base64_decode(&encrypted_key.salt_value)?;
    let mut buffer = Vec::with_capacity(salt_value.len() + passwd.len() * 2);
    buffer.extend_from_slice(&salt_value);
    buffer.extend_from_slice(&encode_utf16le(passwd));

    let mut key = hashing(&encryption.key_data.hash_algorithm, &[&buffer]);
    for i in 0..encrypted_key.spin_count {
        let iterator = create_uint32_le_buffer(i, 4);
        key = hashing(&encryption.key_data.hash_algorithm, &[&iterator, &key]);
    }
    key = hashing(&encryption.key_data.hash_algorithm, &[&key, block_key]);

    let key_bytes = (encrypted_key.key_bits / 8) as usize;
    if key.len() < key_bytes {
        key.extend_from_slice(&vec![0x36u8; 0x36]);
    } else if key.len() > key_bytes {
        key.truncate(key_bytes);
    }
    Ok(key)
}

fn parse_encryption_info(encryption_info: &[u8]) -> Result<Encryption> {
    let xml = String::from_utf8_lossy(encryption_info);
    let stripped = strip_encryption_namespaces(&xml);
    Ok(xml_from_reader(stripped.as_bytes())?)
}

fn strip_encryption_namespaces(xml: &str) -> String {
    let mut s = xml.to_string();
    for ns in [
        "xmlns=\"http://schemas.microsoft.com/office/2006/encryption\"",
        "xmlns:p=\"http://schemas.microsoft.com/office/2006/keyEncryptor/password\"",
    ] {
        s = s.replace(ns, "");
    }
    // Remove the "p:" prefix from element and attribute names without
    // corrupting attribute values such as "http://...".
    let re = regex::Regex::new(r#"(</?|[\s])p:"#).unwrap();
    re.replace_all(&s, "$1").to_string()
}

fn decrypt_aes_cbc(key: &[u8], iv: &[u8], input: &[u8]) -> Result<Vec<u8>> {
    let mut output = input.to_vec();
    let mut iv = iv.to_vec();
    for chunk in output.chunks_mut(16) {
        let encrypted = chunk.to_vec();
        match key.len() {
            16 => {
                let cipher = Aes128::new_from_slice(key).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
                })?;
                cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
            }
            24 => {
                let cipher = Aes192::new_from_slice(key).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
                })?;
                cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
            }
            32 => {
                let cipher = Aes256::new_from_slice(key).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::InvalidData, format!("{e:?}"))
                })?;
                cipher.decrypt_block(GenericArray::from_mut_slice(chunk));
            }
            _ => {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "invalid AES key size",
                )));
            }
        }
        for i in 0..16 {
            chunk[i] ^= iv[i];
        }
        iv = encrypted;
    }
    Ok(output)
}

fn decrypt_package(package_key: &[u8], input: &[u8], encryption: &Encryption) -> Result<Vec<u8>> {
    let encrypted_key = &encryption.key_data;
    if input.len() < PACKAGE_OFFSET {
        return Err(Box::new(ErrWorkbookFileFormat));
    }
    let package_size = u64::from_le_bytes([
        input[0], input[1], input[2], input[3], input[4], input[5], input[6], input[7],
    ]) as usize;
    let input = &input[PACKAGE_OFFSET..];
    let mut output_chunks = Vec::with_capacity(input.len());
    let mut end = 0;
    let mut i = 0;
    while end < input.len() {
        let start = end;
        end = (start + PACKAGE_ENCRYPTION_CHUNK_SIZE).min(input.len());
        let mut input_chunk = input[start..end].to_vec();
        let remainder = input_chunk.len() % encrypted_key.block_size as usize;
        if remainder != 0 {
            input_chunk
                .extend_from_slice(&vec![0u8; encrypted_key.block_size as usize - remainder]);
        }
        let iv = create_iv(i, encryption)?;
        let output_chunk = decrypt_aes_cbc(package_key, &iv, &input_chunk)?;
        output_chunks.extend_from_slice(&output_chunk);
        i += 1;
    }
    if output_chunks.len() > package_size {
        output_chunks.truncate(package_size);
    }
    Ok(output_chunks)
}

fn create_iv(block_key_arg: i32, encryption: &Encryption) -> Result<Vec<u8>> {
    let encrypted_key = &encryption.key_data;
    let block_key_buf = create_uint32_le_buffer(block_key_arg, 4);
    let salt_value = base64_decode(&encrypted_key.salt_value)?;
    let mut iv = hashing(
        &encrypted_key.hash_algorithm,
        &[&salt_value, &block_key_buf],
    );
    if iv.len() < encrypted_key.block_size as usize {
        iv.extend_from_slice(&vec![0x36u8; 0x36]);
    } else if iv.len() > encrypted_key.block_size as usize {
        iv.truncate(encrypted_key.block_size as usize);
    }
    Ok(iv)
}

// ------------------------------------------------------------------
// Hashing & helpers
// ------------------------------------------------------------------

fn hashing(hash_algorithm: &str, buffers: &[&[u8]]) -> Vec<u8> {
    use digest::DynDigest;
    let mut hasher: Box<dyn DynDigest> = match hash_algorithm.to_lowercase().as_str() {
        "md4" => Box::new(md4::Md4::default()),
        "md5" => Box::new(md5::Md5::default()),
        "ripemd-160" => Box::new(ripemd::Ripemd160::default()),
        "sha1" => Box::new(sha1::Sha1::default()),
        "sha256" => Box::new(sha2::Sha256::default()),
        "sha384" => Box::new(sha2::Sha384::default()),
        "sha512" => Box::new(sha2::Sha512::default()),
        _ => return Vec::new(),
    };
    for buf in buffers {
        hasher.update(buf);
    }
    hasher.finalize_reset().to_vec()
}

fn create_uint32_le_buffer(value: i32, buffer_size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; buffer_size];
    let bytes = (value as u32).to_le_bytes();
    buf[..4].copy_from_slice(&bytes);
    buf
}

fn encode_utf16le(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() * 2);
    for unit in s.encode_utf16() {
        out.extend_from_slice(&unit.to_le_bytes());
    }
    out
}

fn base64_decode(s: &str) -> Result<Vec<u8>> {
    Ok(base64::engine::general_purpose::STANDARD.decode(s)?)
}

fn random_bytes(n: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; n];
    rand::thread_rng().fill_bytes(&mut buf);
    Ok(buf)
}

// ------------------------------------------------------------------
// ISO password hashing (sheet / workbook protection)
// ------------------------------------------------------------------

/// Generate an ISO password hash, salt and hash value.
pub fn gen_iso_passwd_hash(
    passwd: &str,
    hash_algorithm: &str,
    salt: &str,
    spin_count: i32,
) -> Result<(String, String)> {
    if count_utf16_string(passwd) < 1 || count_utf16_string(passwd) > MAX_FIELD_LENGTH {
        return Err(Box::new(ErrPasswordLengthInvalid));
    }
    let algorithm_name = match hash_algorithm {
        "MD4" => "md4",
        "MD5" => "md5",
        "SHA-1" => "sha1",
        "SHA-256" => "sha256",
        "SHA-384" => "sha384",
        "SHA-512" => "sha512",
        _ => return Err(Box::new(ErrUnsupportedHashAlgorithm)),
    };
    let mut s = random_bytes(16)?;
    if !salt.is_empty() {
        s = base64_decode(salt)?;
    }
    let mut buffer = Vec::with_capacity(s.len() + passwd.len() * 2);
    buffer.extend_from_slice(&s);
    buffer.extend_from_slice(&encode_utf16le(passwd));
    let mut key = hashing(algorithm_name, &[&buffer]);
    for i in 0..spin_count {
        let iterator = create_uint32_le_buffer(i, 4);
        key = hashing(algorithm_name, &[&key, &iterator]);
    }
    Ok((
        base64::engine::general_purpose::STANDARD.encode(&key),
        base64::engine::general_purpose::STANDARD.encode(&s),
    ))
}

// ------------------------------------------------------------------
// Byte writer used to build the standard encryption info stream
// ------------------------------------------------------------------

struct ByteWriter {
    buf: Vec<u8>,
}

impl ByteWriter {
    fn new() -> Self {
        Self { buf: Vec::new() }
    }

    fn write_bytes(&mut self, value: &[u8]) {
        self.buf.extend_from_slice(value);
    }

    fn write_uint16(&mut self, value: u16) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn write_uint32(&mut self, value: u32) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn write_uint64(&mut self, value: u64) {
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    fn write_utf16le(&mut self, value: &str) {
        self.write_bytes(&encode_utf16le(value));
    }

    fn into_inner(self) -> Vec<u8> {
        self.buf
    }
}
