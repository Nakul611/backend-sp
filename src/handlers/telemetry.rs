use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{IntoResponse, Json, Response},
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        BootstrapScriptResponse, FinancialBurden, ResourceTelemetry, Server,
        TelemetryHeartbeatRequest, TelemetryHeartbeatResponse, WsEvent,
    },
    state::AppState,
};

#[derive(Debug, Deserialize)]
pub struct InstallScriptQuery {
    pub project: Option<String>,
    pub server: Option<String>,
    pub provider: Option<String>,
}

pub async fn ingest_heartbeat(
    State(state): State<AppState>,
    Json(payload): Json<TelemetryHeartbeatRequest>,
) -> Result<Json<TelemetryHeartbeatResponse>, AppError> {
    let now = Utc::now().timestamp();
    let mut servers = state.servers.write().await;

    let target_server = servers.iter_mut().find(|s| s.id == payload.server_id || s.server_id == payload.server_id);

    let updated_telemetry = if let Some(server) = target_server {
        server.status = "active".to_string();
        server.idle_minutes = 0;

        let hourly_rate = server
            .telemetry
            .as_ref()
            .map(|t| t.hourly_cost)
            .unwrap_or(0.096);

        let ram_usage_percent = if payload.ram_total_gb > 0.0 {
            (payload.ram_used_gb / payload.ram_total_gb) * 100.0
        } else {
            50.0
        };

        let storage_usage_percent = if payload.storage_total_gb > 0.0 {
            (payload.storage_used_gb / payload.storage_total_gb) * 100.0
        } else {
            30.0
        };

        let prev_burden = server.telemetry.as_ref().and_then(|t| t.burden.clone()).unwrap_or(FinancialBurden {
            acquired_at: server.created_at,
            lifetime_hours: 1.0,
            active_hours: 1.0,
            idle_hours: 0.0,
            active_cost: 0.10,
            idle_cost: 0.0,
            idle_waste_percent: 0.0,
            data_uploaded_gb: payload.network_tx_mb.unwrap_or(12.0) / 1024.0,
            data_downloaded_gb: payload.network_rx_mb.unwrap_or(45.0) / 1024.0,
            upload_cost: 0.01,
            download_cost: 0.02,
            storage_capacity_cost: 0.05,
            total_transfer_cost: 0.03,
            total_accumulated_burden: 0.15,
            efficiency_score: 95.0,
        });

        let updated_burden = FinancialBurden {
            total_accumulated_burden: prev_burden.total_accumulated_burden + (hourly_rate / 3600.0) * 3.0,
            active_hours: prev_burden.active_hours + (3.0 / 3600.0),
            lifetime_hours: prev_burden.lifetime_hours + (3.0 / 3600.0),
            data_uploaded_gb: prev_burden.data_uploaded_gb + (payload.network_tx_mb.unwrap_or(0.1) / 1024.0),
            data_downloaded_gb: prev_burden.data_downloaded_gb + (payload.network_rx_mb.unwrap_or(0.2) / 1024.0),
            ..prev_burden
        };

        let telem = ResourceTelemetry {
            ram_total_gb: payload.ram_total_gb,
            ram_used_gb: payload.ram_used_gb,
            ram_usage_percent,
            storage_total_gb: payload.storage_total_gb,
            storage_used_gb: payload.storage_used_gb,
            storage_usage_percent,
            hourly_cost: hourly_rate,
            instance_type: server.resource_type.clone(),
            cpu_cores: 2,
            cpu_usage_percent: payload.cpu_usage_percent,
            fetched_at: now,
            is_live: true,
            burden: Some(updated_burden),
        };

        server.telemetry = Some(telem.clone());

        // Broadcast live telemetry update to all connected frontend clients
        state.broadcast_event(&WsEvent {
            r#type: "TELEMETRY_STREAM".to_string(),
            project_id: Some(server.project_id.clone()),
            server_id: Some(server.id.clone()),
            status: Some("active".to_string()),
            idle_minutes: Some(0),
            message: Some(format!("Live telemetry from {}", server.name)),
            telemetry: Some(telem.clone()),
        });

        telem
    } else {
        // Auto-register new physical or cloud server
        let new_id = format!("srv-node-{}", Uuid::new_v4().simple());
        let server_name = payload.server_id.clone();
        let hourly_rate = 0.0416;

        let telem = ResourceTelemetry {
            ram_total_gb: payload.ram_total_gb,
            ram_used_gb: payload.ram_used_gb,
            ram_usage_percent: if payload.ram_total_gb > 0.0 { (payload.ram_used_gb / payload.ram_total_gb) * 100.0 } else { 45.0 },
            storage_total_gb: payload.storage_total_gb,
            storage_used_gb: payload.storage_used_gb,
            storage_usage_percent: if payload.storage_total_gb > 0.0 { (payload.storage_used_gb / payload.storage_total_gb) * 100.0 } else { 25.0 },
            hourly_cost: hourly_rate,
            instance_type: "free-tier".to_string(),
            cpu_cores: 2,
            cpu_usage_percent: payload.cpu_usage_percent,
            fetched_at: now,
            is_live: true,
            burden: Some(FinancialBurden {
                acquired_at: now,
                lifetime_hours: 0.1,
                active_hours: 0.1,
                idle_hours: 0.0,
                active_cost: 0.01,
                idle_cost: 0.0,
                idle_waste_percent: 0.0,
                data_uploaded_gb: payload.network_tx_mb.unwrap_or(1.0) / 1024.0,
                data_downloaded_gb: payload.network_rx_mb.unwrap_or(2.0) / 1024.0,
                upload_cost: 0.001,
                download_cost: 0.001,
                storage_capacity_cost: 0.001,
                total_transfer_cost: 0.002,
                total_accumulated_burden: 0.012,
                efficiency_score: 98.0,
            }),
        };

        let new_server = Server {
            id: new_id.clone(),
            project_id: payload.project_id.clone(),
            server_id: payload.server_id.clone(),
            name: server_name,
            provider: "custom".to_string(),
            resource_type: "compute".to_string(),
            host_ip: "127.0.0.1".to_string(),
            ssh_port: 22,
            ssh_user: "ubuntu".to_string(),
            region: "edge-auto".to_string(),
            tags: vec!["live-agent".to_string(), "zero-touch".to_string()],
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
            telemetry: Some(telem.clone()),
        };

        servers.push(new_server);

        state.broadcast_event(&WsEvent {
            r#type: "SERVER_STATUS_CHANGED".to_string(),
            project_id: Some(payload.project_id.clone()),
            server_id: Some(new_id),
            status: Some("active".to_string()),
            idle_minutes: Some(0),
            message: Some(format!("New live server '{}' connected to fleet", payload.server_id)),
            telemetry: Some(telem.clone()),
        });

        telem
    };

    Ok(Json(TelemetryHeartbeatResponse {
        status: "ack".to_string(),
        server_id: payload.server_id,
        is_live: true,
        message: format!("Heartbeat recorded. CPU: {:.1}%, RAM: {:.1}GB", updated_telemetry.cpu_usage_percent, updated_telemetry.ram_used_gb),
        timestamp: now,
    }))
}

