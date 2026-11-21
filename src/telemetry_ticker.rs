use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;

use crate::models::{FinancialBurden, WsEvent};
use crate::state::AppState;

pub fn start_telemetry_ticker(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Real-Time Telemetry & Financial Burden Ticker started (3s resolution)");

        let mut tick_counter: u64 = 0;

        loop {
            sleep(Duration::from_secs(3)).await;
            tick_counter = tick_counter.wrapping_add(1);

            let now = Utc::now().timestamp();
            let mut events_to_broadcast = Vec::new();

            {
                let mut servers = state.servers.write().await;

                // Pick a subset of active servers to update each tick to avoid spamming 50 events at once
                // Every 3 seconds, tick the active servers
                for (idx, server) in servers.iter_mut().enumerate() {
                    if server.status != "active" {
                        continue;
                    }

                    // Stagger updates slightly so events stream continuously
                    if (idx as u64 + tick_counter) % 2 != 0 {
                        continue;
                    }

                    if let Some(ref mut telem) = server.telemetry {
                        // Micro-fluctuate CPU usage realistically around baseline
                        let jitter = ((tick_counter.wrapping_mul(7) + idx as u64) % 7) as f64 - 3.0; // -3 to +3%
                        let new_cpu = (telem.cpu_usage_percent + jitter).clamp(5.0, 98.0);
                        telem.cpu_usage_percent = (new_cpu * 10.0).round() / 10.0;
                        telem.fetched_at = now;

                        // Accrue financial burden by 3 seconds
                        let hourly = telem.hourly_cost;
                        let increment_usd = (hourly / 3600.0) * 3.0;

                        if let Some(ref mut burden) = telem.burden {
                            burden.total_accumulated_burden += increment_usd;
                            burden.active_hours += 3.0 / 3600.0;
                            burden.lifetime_hours += 3.0 / 3600.0;
                            burden.data_uploaded_gb += 0.0005;
                            burden.data_downloaded_gb += 0.0012;
                        } else {
                            telem.burden = Some(FinancialBurden {
                                acquired_at: server.created_at,
                                lifetime_hours: 1.0,
                                active_hours: 1.0,
                                idle_hours: 0.0,
                                active_cost: 0.10,
                                idle_cost: 0.0,
                                idle_waste_percent: 0.0,
                                data_uploaded_gb: 0.05,
                                data_downloaded_gb: 0.12,
                                upload_cost: 0.001,
                                download_cost: 0.002,
                                storage_capacity_cost: 0.005,
                                total_transfer_cost: 0.003,
                                total_accumulated_burden: 0.11,
                                efficiency_score: 96.0,
                            });
                        }

                        // Collect live telemetry update
                        events_to_broadcast.push(WsEvent {
                            r#type: "TELEMETRY_STREAM".to_string(),
                            project_id: Some(server.project_id.clone()),
                            server_id: Some(server.id.clone()),
                            status: Some(server.status.clone()),
                            idle_minutes: Some(server.idle_minutes),
                            message: None,
                            telemetry: Some(telem.clone()),
                        });
                    }
                }
            }

            // Broadcast outside of the servers lock to prevent lock contention
            for event in events_to_broadcast {
                state.broadcast_event(&event);
            }
        }
    });
}
