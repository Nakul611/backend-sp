# StackPlane Backend

High-performance cloud infrastructure engine and control plane REST/WebSocket server written in Rust.

---

## Technical Stack

- Language: Rust 2021 Edition
- Web Framework: Axum 0.8
- Async Runtime: Tokio
- Database: MongoDB Atlas (with in-memory fallback state)
- Middleware: Tower HTTP (CORS, TraceLayer)
- Serialization: Serde, Serde JSON

---

## Core Capabilities

- Multi-Cloud Server Telemetry: Real-time CPU, RAM, disk, ingress, and egress monitoring with periodic tick broadcast.
- Quiet Fleet Idle Watcher: Automated background monitor that detects and flags idle infrastructure past thresholds.
- Cadence-Aware Budget System: Tracks financial burn rates across Daily, Weekly, Monthly, and Yearly evaluation windows.
- Automated Git Webhook CI/CD: Ingests `push` webhooks from GitHub, GitLab, and Bitbucket to automatically trigger deployment pipelines.
- Real-Time WebSocket Streaming: Pushes execution logs, status changes, and telemetry alerts to all connected frontend clients.

---

## Getting Started

### Prerequisites

- Rust 1.75+ and Cargo
- LLVM / MinGW or MSVC toolchain (Windows)

### Run in Development

```bash
cargo run
```

The REST and WebSocket server runs on `http://127.0.0.1:8000`.

### Production Build

```bash
cargo build --release
```

Binary is output to `target/release/backend`.

---

## Environment Configuration

Configure via environment variables or a `.env` file in the `backend/` directory:

| Variable | Default | Description |
|---|---|---|
| `PORT` | `8000` | HTTP and WebSocket listening port |
| `MONGODB_URI` | `mongodb://localhost:27017` | MongoDB connection string |
| `JWT_SECRET` | `stackplane-secret-key-change-in-production` | Secret key for JWT auth tokens |
| `RUST_LOG` | `backend=info,tower_http=info` | Tracing log filter level |

---

## API Routes Summary

### Projects & Workspaces
- `GET /api/projects` - List all projects
- `POST /api/projects` - Create project workspace
- `POST /api/projects/:id/close` - Archive/close project
- `POST /api/projects/:id/reopen` - Reopen project

### Servers & Compute Nodes
- `GET /api/projects/:id/servers` - List servers for a project
- `POST /api/servers/provision` - Provision new nodes
- `POST /api/servers/:id/stop` - Stop a running node
- `GET /api/servers/:id/telemetry` - Get live server metrics

### Pipeline & Execution
- `POST /api/projects/:id/execute` - Dispatch command across project nodes
- `POST /api/vcs-providers/webhook/:provider` - Automated Git push webhook listener

### Real-Time WebSocket
- `GET /api/system/ws` - WebSocket feed for live telemetry and execution logs
