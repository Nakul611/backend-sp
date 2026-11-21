use crate::models::CloudCatalogService;

pub struct AzureService;

impl AzureService {
    pub fn get_ping_latency_ms() -> u32 {
        44
    }

    pub fn get_catalog_instances() -> Vec<CloudCatalogService> {
        vec![
            CloudCatalogService {
                id: "Standard_D2s_v5".to_string(),
                provider: "azure".to_string(),
                service_name: "Azure Virtual Machines".to_string(),
                category: "compute".to_string(),
                display_name: "Azure Standard D2s v5 (General Purpose)".to_string(),
                vcpu: 2,
                ram_gb: 8.0,
                gpu: None,
                hourly_cost: 0.096,
                monthly_cost: 70.08,
                billing_unit: "per hour".to_string(),
                description: "Balanced CPU-to-memory ratio for enterprise web applications".to_string(),
                is_sandbox: true,
                available_regions: vec!["eastus".to_string(), "westeurope".to_string()],
            },
            CloudCatalogService {
                id: "Standard_NC4as_T4_v3".to_string(),
                provider: "azure".to_string(),
                service_name: "Azure Virtual Machines".to_string(),
                category: "gpu".to_string(),
                display_name: "Azure Standard NC4as T4 (1x NVIDIA T4)".to_string(),
                vcpu: 4,
                ram_gb: 28.0,
                gpu: Some("1x NVIDIA T4 (16GB)".to_string()),
                hourly_cost: 0.526,
                monthly_cost: 383.98,
                billing_unit: "per hour".to_string(),
                description: "GPU accelerated compute for AI training and rendering jobs".to_string(),
                is_sandbox: true,
                available_regions: vec!["eastus".to_string(), "westeurope".to_string()],
            },
        ]
    }
}
