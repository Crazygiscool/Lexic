#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GsetDiagnostic {
    pub message: String,
    pub raw: String,
}

#[derive(Debug, Clone)]
pub struct GsetRunResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub diagnostics: Vec<GsetDiagnostic>,
}

pub fn gset_binary() -> PathBuf {
    if let Ok(value) = std::env::var("GSET_BINARY") {
        return PathBuf::from(value);
    }
    for candidate in ["GSET/target/release/gset", "GSET/target/debug/gset"] {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return path;
        }
    }
    PathBuf::from("gset")
}

pub fn transpile(
    path: &Path,
    target: &str,
    source: Option<&str>,
    output: Option<&Path>,
    verbose: bool,
    experimental_api_map: bool,
) -> Result<GsetRunResult, String> {
    let mut command = Command::new(gset_binary());
    command.arg("transpile").arg(path);
    command.arg("--to").arg(target);
    if let Some(source) = source {
        command.arg("--from").arg(source);
    }
    if let Some(output) = output {
        command.arg("--output").arg(output);
    }
    if verbose {
        command.arg("--verbose");
    }
    if experimental_api_map {
        command.arg("--experimental-api-map");
    }

    let output = command.output().map_err(|error| {
        format!(
            "failed to execute GSET ({}): {}",
            gset_binary().display(),
            error
        )
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let diagnostics = parse_diagnostics(&stderr);
    Ok(GsetRunResult {
        success: output.status.success(),
        exit_code: output.status.code(),
        stdout,
        stderr,
        diagnostics,
    })
}

fn parse_diagnostics(stderr: &str) -> Vec<GsetDiagnostic> {
    stderr
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(GsetDiagnostic {
                    message: trimmed.to_string(),
                    raw: line.to_string(),
                })
            }
        })
        .collect()
}

pub fn is_available() -> bool {
    gset_binary().components().count() > 0
}
