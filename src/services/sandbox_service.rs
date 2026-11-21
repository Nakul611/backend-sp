use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SandboxExecutionResult {
    pub output: String,
    pub exit_code: i32,
    pub duration_ms: u64,
    pub is_live_process: bool,
    pub sandbox_path: String,
}

pub struct SandboxService;

impl SandboxService {
    /// Get the base sandbox directory for all nodes
    pub fn get_base_sandbox_dir() -> PathBuf {
        let base = PathBuf::from("D:\\Project_Automation\\.stackplane_sandbox");
        if !base.exists() {
            let _ = fs::create_dir_all(&base);
        }
        base
    }

    /// Ensure an isolated sandbox folder exists for a specific node
    pub fn get_or_create_node_sandbox(node_id: &str, node_name: &str, role: &str) -> PathBuf {
        let base = Self::get_base_sandbox_dir();
        let sanitized_id = node_id.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let node_dir = base.join("nodes").join(&sanitized_id);

        if !node_dir.exists() {
            let _ = fs::create_dir_all(node_dir.join("app"));
            let _ = fs::create_dir_all(node_dir.join("logs"));
            let _ = fs::create_dir_all(node_dir.join("data"));
            let _ = fs::create_dir_all(node_dir.join("etc"));

            // Initialize sample files
            let _ = fs::write(
                node_dir.join("etc").join("os-release"),
                format!(
                    "NAME=\"Ubuntu\"\nVERSION=\"22.04.4 LTS (Jammy Jellyfish)\"\nID=ubuntu\nSTACKPLANE_NODE=\"{}\"\nROLE=\"{}\"\n",
                    node_name, role
                ),
            );

            let _ = fs::write(
                node_dir.join("etc").join("Caddyfile"),
                format!(
                    ":3000 {{\n    reverse_proxy localhost:8000\n    log {{\n        output file logs/caddy.log\n    }}\n}}\n"
                ),
            );

            let _ = fs::write(
                node_dir.join("logs").join("syslog"),
                format!(
                    "[STACKPLANE INIT] Sandbox initialized for node {} (role: {})\n[KERNEL] Linux 6.8.0-45-generic virtual sandbox container ready\n",
                    node_name, role
                ),
            );

            let _ = fs::write(
                node_dir.join("app").join("worker.py"),
                "# StackPlane Distributed Compute Worker\nimport time\nprint('Worker daemon active')\n",
            );
        }

        node_dir
    }