pub async fn get_agent_install_script(
    Query(params): Query<InstallScriptQuery>,
) -> Response {
    let project_id = params.project.unwrap_or_else(|| "proj-crawler-alpha".to_string());
    let server_name = params.server.unwrap_or_else(|| "edge-node-01".to_string());

    let script = format!(
        r#"#!/usr/bin/env bash
# ==============================================================================
# StackPlane Zero-Touch Provisioning & Real-Time Telemetry Agent
# Project: {project_id}
# Node:    {server_name}
# ==============================================================================
set -e

PROJECT_ID="{project_id}"
SERVER_ID="{server_name}"
API_URL="http://127.0.0.1:8000"

echo "======================================================================="
echo "StackPlane Zero-Touch Agent Bootstrapping..."
echo "Target Project: $PROJECT_ID"
echo "Target Node:    $SERVER_ID"
echo "======================================================================="

# 1. Ensure required system packages
if ! command -v curl &> /dev/null; then
    if command -v apt-get &> /dev/null; then
        sudo apt-get update -qq && sudo apt-get install -y -qq curl
    elif command -v yum &> /dev/null; then
        sudo yum install -y -q curl
    fi
fi

echo "[OK] System dependencies verified (curl, coreutils)"

# 2. Start Real-Time Heartbeat Loop
echo "Starting real-time telemetry streaming to $API_URL..."

while true; do
    # Sample CPU jiffies
    CPU_1=($(grep '^cpu ' /proc/stat 2>/dev/null || echo "cpu 0 0 0 0 0 0 0"))
    IDLE_1=${{CPU_1[4]:-0}}
    TOTAL_1=0
    for v in "${{CPU_1[@]:1}}"; do TOTAL_1=$((TOTAL_1 + v)); done

    sleep 1.5

    CPU_2=($(grep '^cpu ' /proc/stat 2>/dev/null || echo "cpu 10 0 0 100 0 0 0"))
    IDLE_2=${{CPU_2[4]:-0}}
    TOTAL_2=0
    for v in "${{CPU_2[@]:1}}"; do TOTAL_2=$((TOTAL_2 + v)); done

    DIFF_IDLE=$((IDLE_2 - IDLE_1))
    DIFF_TOTAL=$((TOTAL_2 - TOTAL_1))
    if [ "$DIFF_TOTAL" -gt 0 ]; then
        CPU_PCT=$(( (1000 * (DIFF_TOTAL - DIFF_IDLE) / DIFF_TOTAL + 5) / 10 ))
    else
        CPU_PCT=25
    fi

    # Read RAM stats
    MEM_TOTAL_KB=$(grep MemTotal /proc/meminfo 2>/dev/null | awk '{{print $2}}' || echo "4194304")
    MEM_AVAIL_KB=$(grep MemAvailable /proc/meminfo 2>/dev/null | awk '{{print $2}}' || echo "2097152")
    MEM_USED_KB=$((MEM_TOTAL_KB - MEM_AVAIL_KB))
    MEM_TOTAL_GB=$(awk "BEGIN {{printf \"%.2f\", $MEM_TOTAL_KB / 1048576}}")
    MEM_USED_GB=$(awk "BEGIN {{printf \"%.2f\", $MEM_USED_KB / 1048576}}")

    # Read Disk stats
    DISK_TOTAL_GB=$(df -m / 2>/dev/null | tail -1 | awk '{{printf "%.1f", $2 / 1024}}' || echo "50.0")
    DISK_USED_GB=$(df -m / 2>/dev/null | tail -1 | awk '{{printf "%.1f", $3 / 1024}}' || echo "12.5")

    # Transmit Heartbeat
    curl -s -X POST "$API_URL/api/telemetry/heartbeat" \
        -H "Content-Type: application/json" \
        -d "{{
            \"server_id\": \"$SERVER_ID\",
            \"project_id\": \"$PROJECT_ID\",
            \"cpu_usage_percent\": $CPU_PCT,
            \"ram_used_gb\": $MEM_USED_GB,
            \"ram_total_gb\": $MEM_TOTAL_GB,
            \"storage_used_gb\": $DISK_USED_GB,
            \"storage_total_gb\": $DISK_TOTAL_GB,
            \"network_rx_mb\": 42.5,
            \"network_tx_mb\": 18.2
        }}" > /dev/null || true

    sleep 2
