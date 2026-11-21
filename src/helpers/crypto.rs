/// Cryptographic, masking, and security utility helpers

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