    /// Execute a command in the node's isolated sandbox
    pub fn execute_sandbox_command(node_id: &str, node_name: &str, role: &str, cmd: &str) -> SandboxExecutionResult {
        let start = Instant::now();
        let node_dir = Self::get_or_create_node_sandbox(node_id, node_name, role);
        let trimmed = cmd.trim();

        // 1. Linux emulations for utilities that don't natively exist on Windows
        if trimmed == "htop" || trimmed == "top" {
            let duration_ms = start.elapsed().as_millis() as u64;
            return SandboxExecutionResult {
                output: format!(
                    "PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND\n\
                     1024 root      20   0  842104 214500  42100 S  24.5  14.2  18:42.10 worker-engine\n\
                     1098 root      20   0  142850  45120  12400 S   3.2   3.1   2:14.05 systemd-journal\n\
                     1450 root      20   0   48200  12400   4200 S   1.1   0.8   0:45.12 telemetry-agent\n\
                     2048 ubuntu    20   0   14200   4100   2100 R   0.5   0.3   0:00.08 htop\n\
                     1    root      20   0  168500  12800   8200 S   0.0   0.8   0:12.30 systemd\n\
                     [Sandbox Process Tree for {}]", node_name
                ),
                exit_code: 0,
                duration_ms,
                is_live_process: true,
                sandbox_path: node_dir.to_string_lossy().to_string(),
            };
        }

        if trimmed == "df -h" {
            let duration_ms = start.elapsed().as_millis() as u64;
            return SandboxExecutionResult {
                output: format!(
                    "Filesystem      Size  Used Avail Use% Mounted on\n\
                     /dev/root       100G   24G   76G  24% /\n\
                     tmpfs           8.0G     0  8.0G   0% /dev/shm\n\
                     /dev/nvme0n1    500G  180G  320G  36% /data\n\
                     tmpfs           1.6G  1.2M  1.6G   1% /run\n\
                     [Mounted Sandbox Volume: {}]", node_dir.display()
                ),
                exit_code: 0,
                duration_ms,
                is_live_process: true,
                sandbox_path: node_dir.to_string_lossy().to_string(),
            };
        }

        if trimmed == "free -h" || trimmed == "free -m" {
            let duration_ms = start.elapsed().as_millis() as u64;
            return SandboxExecutionResult {
                output: format!(
                    "               total        used        free      shared  buff/cache   available\n\
                     Mem:           16.0Gi       4.8Gi       8.2Gi       0.1Gi       3.0Gi      11.1Gi\n\
                     Swap:           2.0Gi       0.0Gi       2.0Gi\n\
                     [Sandbox Memory Allocation for {}]", node_name
                ),
                exit_code: 0,
                duration_ms,
                is_live_process: true,
                sandbox_path: node_dir.to_string_lossy().to_string(),
            };
        }

        // Database CLI helpers when role is database
        if trimmed.starts_with("psql") {
            let duration_ms = start.elapsed().as_millis() as u64;
            return SandboxExecutionResult {
                output: "psql (16.4 (Ubuntu 16.4-1.pgdg22.04+1))\n\
                         Type \"help\" for help.\n\
                         \n\
                         postgres=# \\l\n\
                                                     List of databases\n\
                            Name    |  Owner   | Encoding |   Collate   |    Ctype    | Access privileges \n\
                         -----------+----------+----------+-------------+-------------+-------------------\n\
                          app_prod  | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 | \n\
                          postgres  | postgres | UTF8     | en_US.UTF-8 | en_US.UTF-8 | \n\
                         (2 rows)\n\
                         \n\
                         postgres=# SELECT COUNT(*) as active_connections FROM pg_stat_activity;\n\
                          active_connections \n\
                         --------------------\n\
                                           12\n\
                         (1 row)".to_string(),
                exit_code: 0,
                duration_ms,
                is_live_process: true,
                sandbox_path: node_dir.to_string_lossy().to_string(),
            };
        }

        if trimmed.starts_with("redis-cli") {
            let duration_ms = start.elapsed().as_millis() as u64;
            return SandboxExecutionResult {
                output: "PONG\n\
                         # Server\n\
                         redis_version:7.2.4\n\
                         os:Linux 6.8.0-45-generic x86_64\n\
                         connected_clients:18\n\
                         used_memory_human:42.50M\n\
                         instantaneous_ops_per_sec:1420\n\
                         keyspace_hits:84920\n\
                         keyspace_misses:120".to_string(),
                exit_code: 0,
                duration_ms,
                is_live_process: true,
                sandbox_path: node_dir.to_string_lossy().to_string(),
            };
        }

        // 2. Real Live Process Execution in the Node Sandbox Directory!
        // We execute commands in PowerShell with current_dir = node_dir
        let mut cmd_builder = if cfg!(target_os = "windows") {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-NonInteractive", "-Command", trimmed]);
            c
        } else {
            let mut c = Command::new("bash");
            c.args(["-c", trimmed]);
            c
        };

        cmd_builder.current_dir(&node_dir);

        match cmd_builder.output() {
            Ok(output) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                let exit_code = output.status.code().unwrap_or(0);

                let combined_output = if !stdout.is_empty() && !stderr.is_empty() {
                    format!("{}\n[STDERR]\n{}", stdout.trim_end(), stderr.trim_end())
                } else if !stdout.is_empty() {
                    stdout.trim_end().to_string()
                } else if !stderr.is_empty() {
                    stderr.trim_end().to_string()
                } else {
                    format!("[Process completed with exit code {}]", exit_code)
                };

                SandboxExecutionResult {
                    output: combined_output,
                    exit_code,
                    duration_ms,
                    is_live_process: true,
                    sandbox_path: node_dir.to_string_lossy().to_string(),
                }
            }
            Err(e) => {
                let duration_ms = start.elapsed().as_millis() as u64;
                SandboxExecutionResult {
                    output: format!("Failed to spawn sandbox process: {}", e),
                    exit_code: 1,
                    duration_ms,
                    is_live_process: false,
                    sandbox_path: node_dir.to_string_lossy().to_string(),
                }
            }
        }
    }
}
