pub mod aws;
pub mod azure;
pub mod gcp;

pub use aws::AwsService;
pub use azure::AzureService;
pub use gcp::GcpService;

use crate::models::CloudCatalogService;

pub struct CloudService;

impl CloudService {
    pub fn get_full_catalog() -> Vec<CloudCatalogService> {
        let mut catalog = Vec::new();
        catalog.extend(AwsService::get_catalog_instances());
        catalog.extend(AzureService::get_catalog_instances());
        catalog.extend(GcpService::get_catalog_instances());
        catalog
    }

    pub fn get_provider_ping(provider: &str) -> u32 {
        match provider.to_lowercase().as_str() {
            "aws" => AwsService::get_ping_latency_ms(),
            "azure" => AzureService::get_ping_latency_ms(),
            "gcp" => GcpService::get_ping_latency_ms(),
            _ => 30,
        }
    }
}
