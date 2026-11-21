use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use chrono::Utc;

use crate::auth::AuthUser;
use crate::models::{CloudProviderConfig, Credential, FleetNotification, Project, ProjectRepository, Server, VcsProviderConfig, WsEvent};

#[derive(Clone)]
pub struct AppState {
    pub users: Arc<RwLock<Vec<(AuthUser, String)>>>,
    pub projects: Arc<RwLock<Vec<Project>>>,
    pub servers: Arc<RwLock<Vec<Server>>>,
    pub credentials: Arc<RwLock<Vec<Credential>>>,
    pub cloud_providers: Arc<RwLock<Vec<CloudProviderConfig>>>,
    pub vcs_providers: Arc<RwLock<Vec<VcsProviderConfig>>>,
    pub notifications: Arc<RwLock<Vec<FleetNotification>>>,
    pub ws_tx: broadcast::Sender<String>,
    pub mongo_client: Option<mongodb::Client>,
}

impl AppState {
    pub async fn new(mongo_uri: &str) -> Self {
        let (ws_tx, _) = broadcast::channel(1024);

        // Attempt MongoDB Atlas connection
        let mongo_client = match mongodb::Client::with_uri_str(mongo_uri).await {
            Ok(client) => {
                tracing::info!("Connected to MongoDB Atlas successfully");
                Some(client)
            }
            Err(err) => {
                tracing::warn!("MongoDB connection warning: {}. Running with persistent memory store.", err);
                None
            }
        };

        let now = Utc::now().timestamp();

        // Seeded Team Users (including user requested: user1234 / pass5678)
        let initial_users = vec![
            (
                AuthUser {
                    id: "user-admin-01".to_string(),
                    username: "user1234".to_string(),
                    email: "user1234@stackplane.io".to_string(),
                    role: "admin".to_string(),
                    assigned_projects: vec!["all".to_string()],
                },
                "pass5678".to_string(),
            ),
            (
                AuthUser {
                    id: "user-admin-02".to_string(),
                    username: "nakul_admin".to_string(),
                    email: "admin@stackplane.io".to_string(),
                    role: "admin".to_string(),
                    assigned_projects: vec!["all".to_string()],
                },
                "admin123".to_string(),
            ),
            (
                AuthUser {
                    id: "user-devops-03".to_string(),
                    username: "devops_lead".to_string(),
                    email: "devops@stackplane.io".to_string(),
                    role: "devops".to_string(),
                    assigned_projects: vec!["all".to_string()],
                },
                "devops123".to_string(),
            ),
            (
                AuthUser {
                    id: "user-member-04".to_string(),
                    username: "alex_engineer".to_string(),
                    email: "member@stackplane.io".to_string(),
                    role: "member".to_string(),
                    assigned_projects: vec!["proj-crawler-alpha".to_string()],
                },
                "member123".to_string(),
            ),
        ];

        // Seed Project 1: Crawler Fleet (50 servers)
        let project_id_1 = "proj-crawler-alpha".to_string();
        let project_1 = Project {
            id: project_id_1.clone(),
            name: "High-Throughput Parallel Web Crawler".to_string(),
            slug: "parallel-crawler".to_string(),
            description: "50x AWS EC2 c5.large & Azure VM compute nodes running distributed batch jobs.".to_string(),
            status: "open".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/parallel-crawler.git".to_string(),
                branch: "main".to_string(),
                build_command: Some("cargo build --release".to_string()),
                run_command: "./run_worker.sh --batch-size 500".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 15,
            auto_stop_idle: false,
            budget_cap_usd: Some(10000.0),
            member_ids: vec!["user-admin-01".to_string(), "user-devops-03".to_string(), "user-member-04".to_string()],
            server_count: 50,
            active_count: 32,
            idle_count: 18,
            created_at: now - 3600 * 24 * 7,
            updated_at: now,
        };

        // Seed Project 2: AI Inference Fleet (20 servers)
        let project_id_2 = "proj-ai-inference".to_string();
        let project_2 = Project {
            id: project_id_2.clone(),
            name: "Distributed LLM & Vision Inference Fleet".to_string(),
            slug: "ai-inference-fleet".to_string(),
            description: "20x GPU instances (AWS g4dn.xlarge & GCP T4) handling asynchronous model inference batches.".to_string(),
            status: "open".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/vision-batch.git".to_string(),
                branch: "production".to_string(),
                build_command: Some("pip install -r requirements.txt".to_string()),
                run_command: "python run_inference.py --shards 20".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 20,
            auto_stop_idle: false,
            budget_cap_usd: Some(5000.0),
            member_ids: vec!["user-admin-01".to_string(), "user-devops-03".to_string()],
            server_count: 20,
            active_count: 16,
            idle_count: 4,
            created_at: now - 3600 * 24 * 3,
            updated_at: now,
        };

        // Seed Project 3: Financial Streaming (15 servers)
        let project_id_3 = "proj-fin-streaming".to_string();
        let project_3 = Project {
            id: project_id_3.clone(),
            name: "Real-Time Financial Streaming Pipeline".to_string(),
            slug: "financial-streaming".to_string(),
            description: "15x AWS c5.2xlarge Kafka & Flink distributed streaming workers processing live tick feeds.".to_string(),
            status: "open".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/fin-stream.git".to_string(),
                branch: "main".to_string(),
                build_command: Some("mvn clean package".to_string()),
                run_command: "java -jar target/stream-consumer.jar".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 10,
            auto_stop_idle: true,
            budget_cap_usd: Some(8000.0),
            member_ids: vec!["user-admin-01".to_string(), "user-devops-03".to_string()],
            server_count: 15,
            active_count: 12,
            idle_count: 3,
            created_at: now - 3600 * 24 * 5,
            updated_at: now,
        };

        // Seed Project 4: Autonomous Kubernetes Edge Ingress (12 servers)
        let project_id_4 = "proj-k8s-edge".to_string();
        let project_4 = Project {
            id: project_id_4.clone(),
            name: "Autonomous Kubernetes Edge Ingress".to_string(),
            slug: "k8s-edge-ingress".to_string(),
            description: "12x Azure D4s_v5 nodes routing edge global ingress, Envoy proxying, and SSL termination.".to_string(),
            status: "open".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/k8s-edge.git".to_string(),
                branch: "main".to_string(),
                build_command: None,
                run_command: "envoy -c /etc/envoy/envoy.yaml".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 15,
            auto_stop_idle: false,
            budget_cap_usd: Some(6000.0),
            member_ids: vec!["user-admin-01".to_string(), "user-devops-03".to_string()],
            server_count: 12,
            active_count: 10,
            idle_count: 2,
            created_at: now - 3600 * 24 * 6,
            updated_at: now,
        };

        // Seed Project 5: ElasticSearch & Clickstream Analytics (16 servers)
        let project_id_5 = "proj-search-analytics".to_string();
        let project_5 = Project {
            id: project_id_5.clone(),
            name: "ElasticSearch & Clickstream Analytics".to_string(),
            slug: "search-analytics".to_string(),
            description: "16x AWS r5.large nodes running distributed Lucene search indexers and clickstream aggregation.".to_string(),
            status: "open".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/search-cluster.git".to_string(),
                branch: "production".to_string(),
                build_command: None,
                run_command: "./bin/opensearch".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 20,
            auto_stop_idle: false,
            budget_cap_usd: Some(7500.0),
            member_ids: vec!["user-admin-01".to_string(), "user-member-04".to_string()],
            server_count: 16,
            active_count: 14,
            idle_count: 2,
            created_at: now - 3600 * 24 * 4,
            updated_at: now,
        };

        // Seed Project 6: GenAI 3D Video Rendering Farm (10 servers, Closed)
        let project_id_6 = "proj-genai-render".to_string();
        let project_6 = Project {
            id: project_id_6.clone(),
            name: "GenAI 3D Video Rendering Farm".to_string(),
            slug: "genai-render-farm".to_string(),
            description: "10x GCP L4/A100 GPU compute nodes for offline video diffusion. (Batch completed / Archived).".to_string(),
            status: "closed".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/video-diffusion.git".to_string(),
                branch: "main".to_string(),
                build_command: Some("pip install torch diffusers".to_string()),
                run_command: "python render_batch.py".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 30,
            auto_stop_idle: true,
            budget_cap_usd: Some(12000.0),
            member_ids: vec!["user-admin-01".to_string(), "user-devops-03".to_string()],
            server_count: 10,
            active_count: 0,
            idle_count: 10,
            created_at: now - 3600 * 24 * 14,
            updated_at: now - 3600 * 24 * 2,
        };

        // Seed Project 7: Legacy Log Archive & Cold Storage Lake (8 servers, Closed)
        let project_id_7 = "proj-log-archive".to_string();
        let project_7 = Project {
            id: project_id_7.clone(),
            name: "Legacy Log Archive & Cold Storage Lake".to_string(),
            slug: "log-archive-lake".to_string(),
            description: "8x GCP e2-standard-4 batch ingestion and GCS sync daemons. (Task Complete / Closed).".to_string(),
            status: "closed".to_string(),
            repository: Some(ProjectRepository {
                git_url: "https://github.com/my-org/log-sink.git".to_string(),
                branch: "master".to_string(),
                build_command: None,
                run_command: "./archive_gcs_sync.sh".to_string(),
            }),
            default_credential_id: Some("cred-aws-prod".to_string()),
            idle_alert_threshold_minutes: 15,
            auto_stop_idle: true,
            budget_cap_usd: Some(3000.0),
            member_ids: vec!["user-admin-01".to_string()],
            server_count: 8,
            active_count: 0,
            idle_count: 8,
            created_at: now - 3600 * 24 * 20,
            updated_at: now - 3600 * 24 * 5,
        };

        // Seed default Admin PEM Credential
        let initial_credential = Credential {
            id: "cred-aws-prod".to_string(),
            name: "AWS EC2 Production Cluster (us-east-1)".to_string(),
            provider: "aws".to_string(),
            credential_type: "ssh_pem".to_string(),
            ssh_user: "ubuntu".to_string(),
            fingerprint: Some("SHA256:4b:9a:12:cd:ef:88:99:aa".to_string()),
            created_at: now - 3600 * 48,
        };

        // Seed initial servers for Project 1 (50 servers)
        let mut initial_servers = Vec::new();
        let regions = ["us-east-1a", "us-east-1b", "us-west-2", "eu-central-1", "ap-south-1"];

        for i in 1..=50i64 {
            let is_idle = i > 32;
            let idle_mins: i64 = if is_idle { (i - 30) * 4 } else { 0 };
            let region = regions[(i as usize) % regions.len()];

            let (provider, resource_type, ip_prefix) = if i <= 35 {
                ("aws", "ec2", "54.210.")
            } else if i <= 45 {
                ("azure", "vm", "20.198.")
            } else if i <= 48 {
                ("gcp", "vm", "34.120.")
            } else {
                ("aws", "s3", "s3.storage.")
            };

            let server_id = format!("i-0{:04x}{:04x}", i * 17, i * 31);
            let name = format!("{}-worker-{:02}", provider, i);
            let host_ip = format!("{}{}.{}", ip_prefix, (i * 3) % 250, (i * 7) % 250);

            initial_servers.push(Server {
                id: format!("srv-{}", i),
                project_id: project_id_1.clone(),
                server_id,
                name,
                provider: provider.to_string(),
                resource_type: resource_type.to_string(),
                host_ip,
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: region.to_string(),
                tags: vec!["parallel-compute".to_string(), format!("batch-{}", (i % 5) + 1)],
                status: if is_idle { "idle".to_string() } else { "active".to_string() },
                active_since: now - (3600 * (i % 12 + 1)),
                last_job_started_at: Some(now - 7200),
                last_job_completed_at: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_since: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_minutes: idle_mins,
                last_error: None,
                created_at: now - 3600 * 24,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 2 (20 GPU servers)
        for j in 1..=20i64 {
            let is_idle = j > 16;
            let idle_mins: i64 = if is_idle { j * 3 } else { 0 };
            initial_servers.push(Server {
                id: format!("srv-gpu-{}", j),
                project_id: project_id_2.clone(),
                server_id: format!("i-gpu-{:04x}", j * 43),
                name: format!("gpu-inference-{:02}", j),
                provider: if j <= 14 { "aws".to_string() } else { "gcp".to_string() },
                resource_type: "ec2".to_string(),
                host_ip: format!("52.90.{}.{}", (j * 7) % 250, (j * 11) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "us-east-1".to_string(),
                tags: vec!["gpu-node".to_string(), "t4-tensor".to_string()],
                status: if is_idle { "idle".to_string() } else { "active".to_string() },
                active_since: now - (3600 * (j % 6 + 1)),
                last_job_started_at: Some(now - 3600),
                last_job_completed_at: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_since: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_minutes: idle_mins,
                last_error: None,
                created_at: now - 3600 * 24 * 3,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 3 (15 Kafka/Flink streaming servers)
        for k in 1..=15i64 {
            let is_idle = k > 12;
            let idle_mins: i64 = if is_idle { (k - 12) * 5 } else { 0 };
            initial_servers.push(Server {
                id: format!("srv-fin-{}", k),
                project_id: project_id_3.clone(),
                server_id: format!("i-fin-{:04x}", k * 29),
                name: format!("fin-stream-node-{:02}", k),
                provider: "aws".to_string(),
                resource_type: "ec2".to_string(),
                host_ip: format!("54.211.{}.{}", (k * 13) % 250, (k * 17) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "us-east-1".to_string(),
                tags: vec!["stream-worker".to_string(), "kafka-consumer".to_string()],
                status: if is_idle { "idle".to_string() } else { "active".to_string() },
                active_since: now - (3600 * (k % 8 + 1)),
                last_job_started_at: Some(now - 3600),
                last_job_completed_at: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_since: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_minutes: idle_mins,
                last_error: None,
                created_at: now - 3600 * 24 * 5,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 4 (12 K8s Edge Ingress servers)
        for k in 1..=12i64 {
            let is_idle = k > 10;
            let idle_mins: i64 = if is_idle { (k - 10) * 8 } else { 0 };
            initial_servers.push(Server {
                id: format!("srv-k8s-{}", k),
                project_id: project_id_4.clone(),
                server_id: format!("i-k8s-{:04x}", k * 37),
                name: format!("k8s-edge-proxy-{:02}", k),
                provider: "azure".to_string(),
                resource_type: "vm".to_string(),
                host_ip: format!("20.199.{}.{}", (k * 19) % 250, (k * 23) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "eastus".to_string(),
                tags: vec!["k8s-ingress".to_string(), "envoy-edge".to_string()],
                status: if is_idle { "idle".to_string() } else { "active".to_string() },
                active_since: now - (3600 * (k % 5 + 1)),
                last_job_started_at: Some(now - 1800),
                last_job_completed_at: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_since: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_minutes: idle_mins,
                last_error: None,
                created_at: now - 3600 * 24 * 6,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 5 (16 ElasticSearch servers)
        for k in 1..=16i64 {
            let is_idle = k > 14;
            let idle_mins: i64 = if is_idle { (k - 14) * 6 } else { 0 };
            initial_servers.push(Server {
                id: format!("srv-search-{}", k),
                project_id: project_id_5.clone(),
                server_id: format!("i-es-{:04x}", k * 41),
                name: format!("es-indexer-node-{:02}", k),
                provider: "aws".to_string(),
                resource_type: "ec2".to_string(),
                host_ip: format!("54.212.{}.{}", (k * 11) % 250, (k * 31) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "us-west-2".to_string(),
                tags: vec!["opensearch".to_string(), "analytics-data".to_string()],
                status: if is_idle { "idle".to_string() } else { "active".to_string() },
                active_since: now - (3600 * (k % 10 + 1)),
                last_job_started_at: Some(now - 5400),
                last_job_completed_at: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_since: if is_idle { Some(now - idle_mins * 60) } else { None },
                idle_minutes: idle_mins,
                last_error: None,
                created_at: now - 3600 * 24 * 4,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 6 (10 GPU Rendering servers, Closed/Standby)
        for k in 1..=10i64 {
            initial_servers.push(Server {
                id: format!("srv-render-{}", k),
                project_id: project_id_6.clone(),
                server_id: format!("i-rnd-{:04x}", k * 47),
                name: format!("gpu-render-l4-{:02}", k),
                provider: "gcp".to_string(),
                resource_type: "ec2".to_string(),
                host_ip: format!("34.121.{}.{}", (k * 7) % 250, (k * 29) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "us-central1".to_string(),
                tags: vec!["gpu-diffusion".to_string(), "offline-batch".to_string()],
                status: "idle".to_string(),
                active_since: now - 3600 * 24 * 10,
                last_job_started_at: Some(now - 3600 * 48),
                last_job_completed_at: Some(now - 3600 * 46),
                idle_since: Some(now - 3600 * 46),
                idle_minutes: 2760,
                last_error: None,
                created_at: now - 3600 * 24 * 14,
                telemetry: None,
            });
        }

        // Seed initial servers for Project 7 (8 Log Archive servers, Closed/Complete)
        for k in 1..=8i64 {
            initial_servers.push(Server {
                id: format!("srv-archive-{}", k),
                project_id: project_id_7.clone(),
                server_id: format!("i-arch-{:04x}", k * 53),
                name: format!("gcs-sync-daemon-{:02}", k),
                provider: "gcp".to_string(),
                resource_type: "storage".to_string(),
                host_ip: format!("34.122.{}.{}", (k * 9) % 250, (k * 13) % 250),
                ssh_port: 22,
                ssh_user: "ubuntu".to_string(),
                region: "europe-west1".to_string(),
                tags: vec!["gcs-lake".to_string(), "log-archive".to_string()],
                status: "idle".to_string(),
                active_since: now - 3600 * 24 * 18,
                last_job_started_at: Some(now - 3600 * 72),
                last_job_completed_at: Some(now - 3600 * 70),
                idle_since: Some(now - 3600 * 70),
                idle_minutes: 4200,
                last_error: None,
                created_at: now - 3600 * 24 * 20,
                telemetry: None,
            });
        }

        // Seed initial notifications
        let initial_notifications = vec![
            FleetNotification {
                id: "notif-1".to_string(),
                project_id: project_id_1.clone(),
                r#type: "server_idle_alert".to_string(),
                severity: "warning".to_string(),
                title: "Server aws-worker-45 is Idle (>30m)".to_string(),
                message: "Node 54.210.135.65 has been idle for 60 minutes. Terminate to avoid excess charges.".to_string(),
                server_id: Some("i-002fd0573".to_string()),
                host_ip: Some("54.210.135.65".to_string()),
                idle_minutes: Some(60),
                read: false,
                created_at: now - 900,
            },
            FleetNotification {
                id: "notif-2".to_string(),
                project_id: project_id_1.clone(),
                r#type: "job_completed".to_string(),
                severity: "info".to_string(),
                title: "Parallel Execution Batch #12 Finished".to_string(),
                message: "32 of 32 active servers reported 0 exit code on run_worker.sh".to_string(),
                server_id: None,
                host_ip: None,
                idle_minutes: None,
                read: true,
                created_at: now - 3600,
            },
        ];

        let initial_cloud_providers = vec![
            CloudProviderConfig {
                id: "cp-aws".to_string(),
                provider: "aws".to_string(),
                name: "Amazon Web Services".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                account_id: Some("aws-act-849204128910".to_string()),
                default_region: "us-east-1".to_string(),
                credentials_masked: "AKIA••••••••••••39BA".to_string(),
                last_ping_ms: Some(34),
                last_tested_at: Some(now - 120),
            },
            CloudProviderConfig {
                id: "cp-azure".to_string(),
                provider: "azure".to_string(),
                name: "Microsoft Azure".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                account_id: Some("sub-azure-90284-e401".to_string()),
                default_region: "eastus".to_string(),
                credentials_masked: "c4b1••••••••••••7a9f".to_string(),
                last_ping_ms: Some(42),
                last_tested_at: Some(now - 180),
            },
            CloudProviderConfig {
                id: "cp-gcp".to_string(),
                provider: "gcp".to_string(),
                name: "Google Cloud Platform".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                account_id: Some("gcp-proj-fleet-prod-01".to_string()),
                default_region: "us-central1".to_string(),
                credentials_masked: "sa-fleet-agent@gcp-proj.iam.gserviceaccount.com".to_string(),
                last_ping_ms: Some(29),
                last_tested_at: Some(now - 90),
            },
        ];

        let initial_vcs_providers = vec![
            VcsProviderConfig {
                id: "vcs-github".to_string(),
                provider: "github".to_string(),
                name: "GitHub".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                username: Some("nakul-dev".to_string()),
                base_url: Some("https://github.com".to_string()),
                credentials_masked: "ghp_••••••••••••92ab".to_string(),
                auth_type: "token".to_string(),
                last_ping_ms: Some(28),
                last_tested_at: Some(now - 120),
            },
            VcsProviderConfig {
                id: "vcs-gitlab".to_string(),
                provider: "gitlab".to_string(),
                name: "GitLab".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                username: Some("devops-lead".to_string()),
                base_url: Some("https://gitlab.com".to_string()),
                credentials_masked: "glpat-••••••••••••38fc".to_string(),
                auth_type: "token".to_string(),
                last_ping_ms: Some(35),
                last_tested_at: Some(now - 180),
            },
            VcsProviderConfig {
                id: "vcs-bitbucket".to_string(),
                provider: "bitbucket".to_string(),
                name: "Atlassian Bitbucket".to_string(),
                status: "connected".to_string(),
                is_sandbox: true,
                username: Some("stackplane-ci".to_string()),
                base_url: Some("https://api.bitbucket.org/2.0".to_string()),
                credentials_masked: "ATBB••••••••••••881d".to_string(),
                auth_type: "token".to_string(),
                last_ping_ms: Some(44),
                last_tested_at: Some(now - 240),
            },
        ];

        Self {
            users: Arc::new(RwLock::new(initial_users)),
            projects: Arc::new(RwLock::new(vec![project_1, project_2, project_3, project_4, project_5, project_6, project_7])),
            servers: Arc::new(RwLock::new(initial_servers)),
            credentials: Arc::new(RwLock::new(vec![initial_credential])),
            cloud_providers: Arc::new(RwLock::new(initial_cloud_providers)),
            vcs_providers: Arc::new(RwLock::new(initial_vcs_providers)),
            notifications: Arc::new(RwLock::new(initial_notifications)),
            ws_tx,
            mongo_client,
        }
    }

    pub fn broadcast_event(&self, event: &WsEvent) {
        if let Ok(json) = serde_json::to_string(event) {
            let _ = self.ws_tx.send(json);
        }
    }
}
