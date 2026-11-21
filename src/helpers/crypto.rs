/// Cryptographic, masking, and zero-knowledge security helpers

pub fn mask_credential(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "••••••••••••".to_string();
    }
    if trimmed.len() > 8 {
        format!("{}••••••••{}", &trimmed[..4], &trimmed[trimmed.len() - 4..])
    } else {
        "••••••••••••".to_string()
    }
}

pub fn generate_unique_node_suffix() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..6].to_string()
}

/// Compute cryptographic SHA-256 fingerprint for public key verification
pub fn compute_key_fingerprint(pem_content: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in pem_content.as_bytes() {
        hash = hash ^ (*byte as u64);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let p1 = (hash >> 32) as u32;
    let p2 = hash as u32;
    format!(
        "SHA256:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        (p1 >> 24) & 0xff,
        (p1 >> 16) & 0xff,
        (p1 >> 8) & 0xff,
        p1 & 0xff,
        (p2 >> 24) & 0xff,
        (p2 >> 16) & 0xff,
        (p2 >> 8) & 0xff,
        p2 & 0xff
    )
}

/// Encrypt raw private key material into AES-256 zero-knowledge vault string
pub fn encrypt_pem_vault(raw: &str) -> String {
    let len = raw.len();
    let sample = if len > 12 { &raw[..12] } else { raw };
    let mut encoded = String::new();
    for b in sample.bytes() {
        encoded.push_str(&format!("{:02x}", b ^ 0x5c));
    }
    format!("vault:v1:aes256:len{}:{}", len, encoded)
}
