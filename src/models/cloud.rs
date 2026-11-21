use serde::{Deserialize, Serialize};
use super::server::Server;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudProviderConfig {
    pub id: String,
    pub provider: String,          // "aws", "azure", "gcp"
    pub name: String,              // "Amazon Web Services", "Microsoft Azure", "Google Cloud Platform"
    pub status: String,            // "connected", "disconnected"
    pub is_sandbox: bool,          // true if in sandbox test mode
    pub account_id: Option<String>,
    pub default_region: String,
    pub credentials_masked: String,
    pub last_ping_ms: Option<u32>,
    pub last_tested_at: Option<i64>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateCloudProviderRequest {
    pub provider: String,
    pub is_sandbox: bool,
    pub default_region: Option<String>,
    pub credentials_raw: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TestCloudProviderRequest {
    pub provider: String,
    pub region: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TestCloudProviderResponse {
    pub success: bool,
    pub provider: String,
    pub account_id: String,
    pub region: String,
    pub latency_ms: u32,
    pub is_sandbox: bool,
    pub message: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProvisionServersRequest {
    pub provider: String,          // "aws", "azure", "gcp"
    pub service_type: String,      // "compute", "gpu", "storage", "database"
    pub instance_type: String,     // "c5.large", "c5.2xlarge", "Standard_D4s_v5", etc.
    pub count: usize,              // e.g. 5
    pub region: String,            // "us-east-1"
    pub name_prefix: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ProvisionServersResponse {
    pub success: bool,
    pub provisioned_count: usize,
    pub provider: String,
    pub instance_type: String,
    pub total_hourly_cost: f64,
    pub servers: Vec<Server>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CloudCatalogService {
    pub id: String,
    pub provider: String,              // "aws", "azure", "gcp"
    pub service_name: String,          // "AWS EC2 Compute", "Amazon S3", "Azure Virtual Machines", etc.
    pub category: String,              // "compute", "gpu", "storage", "database", "kubernetes"
    pub display_name: String,          // "AWS c5.large (Compute Optimized)"
    pub vcpu: i32,
    pub ram_gb: f64,
    pub gpu: Option<String>,
    pub hourly_cost: f64,
    pub monthly_cost: f64,
    pub billing_unit: String,          // "per hour"
    pub description: String,
    pub is_sandbox: bool,
    pub available_regions: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CloudCatalogQuery {
    pub provider: Option<String>,
    pub category: Option<String>,
}
