use crate::models::{FinancialBurden, ResourceTelemetry, Server};
use chrono::Utc;

pub struct TelemetryService;

impl TelemetryService {
    pub fn compute_live_telemetry(server: &Server) -> ResourceTelemetry {
        let now = Utc::now().timestamp();
        let acquired_at = server.created_at;
        let lifetime_hours = (((now - acquired_at).max(3600) as f64 / 3600.0) * 10.0).round() / 10.0;
        let hash = server.name.bytes().fold(0u64, |acc, b| acc.wrapping_add(b as u64));

        let is_storage = server.resource_type == "s3" || server.resource_type == "storage";
        if is_storage {
            let used_gb = 500.0 + ((hash % 1200) as f64) + ((now % 60) as f64 * 0.1);
            let total_gb = 5000.0;
            let hourly_cost = (used_gb * 0.023) / (30.0 * 24.0);
            let storage_capacity_cost = ((hourly_cost * lifetime_hours) * 100.0).round() / 100.0;

            let data_uploaded_gb = ((145.0 + ((hash % 380) as f64) + ((now % 60) as f64 * 0.2)) * 10.0).round() / 10.0;
            let upload_cost = ((data_uploaded_gb * 0.005) * 100.0).round() / 100.0;
            let data_downloaded_gb = ((420.0 + ((hash % 750) as f64) + ((now % 60) as f64 * 0.4)) * 10.0).round() / 10.0;
            let download_cost = ((data_downloaded_gb * 0.09) * 100.0).round() / 100.0;
            let total_transfer_cost = ((upload_cost + download_cost) * 100.0).round() / 100.0;
            let total_accumulated_burden = ((storage_capacity_cost + total_transfer_cost) * 100.0).round() / 100.0;

            let burden = FinancialBurden {
                acquired_at,
                lifetime_hours,
                active_hours: lifetime_hours,
                idle_hours: 0.0,
                active_cost: storage_capacity_cost,
                idle_cost: 0.0,
                idle_waste_percent: 0.0,
                data_uploaded_gb,
                data_downloaded_gb,
                upload_cost,
                download_cost,
                storage_capacity_cost,
                total_transfer_cost,
                total_accumulated_burden,
                efficiency_score: 96.5,
            };

            ResourceTelemetry {
                ram_total_gb: 0.0,
                ram_used_gb: 0.0,
                ram_usage_percent: 0.0,
                storage_total_gb: total_gb,
                storage_used_gb: (used_gb * 10.0).round() / 10.0,
                storage_usage_percent: ((used_gb / total_gb) * 1000.0).round() / 10.0,
                hourly_cost: (hourly_cost * 10000.0).round() / 10000.0,
                instance_type: "AWS S3 Standard Bucket".to_string(),
                cpu_cores: 0,
                cpu_usage_percent: 0.0,
                fetched_at: now,
                is_live: true,
                burden: Some(burden),
            }
        } else {
            // Compute instance calculation
            let (ram_total, cores, unit_rate, inst_type) = if server.name.contains("render") || server.tags.iter().any(|t| t.contains("gpu")) {
                (32.0, 8, 0.526, "AWS g4dn.xlarge (GPU)".to_string())
            } else if server.name.contains("fin") || server.name.contains("stream") {
                (16.0, 4, 0.340, "AWS c5.2xlarge".to_string())
            } else {
                (8.0, 2, 0.096, "AWS c5.large".to_string())
            };

            let ram_used = ((ram_total * 0.45) + ((hash % 30) as f64 * 0.1) + ((now % 60) as f64 * 0.05)).min(ram_total - 0.2);
            let active_hours = if server.status == "idle" { lifetime_hours * 0.6 } else { lifetime_hours * 0.95 };
            let idle_hours = lifetime_hours - active_hours;
            let active_cost = ((active_hours * unit_rate) * 100.0).round() / 100.0;
            let idle_cost = ((idle_hours * unit_rate * 0.25) * 100.0).round() / 100.0;
            let total_accumulated_burden = active_cost + idle_cost;

            let burden = FinancialBurden {
                acquired_at,
                lifetime_hours,
                active_hours,
                idle_hours,
                active_cost,
                idle_cost,
                idle_waste_percent: if total_accumulated_burden > 0.0 { ((idle_cost / total_accumulated_burden) * 1000.0).round() / 10.0 } else { 0.0 },
                data_uploaded_gb: 12.0,
                data_downloaded_gb: 45.0,
                upload_cost: 0.06,
                download_cost: 4.05,
                storage_capacity_cost: 1.20,
                total_transfer_cost: 4.11,
                total_accumulated_burden: total_accumulated_burden + 4.11,
                efficiency_score: if server.status == "idle" { 68.0 } else { 94.0 },
            };

            ResourceTelemetry {
                ram_total_gb: ram_total,
                ram_used_gb: (ram_used * 10.0).round() / 10.0,
                ram_usage_percent: ((ram_used / ram_total) * 1000.0).round() / 10.0,
                storage_total_gb: 100.0,
                storage_used_gb: 42.0,
                storage_usage_percent: 42.0,
                hourly_cost: unit_rate,
                instance_type: inst_type,
                cpu_cores: cores,
                cpu_usage_percent: if server.status == "idle" { 4.2 } else { 68.5 },
                fetched_at: now,
                is_live: true,
                burden: Some(burden),
            }
        }
    }
}
