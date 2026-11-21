use crate::models::CloudCatalogService;

pub struct GcpService;

impl GcpService {
    pub fn get_ping_latency_ms() -> u32 {
        28
    }

    pub fn get_catalog_instances() -> Vec<CloudCatalogService> {
        vec![
            CloudCatalogService {
                id: "e2-standard-2".to_string(),
                provider: "gcp".to_string(),
                service_name: "Google Compute Engine".to_string(),
                category: "compute".to_string(),
                display_name: "GCP e2-standard-2 (General Purpose)".to_string(),
                vcpu: 2,
                ram_gb: 8.0,
                gpu: None,
                hourly_cost: 0.067,
                monthly_cost: 48.91,
                billing_unit: "per hour".to_string(),
                description: "Cost-optimized general compute instances with flexible resource sizing".to_string(),
                is_sandbox: true,
                available_regions: vec!["us-central1".to_string(), "europe-west1".to_string()],
            },
            CloudCatalogService {
                id: "g2-standard-4".to_string(),
                provider: "gcp".to_string(),
                service_name: "Google Compute Engine".to_string(),
                category: "gpu".to_string(),
                display_name: "GCP g2-standard-4 (1x NVIDIA L4 GPU)".to_string(),
                vcpu: 4,
                ram_gb: 16.0,
                gpu: Some("1x NVIDIA L4 (24GB)".to_string()),
                hourly_cost: 0.702,
                monthly_cost: 512.46,
                billing_unit: "per hour".to_string(),
                description: "Next-gen inference GPU optimized for generative AI workloads".to_string(),
                is_sandbox: true,
                available_regions: vec!["us-central1".to_string(), "europe-west4".to_string()],
            },
        ]
    }
}
