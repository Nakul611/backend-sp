/// Seed metadata definitions for StackPlane initial boot

pub struct SeedConfig {
    pub default_admin_user: &'static str,
    pub demo_fleet_size: usize,
    pub enable_sandbox_defaults: bool,
}

pub const DEFAULT_SEED: SeedConfig = SeedConfig {
    default_admin_user: "user1234",
    demo_fleet_size: 50,
    enable_sandbox_defaults: true,
};
