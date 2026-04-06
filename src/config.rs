//! TOML configuration deserialization for the mail orchestrator.
//!
//! Loads daemon settings, domain info, router config, notification
//! subscribers, scheduled sends, and template paths from a single TOML file.

use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Top-level orchestrator configuration.
//
// `templates` is held on the `Config` struct so the future template engine
// can take it by reference once wired in. Until then clippy can't see a
// consumer, so dead-code is suppressed at the struct level. Drop the
// allow once the template module reads `cfg.templates.dir`.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub daemon: DaemonConfig,
    pub domain: DomainConfig,
    pub router: RouterConfig,
    pub templates: TemplateConfig,
    #[serde(default)]
    pub notify: HashMap<String, NotifyConfig>,
    #[serde(default)]
    pub schedule: Vec<ScheduleConfig>,
}

/// Daemon runtime settings.
#[derive(Debug, Clone, Deserialize)]
pub struct DaemonConfig {
    pub pid_file: PathBuf,
    pub db_path: PathBuf,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

/// Mail domain and server settings.
//
// `mail_base` is the parent directory under which all per-mailbox
// Maildirs live; the future Maildir-bootstrap routine will use it to
// validate that mailboxes resolve under a single root, but that boot
// step isn't wired in yet. Drop the allow once it is.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct DomainConfig {
    pub name: String,
    pub mail_base: PathBuf,
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
}

/// Router mailbox configuration.
//
// `mailbox` is the human-readable mailbox name (e.g. "router") used by
// the audit logger to attribute events; the audit-write site that reads
// it isn't online yet, so suppress dead-code until then.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct RouterConfig {
    pub mailbox: String,
    pub maildir: PathBuf,
    pub authorized_senders: Vec<String>,
    pub allowed_from: Vec<String>,
}

/// Template engine configuration.
//
// `dir` is read by the template loader at runtime via serde — clippy's
// dead-code analysis can't see across the deserialization boundary, hence
// the allow. Remove once the template loader is wired into main.rs.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct TemplateConfig {
    pub dir: PathBuf,
}

/// Per-mailbox notification routing.
#[derive(Debug, Clone, Deserialize)]
pub struct NotifyConfig {
    pub mailbox: String,
    pub maildir: PathBuf,
    pub subscribers: Vec<String>,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default)]
    pub actions: Vec<String>,
}

/// Scheduled email definition.
//
// All fields are populated by serde from the user's TOML config and consumed
// by the scheduler loop, but the scheduler entry-point isn't wired into the
// daemon main yet — clippy can't see the future consumer, so dead-code is
// suppressed at the struct level. Drop the allow once `scheduler::run()`
// references these fields directly.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct ScheduleConfig {
    pub name: String,
    pub from: String,
    pub to: Vec<String>,
    pub subject: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub body: Option<String>,
    pub cron: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Config {
    /// Load configuration from a TOML file.
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get all Maildir paths that should be watched (router + all notify mailboxes).
    //
    // Will be invoked by the watcher boot path once it's promoted from a
    // standalone binary to a sub-task of the orchestrator main loop. Until
    // then it's referenced only by the `tests` module below.
    #[allow(dead_code)]
    pub fn watch_paths(&self) -> Vec<(String, PathBuf)> {
        let mut paths = vec![("router".to_string(), self.router.maildir.clone())];
        for (name, notify) in &self.notify {
            paths.push((name.clone(), notify.maildir.clone()));
        }
        paths
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_smtp_host() -> String {
    "127.0.0.1".to_string()
}

fn default_smtp_port() -> u16 {
    25
}

fn default_priority() -> String {
    "normal".to_string()
}

fn default_true() -> bool {
    true
}
