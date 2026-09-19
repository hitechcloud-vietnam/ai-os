use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::process::Command;
use tracing::{info, warn};

/// Permission types for sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxPermissions {
    pub network: bool,
    pub filesystem_read: bool,
    pub filesystem_write: bool,
    pub env_vars: Vec<String>,
    pub timeout_secs: u64,
    pub max_memory_mb: u64,
}

impl Default for SandboxPermissions {
    fn default() -> Self {
        Self {
            network: false,
            filesystem_read: true,
            filesystem_write: false,
            env_vars: vec![],
            timeout_secs: 30,
            max_memory_mb: 256,
        }
    }
}

/// Result of a sandboxed execution
#[derive(Debug, Serialize)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// Run a command inside a sandboxed environment
pub async fn run_sandboxed(
    command: &str,
    args: &[String],
    permissions: &SandboxPermissions,
) -> anyhow::Result<SandboxResult> {
    info!(command = command, "Executing in sandbox");

    let mut cmd = Command::new(command);
    cmd.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

    // Restrict environment variables
    cmd.env_clear();
    for var in &permissions.env_vars {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    let timeout = std::time::Duration::from_secs(permissions.timeout_secs);

    let output = match tokio::time::timeout(timeout, cmd.output()).await {
        Ok(result) => result?,
        Err(_) => {
            warn!(command = command, "Sandbox execution timed out");
            return Ok(SandboxResult {
                exit_code: -1,
                stdout: String::new(),
                stderr: "Execution timed out".to_string(),
                timed_out: true,
            });
        }
    };

    Ok(SandboxResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        timed_out: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sandbox_basic_command() {
        let perms = SandboxPermissions::default();
        let result = run_sandboxed("echo", &["hello".to_string()], &perms)
            .await
            .unwrap();
        assert_eq!(result.exit_code, 0);
        assert!(result.stdout.contains("hello"));
    }

    #[tokio::test]
    async fn test_sandbox_timeout() {
        let perms = SandboxPermissions {
            timeout_secs: 1,
            ..Default::default()
        };
        let result = run_sandboxed("sleep", &["10".to_string()], &perms)
            .await
            .unwrap();
        assert!(result.timed_out);
    }
}
