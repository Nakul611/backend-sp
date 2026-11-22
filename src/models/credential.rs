use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credential {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub credential_type: String,
    pub ssh_user: String,
    pub fingerprint: Option<String>,
    pub is_encrypted: bool,
    #[serde(default, skip_serializing)]
    pub encrypted_pem: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CreateCredentialRequest {
    pub name: String,
    pub provider: String,
    pub ssh_user: String,
    pub pem_content: String,
}
