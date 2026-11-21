use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use uuid::Uuid;

use crate::models::{FleetNotification, WsEvent};
use crate::state::AppState;

pub fn start_idle_watcher(state: AppState) {
    tokio::spawn(async move {
        tracing::info!("Quiet Fleet Idle Watcher initiated (60s check cadence)");

        loop {
            // Calm, non-intrusive 60s cadence
            sleep(Duration::from_secs(60)).await;

            let now = Utc::now().timestamp();
            let mut servers = state.servers.write().await;
            let projects = state.projects.read().await;

            let mut new_alerts = Vec::new();

            for server in servers.iter_mut() {
                if server.status == "idle" {
                    server.idle_minutes += 1;

                    let project = projects
                        .iter()
                        .find(|p| p.id == server.project_id);

                    let threshold = project
                        .map(|p| p.idle_alert_threshold_minutes)
                        .unwrap_or(15);
                    let auto_stop = project
                        .map(|p| p.auto_stop_idle)
                        .unwrap_or(false);

                    // If project has auto-stop enabled and server exceeds threshold, auto-stop to save money!
                    if auto_stop && server.idle_minutes >= threshold {
                        server.status = "terminated".to_string();
                        state.broadcast_event(&WsEvent {
                            r#type: "SERVER_STATUS_CHANGED".to_string(),
                            project_id: Some(server.project_id.clone()),
                            server_id: Some(server.server_id.clone()),
                            status: Some("terminated".to_string()),
                            idle_minutes: Some(server.idle_minutes),
                            message: Some(format!("Server {} auto-stopped to prevent idle cost bleed", server.name)),
                            telemetry: None,
                        });
                        new_alerts.push(FleetNotification {
                            id: format!("notif-{}", Uuid::new_v4().simple()),
                            project_id: server.project_id.clone(),
                            r#type: "auto_stopped".to_string(),
                            severity: "info".to_string(),
                            title: format!("Auto-Stopped: {}", server.name),
                            message: format!("Instance {} auto-stopped after {}m idle to save money", server.name, server.idle_minutes),
                            server_id: Some(server.server_id.clone()),
                            host_ip: Some(server.host_ip.clone()),
                            idle_minutes: Some(server.idle_minutes),
                            read: false,
                            created_at: now,
                        });
                        continue;
                    }

                    // Alert exactly ONCE when crossing threshold, preventing alert fatigue and noise
                    if server.idle_minutes == threshold {
                        let alert_msg = format!(
                            "Server {} ({}) has been idle for {}m without active jobs.",
                            server.name, server.host_ip, server.idle_minutes
                        );

                        new_alerts.push(FleetNotification {
                            id: format!("notif-{}", Uuid::new_v4().simple()),
                            project_id: server.project_id.clone(),
                            r#type: "server_idle_alert".to_string(),
                            severity: "warning".to_string(),
                            title: format!("Idle Notice: {}", server.name),
                            message: alert_msg,
                            server_id: Some(server.server_id.clone()),
                            host_ip: Some(server.host_ip.clone()),
                            idle_minutes: Some(server.idle_minutes),
                            read: false,
                            created_at: now,
                        });
                    }
                }
            }

            // Only broadcast and store if there is an actual new alert
            if !new_alerts.is_empty() {
                let mut notifs = state.notifications.write().await;
                for alert in new_alerts {
                    state.broadcast_event(&WsEvent {
                        r#type: "IDLE_ALERT".to_string(),
                        project_id: Some(alert.project_id.clone()),
                        server_id: alert.server_id.clone(),
                        status: Some("idle".to_string()),
                        idle_minutes: alert.idle_minutes,
                        message: Some(alert.message.clone()),
                        telemetry: None,
                    });
                    notifs.insert(0, alert);
                }
                if notifs.len() > 30 {
                    notifs.truncate(30);
                }
            }
        }
    });
}
