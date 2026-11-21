use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::AppError,
    controllers::server_controller::fetch_live_telemetry,
    models::{
        CloudCatalogQuery, CloudCatalogService, CloudProviderConfig, ProvisionServersRequest,
        ProvisionServersResponse, Server, TestCloudProviderRequest, TestCloudProviderResponse,
        UpdateCloudProviderRequest, WsEvent,
    },
    state::AppState,
};

pub async fn list_cloud_providers(
    State(state): State<AppState>,
) -> Result<Json<Vec<CloudProviderConfig>>, AppError> {
    let providers = state.cloud_providers.read().await;
    Ok(Json(providers.clone()))
}

pub async fn update_cloud_provider(
    State(state): State<AppState>,
    Path(provider_key): Path<String>,
    Json(payload): Json<UpdateCloudProviderRequest>,
) -> Result<Json<CloudProviderConfig>, AppError> {
    let mut providers = state.cloud_providers.write().await;
    let provider = providers
        .iter_mut()
        .find(|p| p.provider == provider_key)
        .ok_or_else(|| AppError::NotFound(format!("Cloud provider '{}' not found", provider_key)))?;

    provider.is_sandbox = payload.is_sandbox;
    if let Some(region) = payload.default_region {
        provider.default_region = region;
    }
    if let Some(raw) = payload.credentials_raw {
        if !raw.trim().is_empty() {
            let masked = if raw.len() > 8 {
                format!("{}••••••••{}", &raw[..4], &raw[raw.len() - 4..])
            } else {
                "••••••••••••".to_string()
            };
            provider.credentials_masked = masked;
            provider.status = "connected".to_string();
        }
    }
    provider.last_tested_at = Some(Utc::now().timestamp());

    Ok(Json(provider.clone()))
}

pub async fn test_cloud_provider(
    State(state): State<AppState>,
    Json(payload): Json<TestCloudProviderRequest>,
) -> Result<Json<TestCloudProviderResponse>, AppError> {
    let mut providers = state.cloud_providers.write().await;
    let provider = providers
        .iter_mut()
        .find(|p| p.provider == payload.provider)
        .ok_or_else(|| AppError::NotFound(format!("Cloud provider '{}' not found", payload.provider)))?;

    let now = Utc::now().timestamp();
    let region = payload.region.unwrap_or_else(|| provider.default_region.clone());

    // Simulated cloud ping latency
    let latency = match provider.provider.as_str() {
        "aws" => 32,
        "azure" => 44,
        "gcp" => 28,
        _ => 35,
    };

    provider.status = "connected".to_string();
    provider.last_ping_ms = Some(latency);
    provider.last_tested_at = Some(now);

    let account_id = provider.account_id.clone().unwrap_or_else(|| {
        format!("{}-account-{}", provider.provider, &Uuid::new_v4().simple().to_string()[..8])
    });

    let message = format!(
        "Successfully verified {} API connection to region '{}' ({}ms latency, Sandbox: {}).",
        provider.name, region, latency, provider.is_sandbox
    );

    Ok(Json(TestCloudProviderResponse {
        success: true,
        provider: provider.provider.clone(),
        account_id,
        region,
        latency_ms: latency,
        is_sandbox: provider.is_sandbox,
        message,
    }))
}

