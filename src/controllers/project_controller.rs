use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    error::AppError,
    models::{
        CreateProjectRequest, Project, ProjectRepository, ProjectService, Server,
        UpdateProjectStatusRequest,
    },
    state::AppState,
};

pub async fn list_projects(State(state): State<AppState>) -> Result<Json<Vec<Project>>, AppError> {
    let mut projects = state.projects.read().await.clone();
    let servers = state.servers.read().await;

    // Recalculate server metrics per project
    for project in projects.iter_mut() {
        let proj_servers: Vec<_> = servers.iter().filter(|s| s.project_id == project.id).collect();
        project.server_count = proj_servers.len() as i64;
        project.active_count = proj_servers
            .iter()
            .filter(|s| s.status == "active" || s.status == "executing")
            .count() as i64;
        project.idle_count = proj_servers.iter().filter(|s| s.status == "idle").count() as i64;
    }

    Ok(Json(projects))
}

pub async fn get_project(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Project>, AppError> {
    let projects = state.projects.read().await.clone();
    let servers = state.servers.read().await;

    let mut project = projects
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Project '{}' not found", id)))?;

    let proj_servers: Vec<_> = servers.iter().filter(|s| s.project_id == project.id).collect();
    project.server_count = proj_servers.len() as i64;
    project.active_count = proj_servers
        .iter()
        .filter(|s| s.status == "active" || s.status == "executing")
        .count() as i64;
    project.idle_count = proj_servers.iter().filter(|s| s.status == "idle").count() as i64;

    Ok(Json(project))
}

pub async fn create_project(
    State(state): State<AppState>,
    Json(payload): Json<CreateProjectRequest>,
) -> Result<(StatusCode, Json<Project>), AppError> {
    let now = Utc::now().timestamp();
    let project_id = format!("proj-{}", Uuid::new_v4().simple());
    let slug = payload
        .name
        .to_lowercase()
        .replace(|c: char| !c.is_alphanumeric(), "-");

    let p_type = payload.project_type.clone().unwrap_or_else(|| {
        if payload.services.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            "fullstack".to_string()
        } else if payload.pipeline_stages.as_ref().map(|s| !s.is_empty()).unwrap_or(false) {
            "mass_pipeline".to_string()
        } else {
            "fullstack".to_string()
        }
    });

    let mut generated_services: Vec<ProjectService> = Vec::new();
    let mut generated_servers: Vec<Server> = Vec::new();

    if p_type == "fullstack" {
        if let Some(user_services) = payload.services {
            for (idx, us) in user_services.into_iter().enumerate() {
                let service_id = format!("srv-{}-{}", slug, idx + 1);
                let node_count = us.node_count.unwrap_or(1).max(1);

                let is_database = us.service_type == "database";

                let service_git_url = if is_database {
                    None
                } else if let Some(ref g) = us.git_url {
                    let trimmed = g.trim();
                    if !trimmed.is_empty() {
                        Some(trimmed.to_string())
                    } else {
                        payload.git_url.as_ref().map(|s| s.trim().to_string())
                    }
                } else {
                    payload.git_url.as_ref().map(|s| s.trim().to_string())
                };

                let service_branch = if is_database {
                    None
                } else if let Some(ref b) = us.branch {
                    let trimmed = b.trim();
                    if !trimmed.is_empty() {
                        Some(trimmed.to_string())
                    } else {
                        payload.branch.as_ref().map(|s| s.trim().to_string())
                    }
                } else {
                    payload.branch.as_ref().map(|s| s.trim().to_string())
                };

                generated_services.push(ProjectService {
                    id: service_id.clone(),
                    name: us.name.clone(),
                    service_type: us.service_type.clone(),
                    git_url: service_git_url.clone(),
                    branch: service_branch.clone(),
                    root_directory: us.root_directory.clone(),
                    build_command: us.build_command.clone(),
                    run_command: us.run_command.clone(),
                    port: us.port,
                    database_engine: us.database_engine.clone(),
                    node_count,
                });

                // Allocate nodes for this service
                for n in 1..=node_count {
                    let s_id = format!("i-{}-{:02}", slug, n);
                    let ip = format!("54.210.{}.{}", (idx * 20 + n as usize * 3) % 250, (n as usize * 17) % 250);
                    let s_name = format!("{}-node-{:02}", us.name.to_lowercase().replace(' ', "-"), n);
                    let res_type = if us.service_type == "database" { "database".to_string() } else { "ec2".to_string() };
                    let mut tags = vec![us.service_type.clone(), "stackplane".to_string()];
                    if let Some(ref db) = us.database_engine {
                        tags.push(db.to_lowercase().replace(' ', "-"));
                    }
                    if let Some(ref gu) = service_git_url {
                        let repo_slug = gu.trim_end_matches(".git").split('/').last().unwrap_or("repo");
                        tags.push(format!("repo:{}", repo_slug));
                    }
                    generated_servers.push(Server {
                        id: format!("srv-{}", Uuid::new_v4().simple()),
                        project_id: project_id.clone(),
                        server_id: s_id,
                        name: s_name,
                        provider: "aws".to_string(),
                        resource_type: res_type,
                        service_id: Some(service_id.clone()),
                        role: Some(us.service_type.clone()),
                        service_name: Some(us.name.clone()),
                        database_engine: us.database_engine.clone(),
                        port: us.port,
                        host_ip: ip,
                        ssh_port: 22,
                        ssh_user: "ubuntu".to_string(),
                        region: "us-east-1".to_string(),
                        tags,
                        status: "active".to_string(),
                        active_since: now,
                        last_job_started_at: Some(now),
                        last_job_completed_at: None,
                        idle_since: None,
                        idle_minutes: 0,
                        last_error: None,
                        created_at: now,
                        telemetry: None,
                    });
                }
            }
        } else {
            // Default 4 fullstack services
            let default_stack = vec![
                ("Frontend Web App", "frontend", Some("https://github.com/my-org/web.git"), Some(3000), None, 1),
                ("Core API Backend", "backend", Some("https://github.com/my-org/api.git"), Some(8000), None, 1),
                ("PostgreSQL Database", "database", None, Some(5432), Some("PostgreSQL 16"), 1),
                ("Redis Cache", "database", None, Some(6379), Some("Redis 7.2"), 1),
            ];
            for (idx, (s_name, s_type, git, port, db, count)) in default_stack.into_iter().enumerate() {
                let service_id = format!("srv-{}-{}", slug, idx + 1);
                generated_services.push(ProjectService {
                    id: service_id.clone(),
                    name: s_name.to_string(),
                    service_type: s_type.to_string(),
                    git_url: git.map(|g| g.to_string()),
                    branch: Some("main".to_string()),
                    root_directory: None,
                    build_command: None,
                    run_command: None,
                    port,
                    database_engine: db.map(|d| d.to_string()),
                    node_count: count,
                });
                for n in 1..=count {
                    let s_id = format!("i-{}-{:02}", slug, n);
                    let ip = format!("54.210.{}.{}", (idx * 20 + n as usize * 3) % 250, (n as usize * 17) % 250);
                    let node_name = format!("{}-node-{:02}", s_name.to_lowercase().replace(' ', "-"), n);
                    let res_type = if s_type == "database" { "database".to_string() } else { "ec2".to_string() };
                    let mut tags = vec![s_type.to_string(), "stackplane".to_string()];
                    if let Some(d) = db { tags.push(d.to_lowercase().replace(' ', "-")); }
                    generated_servers.push(Server {
                        id: format!("srv-{}", Uuid::new_v4().simple()),
                        project_id: project_id.clone(),
                        server_id: s_id,
                        name: node_name,
                        provider: "aws".to_string(),
                        resource_type: res_type,
                        service_id: Some(service_id.clone()),
                        role: Some(s_type.to_string()),
                        service_name: Some(s_name.to_string()),
                        database_engine: db.map(|d| d.to_string()),
                        port,
                        host_ip: ip,
                        ssh_port: 22,
                        ssh_user: "ubuntu".to_string(),
                        region: "us-east-1".to_string(),
                        tags,
                        status: "active".to_string(),
                        active_since: now,
                        last_job_started_at: Some(now),
                        last_job_completed_at: None,
                        idle_since: None,
                        idle_minutes: 0,
                        last_error: None,
                        created_at: now,
                        telemetry: None,
                    });
                }
            }
        }
    } else {
        // mass_pipeline
        let stages = payload.pipeline_stages.unwrap_or_else(|| {
            vec!["scraping".to_string(), "curating".to_string(), "cleaning".to_string()]
        });
        for (idx, st) in stages.into_iter().enumerate() {
            let (name, s_type, count) = match st.as_str() {
                "scraping" => ("Mass Web Scrapers", "scraper", 4),
                "curating" => ("Data Curation & Normalization", "curator", 2),
                "cleaning" => ("Mass Data Cleaning & Deduplication", "cleaner", 2),
                other => (other, "worker", 2),
            };
            let service_id = format!("srv-{}-{}", slug, idx + 1);
            generated_services.push(ProjectService {
                id: service_id.clone(),
                name: name.to_string(),
                service_type: s_type.to_string(),
                git_url: payload.git_url.clone(),
                branch: payload.branch.clone().or_else(|| Some("main".to_string())),
                root_directory: None,
                build_command: None,
                run_command: payload.run_command.clone(),
                port: None,
                database_engine: None,
                node_count: count,
            });
            for n in 1..=count {
                let s_id = format!("i-{}-{:02}", slug, n);
                let ip = format!("54.210.{}.{}", (idx * 30 + n as usize * 5) % 250, (n as usize * 13) % 250);
                let node_name = format!("{}-worker-{:02}", s_type, n);
                generated_servers.push(Server {
                    id: format!("srv-{}", Uuid::new_v4().simple()),
                    project_id: project_id.clone(),
                    server_id: s_id,
                    name: node_name,
                    provider: "aws".to_string(),
                    resource_type: "ec2".to_string(),
                    service_id: Some(service_id.clone()),
                    role: Some(s_type.to_string()),
                    service_name: Some(name.to_string()),
                    database_engine: None,
                    port: None,
                    host_ip: ip,
                    ssh_port: 22,
                    ssh_user: "ubuntu".to_string(),
                    region: "us-east-1".to_string(),
                    tags: vec![s_type.to_string(), "distributed-pipeline".to_string()],
                    status: "active".to_string(),
                    active_since: now,
                    last_job_started_at: Some(now),
                    last_job_completed_at: None,
                    idle_since: None,
                    idle_minutes: 0,
                    last_error: None,
                    created_at: now,
                    telemetry: None,
                });
            }
        }
    }

    let repo = payload.git_url.as_ref().map(|url| ProjectRepository {
        git_url: url.clone(),
        branch: payload.branch.clone().unwrap_or_else(|| "main".to_string()),
        build_command: None,
        run_command: payload.run_command.clone().unwrap_or_else(|| "./run_job.sh".to_string()),
    });

    let srv_count = generated_servers.len() as i64;

    let repo_strat = payload.repo_strategy.clone().unwrap_or_else(|| {
        if generated_services.iter().any(|s| s.git_url.is_some() && s.git_url.as_deref() != payload.git_url.as_deref()) {
            "polyrepo".to_string()
        } else {
            "monorepo".to_string()
        }
    });

    let new_project = Project {
        id: project_id,
        name: payload.name,
        slug,
        description: payload.description.unwrap_or_default(),
        status: "open".to_string(),
        project_type: Some(p_type),
        repo_strategy: Some(repo_strat),
        services: Some(generated_services),
        domain_guidance: Some(
            "StackPlane provisions dedicated servers and databases with direct public IPs and root SSM access. We do not manage DNS domains or registrar subscriptions. To attach your custom domain, point your DNS A-record to the server IP and configure Nginx/Caddy SSL certificates directly via the built-in Terminal.".to_string(),
        ),
        repository: repo,
        default_credential_id: payload.credential_id,
        idle_alert_threshold_minutes: payload.idle_alert_threshold_minutes.unwrap_or(15),
        auto_stop_idle: false,
        budget_cap_usd: payload.budget_cap_usd.or(Some(10000.0)),
        budget_cadence: payload.budget_cadence.or_else(|| Some("monthly".to_string())),
        member_ids: vec!["member-admin".to_string()],
        server_count: srv_count,
        active_count: srv_count,
        idle_count: 0,
        created_at: now,
        updated_at: now,
    };

    let mut state_servers = state.servers.write().await;
    for srv in generated_servers {
        state_servers.push(srv);
    }

    let mut projects = state.projects.write().await;
    projects.push(new_project.clone());

    Ok((StatusCode::CREATED, Json(new_project)))
}

pub async fn update_project_status(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateProjectStatusRequest>,
) -> Result<Json<Project>, AppError> {
    let now = Utc::now().timestamp();
    let mut projects = state.projects.write().await;

    let project = projects
        .iter_mut()
        .find(|p| p.id == id)
        .ok_or_else(|| AppError::NotFound(format!("Project '{}' not found", id)))?;

    project.status = payload.status;
    project.updated_at = now;

    Ok(Json(project.clone()))
}