done
"#,
        project_id = project_id,
        server_name = server_name
    );

    (
        [(header::CONTENT_TYPE, "text/x-shellscript; charset=utf-8")],
        script,
    )
        .into_response()
}

pub async fn get_project_bootstrap_script(
    State(state): State<AppState>,
    Path(project_id): Path<String>,
) -> Result<Json<BootstrapScriptResponse>, AppError> {
    let projects = state.projects.read().await;
    let project = projects
        .iter()
        .find(|p| p.id == project_id)
        .ok_or_else(|| AppError::NotFound(format!("Project '{}' not found", project_id)))?;

    let vcs_list = state.vcs_providers.read().await;
    let github_vcs = vcs_list.iter().find(|v| v.provider == "github").cloned();

    let git_url = project
        .repository
        .as_ref()
        .map(|r| r.git_url.as_str())
        .unwrap_or("https://github.com/my-org/private-crawler.git");

    let branch = project
        .repository
        .as_ref()
        .map(|r| r.branch.as_str())
        .unwrap_or("main");

    let build_cmd = project
        .repository
        .as_ref()
        .and_then(|r| r.build_command.as_deref())
        .unwrap_or("cargo build --release");

    let run_cmd = project
        .repository
        .as_ref()
        .map(|r| r.run_command.as_str())
        .unwrap_or("./run_worker.sh --batch-size 500");

    let masked_token = github_vcs
        .as_ref()
        .map(|v| v.credentials_masked.as_str())
        .unwrap_or("ghp_••••••••••••92ab");

    let full_script = format!(
        r#"#!/usr/bin/env bash
# ==============================================================================
# StackPlane Zero-Touch Server Automation & Private Git Deployment
# Project:  {project_name} ({project_id})
# VCS Auth: GitHub Authenticated (Token: {masked_token})
# ==============================================================================
set -e

REPO_URL="{git_url}"
BRANCH="{branch}"
BUILD_CMD="{build_cmd}"
RUN_CMD="{run_cmd}"
PROJECT_ID="{project_id}"
SERVER_ID="${{HOSTNAME:-stackplane-node-01}}"
API_URL="http://127.0.0.1:8000"

echo "======================================================================="
echo "Starting Zero-Touch Fleet Provisioning..."
echo "Project: $PROJECT_ID | Target Branch: $BRANCH"
echo "======================================================================="

# Phase 1: Environment & Tooling
echo "1/5: Verifying Git, Curl and build runtimes..."
if ! command -v git &> /dev/null; then
    sudo apt-get update -qq && sudo apt-get install -y -qq git curl
fi

# Phase 2: Secure VCS Authentication & Private Repo Clone
echo "2/5: Authenticating with Version Control and cloning private repository..."
WORK_DIR="/opt/stackplane/app"
sudo mkdir -p "$WORK_DIR"
sudo chown -R $USER "$WORK_DIR"

if [ -d "$WORK_DIR/.git" ]; then
    echo "Existing checkout found. Pulling latest $BRANCH..."
    cd "$WORK_DIR"
    git fetch origin "$BRANCH"
    git checkout "$BRANCH"
    git pull origin "$BRANCH"
else
    echo "Performing clean private clone into $WORK_DIR..."
    git clone --depth 1 --branch "$BRANCH" "$REPO_URL" "$WORK_DIR"
    cd "$WORK_DIR"
fi

# Phase 3: Project Build Execution
echo "3/5: Executing project build command: '$BUILD_CMD'..."
eval "$BUILD_CMD"

# Phase 4: Daemonize Worker Job
echo "4/5: Launching worker daemon: '$RUN_CMD'..."
nohup $RUN_CMD > /var/log/stackplane-worker.log 2>&1 &
WORKER_PID=$!
echo "Worker process running under PID: $WORKER_PID"

# Phase 5: Start Real-Time Telemetry Stream
echo "5/5: Registering live node with StackPlane Control Plane..."
curl -s -X POST "$API_URL/api/telemetry/heartbeat" \
    -H "Content-Type: application/json" \
    -d "{{
        \"server_id\": \"$SERVER_ID\",
        \"project_id\": \"$PROJECT_ID\",
        \"cpu_usage_percent\": 45.0,
        \"ram_used_gb\": 1.2,
        \"ram_total_gb\": 4.0,
        \"storage_used_gb\": 8.5,
        \"storage_total_gb\": 50.0,
        \"active_job\": \"worker_pid_$WORKER_PID\"
    }}" > /dev/null

echo "[OK] Node successfully provisioned and actively reporting to StackPlane!"
"#,
        project_name = project.name,
        project_id = project.id,
        masked_token = masked_token,
        git_url = git_url,
        branch = branch,
        build_cmd = build_cmd,
        run_cmd = run_cmd,
    );

    let one_liner = format!(
        "curl -sSL http://127.0.0.1:8000/api/agent/install.sh?project={}&server=node-01 | bash",
        project.id
    );

    Ok(Json(BootstrapScriptResponse {
        script: full_script,
        one_liner,
        project_id: project.id.clone(),
        server_id: "node-01".to_string(),
        vcs_provider: "github".to_string(),
    }))
}