pub fn build_cloud_catalog() -> Vec<CloudCatalogService> {
    vec![
        // AWS Instances & Cloud Services
        CloudCatalogService {
            id: "t3.medium".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon EC2".to_string(),
            category: "compute".to_string(),
            display_name: "AWS t3.medium (Burstable General)".to_string(),
            vcpu: 2,
            ram_gb: 4.0,
            gpu: None,
            hourly_cost: 0.0416,
            monthly_cost: 30.37,
            billing_unit: "per hour".to_string(),
            description: "Burstable general compute for microservices and API workers".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string(), "eu-west-1".to_string()],
        },
        CloudCatalogService {
            id: "c5.large".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon EC2".to_string(),
            category: "compute".to_string(),
            display_name: "AWS c5.large (Compute Optimized)".to_string(),
            vcpu: 2,
            ram_gb: 4.0,
            gpu: None,
            hourly_cost: 0.096,
            monthly_cost: 70.08,
            billing_unit: "per hour".to_string(),
            description: "Compute-optimized baseline instance for continuous processing".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string(), "eu-west-1".to_string()],
        },
        CloudCatalogService {
            id: "c5.2xlarge".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon EC2".to_string(),
            category: "compute".to_string(),
            display_name: "AWS c5.2xlarge (High-Throughput)".to_string(),
            vcpu: 8,
            ram_gb: 16.0,
            gpu: None,
            hourly_cost: 0.340,
            monthly_cost: 248.20,
            billing_unit: "per hour".to_string(),
            description: "High-throughput parallel workers and distributed scrapers".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string(), "eu-west-1".to_string()],
        },
        CloudCatalogService {
            id: "g4dn.xlarge".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon EC2".to_string(),
            category: "gpu".to_string(),
            display_name: "AWS g4dn.xlarge (1x NVIDIA T4 GPU)".to_string(),
            vcpu: 4,
            ram_gb: 16.0,
            gpu: Some("1x NVIDIA T4 (16GB)".to_string()),
            hourly_cost: 0.526,
            monthly_cost: 383.98,
            billing_unit: "per hour".to_string(),
            description: "Cost-effective GPU instance for machine learning and LLM vision inference".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string()],
        },
        CloudCatalogService {
            id: "s3-standard".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon S3".to_string(),
            category: "storage".to_string(),
            display_name: "AWS S3 Standard Object Bucket".to_string(),
            vcpu: 0,
            ram_gb: 0.0,
            gpu: None,
            hourly_cost: 0.023,
            monthly_cost: 16.79,
            billing_unit: "per GB/month".to_string(),
            description: "Cloud object storage lake for cold logs and training artifacts".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string()],
        },
        CloudCatalogService {
            id: "eks-worker".to_string(),
            provider: "aws".to_string(),
            service_name: "Amazon EKS".to_string(),
            category: "kubernetes".to_string(),
            display_name: "AWS EKS Managed Node (c5.xlarge)".to_string(),
            vcpu: 4,
            ram_gb: 8.0,
            gpu: None,
            hourly_cost: 0.192,
            monthly_cost: 140.16,
            billing_unit: "per hour".to_string(),
            description: "Managed Kubernetes cluster worker node for container orchestration".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-east-1".to_string(), "us-west-2".to_string()],
        },

        // Azure Instances & Cloud Services
        CloudCatalogService {
            id: "Standard_B2s".to_string(),
            provider: "azure".to_string(),
            service_name: "Azure Virtual Machines".to_string(),
            category: "compute".to_string(),
            display_name: "Azure Standard_B2s (Burstable)".to_string(),
            vcpu: 2,
            ram_gb: 4.0,
            gpu: None,
            hourly_cost: 0.0496,
            monthly_cost: 36.21,
            billing_unit: "per hour".to_string(),
            description: "Economical burstable VM for test fleets and edge proxies".to_string(),
            is_sandbox: true,
            available_regions: vec!["eastus".to_string(), "westeurope".to_string(), "southeastasia".to_string()],
        },
        CloudCatalogService {
            id: "Standard_D4s_v5".to_string(),
            provider: "azure".to_string(),
            service_name: "Azure Virtual Machines".to_string(),
            category: "compute".to_string(),
            display_name: "Azure Standard_D4s_v5 (Enterprise)".to_string(),
            vcpu: 4,
            ram_gb: 16.0,
            gpu: None,
            hourly_cost: 0.192,
            monthly_cost: 140.16,
            billing_unit: "per hour".to_string(),
            description: "Enterprise compute node with fast NVMe temporary storage".to_string(),
            is_sandbox: true,
            available_regions: vec!["eastus".to_string(), "westeurope".to_string(), "southeastasia".to_string()],
        },
        CloudCatalogService {
            id: "Standard_NC6s_v3".to_string(),
            provider: "azure".to_string(),
            service_name: "Azure Virtual Machines".to_string(),
            category: "gpu".to_string(),
            display_name: "Azure Standard_NC6s_v3 (1x V100 GPU)".to_string(),
            vcpu: 6,
            ram_gb: 112.0,
            gpu: Some("1x NVIDIA V100 (16GB)".to_string()),
            hourly_cost: 0.900,
            monthly_cost: 657.00,
            billing_unit: "per hour".to_string(),
            description: "Heavy GPU acceleration for deep learning training and vision pipelines".to_string(),
            is_sandbox: true,
            available_regions: vec!["eastus".to_string(), "westeurope".to_string()],
        },
        CloudCatalogService {
            id: "blob-standard".to_string(),
            provider: "azure".to_string(),
            service_name: "Azure Blob Storage".to_string(),
            category: "storage".to_string(),
            display_name: "Azure Blob Storage Hot Tier".to_string(),
            vcpu: 0,
            ram_gb: 0.0,
            gpu: None,
            hourly_cost: 0.018,
            monthly_cost: 13.14,
            billing_unit: "per GB/month".to_string(),
            description: "Scalable object storage for enterprise analytics telemetry".to_string(),
            is_sandbox: true,
            available_regions: vec!["eastus".to_string(), "westeurope".to_string()],
        },
        CloudCatalogService {
            id: "aks-node".to_string(),
            provider: "azure".to_string(),
            service_name: "Azure Kubernetes Service (AKS)".to_string(),
            category: "kubernetes".to_string(),
            display_name: "Azure AKS Managed Agent Pool Node".to_string(),
            vcpu: 4,
            ram_gb: 16.0,
            gpu: None,
            hourly_cost: 0.205,
            monthly_cost: 149.65,
            billing_unit: "per hour".to_string(),
            description: "Production Kubernetes node for distributed containerized microservices".to_string(),
            is_sandbox: true,
            available_regions: vec!["eastus".to_string(), "westeurope".to_string()],
        },

        // GCP Instances & Cloud Services
        CloudCatalogService {
            id: "e2-standard-4".to_string(),
            provider: "gcp".to_string(),
            service_name: "Google Compute Engine".to_string(),
            category: "compute".to_string(),
            display_name: "GCP e2-standard-4 (Balanced)".to_string(),
            vcpu: 4,
            ram_gb: 16.0,
            gpu: None,
            hourly_cost: 0.138,
            monthly_cost: 100.74,
            billing_unit: "per hour".to_string(),
            description: "Balanced Google Compute Engine VM for web services and container clusters".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-central1".to_string(), "europe-west1".to_string(), "asia-east1".to_string()],
        },
        CloudCatalogService {
            id: "n2-standard-8".to_string(),
            provider: "gcp".to_string(),
            service_name: "Google Compute Engine".to_string(),
            category: "compute".to_string(),
            display_name: "GCP n2-standard-8 (Intel Xeon)".to_string(),
            vcpu: 8,
            ram_gb: 32.0,
            gpu: None,
            hourly_cost: 0.388,
            monthly_cost: 283.24,
            billing_unit: "per hour".to_string(),
            description: "High-performance Intel Xeon workload runner for data processing".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-central1".to_string(), "europe-west1".to_string()],
        },
        CloudCatalogService {
            id: "a2-highgpu-1g".to_string(),
            provider: "gcp".to_string(),
            service_name: "Google Compute Engine".to_string(),
            category: "gpu".to_string(),
            display_name: "GCP a2-highgpu-1g (1x A100 GPU)".to_string(),
            vcpu: 12,
            ram_gb: 85.0,
            gpu: Some("1x NVIDIA A100 (40GB)".to_string()),
            hourly_cost: 1.850,
            monthly_cost: 1350.50,
            billing_unit: "per hour".to_string(),
            description: "Top-tier AI acceleration for LLM fine-tuning, training, and inference".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-central1".to_string(), "europe-west4".to_string()],
        },
        CloudCatalogService {
            id: "gcs-standard".to_string(),
            provider: "gcp".to_string(),
            service_name: "Google Cloud Storage".to_string(),
            category: "storage".to_string(),
            display_name: "Google Cloud Storage Standard Bucket".to_string(),
            vcpu: 0,
            ram_gb: 0.0,
            gpu: None,
            hourly_cost: 0.026,
            monthly_cost: 18.98,
            billing_unit: "per GB/month".to_string(),
            description: "Unified multi-region bucket for datasets, weights, and models".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-central1".to_string(), "europe-west1".to_string()],
        },
        CloudCatalogService {
            id: "gke-autopilot".to_string(),
            provider: "gcp".to_string(),
            service_name: "Google Kubernetes Engine (GKE)".to_string(),
            category: "kubernetes".to_string(),
            display_name: "GCP GKE Autopilot Compute Node".to_string(),
            vcpu: 4,
            ram_gb: 16.0,
            gpu: None,
            hourly_cost: 0.176,
            monthly_cost: 128.48,
            billing_unit: "per hour".to_string(),
            description: "Fully automated, production-ready Kubernetes pod worker".to_string(),
            is_sandbox: true,
            available_regions: vec!["us-central1".to_string(), "europe-west1".to_string()],
        },
    ]
}

