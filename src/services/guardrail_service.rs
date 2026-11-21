use crate::models::{Project, Server};

pub struct GuardrailService;

impl GuardrailService {
    pub fn should_terminate_idle(server: &Server, timeout_minutes: i64) -> bool {
        server.status == "idle" && server.idle_minutes >= timeout_minutes
    }

    pub fn is_budget_exceeded(project: &Project, current_burn: f64) -> bool {
        if let Some(budget_cap) = project.budget_cap_usd {
            current_burn >= budget_cap
        } else {
            false
        }
    }
}