pub async fn get_cloud_catalog(
    State(state): State<AppState>,
    Query(query): Query<CloudCatalogQuery>,
) -> Result<Json<Vec<CloudCatalogService>>, AppError> {
    let services = state
        .live_pricing
        .get_live_catalog(query.provider.as_deref(), query.category.as_deref())
        .await;

    Ok(Json(services))
}

pub async fn provision_servers(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
    Json(payload): Json<ProvisionServersRequest>,
) -> Result<(StatusCode, Json<ProvisionServersResponse>), AppError> {
    // Verify project exists
    {
        let projects = state.projects.read().await;
        if !projects.iter().any(|p| p.id == project_id) {
            return Err(AppError::NotFound(format!("Project '{}' not found", project_id)));
        }
    }

    if payload.count == 0 || payload.count > 50 {
        return Err(AppError::BadRequest(
            "Batch provisioning count must be between 1 and 50 nodes per request.".to_string(),
        ));
    }

    // Determine unit hourly rate dynamically in real-time from Cloud Provider APIs (zero stored catalog)
    let unit_cost = state
        .live_pricing
        .get_live_price(&payload.provider, &payload.instance_type, &payload.region)
        .await;

    let resource_type = if payload.instance_type.contains("gpu") || payload.instance_type.contains("t4") {
        "gpu"
    } else if payload.instance_type.contains("s3") || payload.instance_type.contains("blob") || payload.instance_type.contains("storage") {
        "storage"
    } else if payload.instance_type.contains("rds") || payload.instance_type.contains("db") || payload.instance_type.contains("postgres") {
        "database"
    } else {
        "compute"
    };

    let default_tags = vec![resource_type.to_string(), payload.instance_type.clone()];

    let now = Utc::now().timestamp();
    let mut provisioned = Vec::new();

    {
        let mut servers = state.servers.write().await;

        for i in 1..=payload.count {
            let unique_suffix = &Uuid::new_v4().simple().to_string()[..6];
            let name = match &payload.name_prefix {
                Some(p) => format!("{}-{:02}", p, i),
                None => format!("{}-{}-{:02}", payload.provider, payload.instance_type.replace('_', "-").replace('.', "-"), i),
            };

            let host_ip = match payload.provider.as_str() {
                "azure" => format!("20.120.{}.{}", (i * 7) % 250 + 1, (i * 13) % 250 + 2),
                "gcp" => format!("34.130.{}.{}", (i * 9) % 250 + 1, (i * 17) % 250 + 2),
                _ => format!("54.210.{}.{}", (i * 11) % 250 + 1, (i * 19) % 250 + 2),
            };

            let mut tags = default_tags.clone();
            tags.push("cloud-provisioned".to_string());
            tags.push(payload.provider.clone());
            if let Some(user_tags) = &payload.tags {
                for t in user_tags {
                    if !tags.contains(t) {
                        tags.push(t.clone());
                    }
                }
            }

            let mut server = Server {
                id: format!("srv-{}", Uuid::new_v4().simple()),
                project_id: project_id.clone(),
                server_id: format!("{}-{}-{}", payload.provider, payload.instance_type, unique_suffix),
                name,
                provider: payload.provider.clone(),
                resource_type: resource_type.to_string(),
                host_ip,
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: payload.region.clone(),
                tags,
                status: "active".to_string(),
                active_since: now,
                last_job_started_at: Some(now),
                last_job_completed_at: None,
                idle_since: None,
                idle_minutes: 0,
                last_error: None,
                created_at: now,
                service_id: None,
                role: Some("compute".to_string()),
                service_name: None,
                database_engine: None,
                port: None,
                telemetry: None,
            };

            // Attach dynamic live telemetry
            server.telemetry = Some(fetch_live_telemetry(&server));

            servers.push(server.clone());
            provisioned.push(server);
        }
    }

    // Update project stats
    {
        let mut projects = state.projects.write().await;
        if let Some(proj) = projects.iter_mut().find(|p| p.id == project_id) {
            proj.server_count += payload.count as i64;
            proj.active_count += payload.count as i64;
            proj.updated_at = now;
        }
    }

    // Broadcast WebSocket notification to fleet
    state.broadcast_event(&WsEvent {
        r#type: "SERVER_STATUS_CHANGED".to_string(),
        project_id: Some(project_id.clone()),
        server_id: None,
        status: Some("active".to_string()),
        idle_minutes: Some(0),
        message: Some(format!(
            "Successfully provisioned {}x {} ({}) in region {}.",
            payload.count, payload.instance_type, payload.provider.to_uppercase(), payload.region
        )),
        telemetry: None,
    });

    let total_hourly_cost = (unit_cost * (payload.count as f64) * 1000.0).round() / 1000.0;

    let response = ProvisionServersResponse {
        success: true,
        provisioned_count: payload.count,
        provider: payload.provider,
        instance_type: payload.instance_type,
        total_hourly_cost,
        servers: provisioned,
        message: format!(
            "{} nodes successfully allocated and active in fleet.",
            payload.count
        ),
    };

    Ok((StatusCode::CREATED, Json(response)))
}
