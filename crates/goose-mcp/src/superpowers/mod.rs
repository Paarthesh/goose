use std::collections::HashMap;

use reqwest::Method;
use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, ErrorCode, ErrorData, Implementation, InitializeResult,
        ServerCapabilities, ServerInfo,
    },
    schemars::JsonSchema,
    tool, tool_handler, tool_router, ServerHandler,
};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

// ── Parameter structs ────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitLogParams {
    /// Absolute path to the repository root. Defaults to the current working directory.
    pub path: Option<String>,
    /// Maximum number of commits to return. Defaults to 20.
    pub limit: Option<u32>,
    /// Filter commits by author name or email (partial match).
    pub author: Option<String>,
    /// Show commits more recent than this date/relative-time, e.g. "2024-01-01" or "1 week ago".
    pub since: Option<String>,
    /// Show commits older than this date/relative-time.
    pub until: Option<String>,
    /// Restrict the log to commits that touched this file path.
    pub file: Option<String>,
    /// Output format: "oneline", "short", or "medium". Defaults to "short".
    pub format: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitDiffParams {
    /// Absolute path to the repository root. Defaults to the current working directory.
    pub path: Option<String>,
    /// Base ref or commit (e.g. "HEAD~3", a branch name, or a commit SHA).
    pub base: Option<String>,
    /// Head ref or commit. When omitted, compares base against the working tree.
    pub head: Option<String>,
    /// Restrict the diff to this file path.
    pub file: Option<String>,
    /// Show only the --stat summary (file change counts), not the full patch.
    pub stat_only: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GitBlameParams {
    /// File to blame, relative to the repository root.
    pub file: String,
    /// Absolute path to the repository root. Defaults to the current working directory.
    pub repo_path: Option<String>,
    /// First line to annotate (1-indexed, inclusive).
    pub start_line: Option<u32>,
    /// Last line to annotate (1-indexed, inclusive).
    pub end_line: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RipgrepSearchParams {
    /// Regular expression pattern to search for.
    pub pattern: String,
    /// Directory to search. Defaults to the current working directory.
    pub path: Option<String>,
    /// Restrict search to this ripgrep file type, e.g. "rust", "py", "js".
    pub file_type: Option<String>,
    /// Number of context lines to show before and after each match.
    pub context_lines: Option<u32>,
    /// Treat the pattern as a literal string rather than a regex.
    pub fixed_strings: bool,
    /// Make the search case-sensitive. Defaults to false (case-insensitive).
    pub case_sensitive: bool,
    /// Maximum number of matches to return. Defaults to 50.
    pub max_results: Option<u32>,
    /// Glob pattern to filter files, e.g. "*.rs" or "src/**/*.ts".
    pub glob: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
    HEAD,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HttpRequestParams {
    /// HTTP method.
    pub method: HttpMethod,
    /// Full URL including scheme, host, path, and query string.
    pub url: String,
    /// Optional request headers as key-value pairs.
    pub headers: Option<HashMap<String, String>>,
    /// Optional request body (for POST, PUT, PATCH). Pass pre-serialised JSON when needed.
    pub body: Option<String>,
    /// Request timeout in seconds. Defaults to 30.
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct JsonQueryParams {
    /// The JSON string to parse and query.
    pub json: String,
    /// RFC 6901 JSON Pointer to extract a value, e.g. "/users/0/name". Omit to return the whole document.
    pub pointer: Option<String>,
    /// Pretty-print the output. Defaults to false.
    pub pretty: bool,
    /// Return only the keys of the top-level object (or indices for arrays) instead of the full value.
    pub keys_only: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EnvInfoCommand {
    /// List environment variables, optionally filtered by a key prefix.
    EnvVars,
    /// Show OS name, hostname, username, and available CPU count.
    System,
    /// Show disk usage for a path (uses `df -h` on Unix, `dir` on Windows).
    DiskUsage,
    /// List running processes (uses `ps aux` on Unix, `tasklist` on Windows).
    Processes,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EnvironmentInfoParams {
    /// The information category to query.
    pub command: EnvInfoCommand,
    /// For EnvVars: only return variables whose name starts with this prefix.
    pub filter: Option<String>,
    /// For DiskUsage: the path to report on. Defaults to the current directory.
    pub path: Option<String>,
}

// ── Server ───────────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct SuperpowersServer {
    tool_router: ToolRouter<Self>,
    http_client: reqwest::Client,
}

impl Default for SuperpowersServer {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn internal_error(msg: impl Into<String>) -> ErrorData {
    ErrorData::new(ErrorCode::INTERNAL_ERROR, msg.into(), None)
}

async fn run_command(program: &str, args: &[&str], cwd: Option<&str>) -> Result<String, ErrorData> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| internal_error(format!("failed to launch `{}`: {}", program, e)))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        Err(internal_error(format!(
            "`{}` exited with {}: {}",
            program, output.status, stderr
        )))
    }
}

const MAX_BODY_BYTES: usize = 102_400; // 100 KB

fn truncate_if_needed(mut s: String, label: &str) -> String {
    if s.len() > MAX_BODY_BYTES {
        s.truncate(MAX_BODY_BYTES);
        s.push_str(&format!("\n[{} truncated at 100 KB]", label));
    }
    s
}

// ── Tool implementations ─────────────────────────────────────────────────────

#[tool_router(router = tool_router)]
impl SuperpowersServer {
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(concat!("goose-superpowers/", env!("CARGO_PKG_VERSION")))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("reqwest client should build");

        Self {
            tool_router: Self::tool_router(),
            http_client,
        }
    }

    /// Show the git commit history for a repository with structured filtering options.
    /// Returns formatted commit log output. Prefer this over raw shell `git log` when
    /// you need author/date/file filters or want structured params instead of flag strings.
    #[tool(
        name = "git_log",
        description = "Show the git commit history for a repository with structured filtering options. Returns formatted commit log output."
    )]
    pub async fn git_log(
        &self,
        params: Parameters<GitLogParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = p.path.as_deref().unwrap_or(&cwd);

        let limit = p.limit.unwrap_or(20);
        let format_flag = format!(
            "--format={}",
            match p.format.as_deref().unwrap_or("short") {
                "oneline" => "%h %s (%an, %ar)",
                "medium" => "%H%n%an <%ae>%n%ad%n%n    %s%n",
                _ => "%h - %s%n    %an <%ae>, %ar",
            }
        );
        let limit_flag = format!("-{}", limit);

        let mut args: Vec<&str> = vec!["log", &format_flag, &limit_flag];

        let author_flag;
        if let Some(ref a) = p.author {
            author_flag = format!("--author={}", a);
            args.push(&author_flag);
        }

        let since_flag;
        if let Some(ref s) = p.since {
            since_flag = format!("--since={}", s);
            args.push(&since_flag);
        }

        let until_flag;
        if let Some(ref u) = p.until {
            until_flag = format!("--until={}", u);
            args.push(&until_flag);
        }

        if let Some(ref file) = p.file {
            args.push("--");
            args.push(file.as_str());
        }

        let output = run_command("git", &args, Some(dir)).await?;
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    /// Show a git diff between refs or against the working tree.
    /// Use stat_only for a compact summary (file names + line counts) without the full patch.
    #[tool(
        name = "git_diff",
        description = "Show a git diff between refs or against the working tree. Use stat_only for a compact summary without the full patch."
    )]
    pub async fn git_diff(
        &self,
        params: Parameters<GitDiffParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = p.path.as_deref().unwrap_or(&cwd);

        let mut args: Vec<&str> = vec!["diff"];
        if p.stat_only {
            args.push("--stat");
        }

        let range;
        match (p.base.as_deref(), p.head.as_deref()) {
            (Some(base), Some(head)) => {
                range = format!("{}..{}", base, head);
                args.push(&range);
            }
            (Some(base), None) => {
                args.push(base);
            }
            _ => {}
        }

        if let Some(ref file) = p.file {
            args.push("--");
            args.push(file.as_str());
        }

        let output = run_command("git", &args, Some(dir)).await?;
        let text = if output.is_empty() {
            "(no differences)".to_string()
        } else {
            truncate_if_needed(output, "diff")
        };
        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    /// Show line-by-line git blame annotations for a file, optionally restricted to a line range.
    #[tool(
        name = "git_blame",
        description = "Show line-by-line git blame annotations for a file, optionally restricted to a line range."
    )]
    pub async fn git_blame(
        &self,
        params: Parameters<GitBlameParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;
        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = p.repo_path.as_deref().unwrap_or(&cwd);

        let mut args: Vec<&str> = vec!["blame"];

        let range_flag;
        match (p.start_line, p.end_line) {
            (Some(start), Some(end)) => {
                range_flag = format!("-L {},{}", start, end);
                args.push(&range_flag);
            }
            (Some(start), None) => {
                range_flag = format!("-L {}", start);
                args.push(&range_flag);
            }
            _ => {}
        }

        args.push(p.file.as_str());

        let output = run_command("git", &args, Some(dir)).await?;
        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    /// Search for a pattern across files using ripgrep (rg). Returns structured results with
    /// file paths, line numbers, and match context. Prefer this over raw `rg` shell commands
    /// when you need consistent output formatting or parameterised filtering.
    #[tool(
        name = "ripgrep_search",
        description = "Search for a pattern across files using ripgrep. Returns file paths, line numbers, and match context."
    )]
    pub async fn ripgrep_search(
        &self,
        params: Parameters<RipgrepSearchParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;

        // Verify rg is available before building args
        which::which("rg").map_err(|_| {
            internal_error(
                "ripgrep (`rg`) is not installed or not in PATH. \
                 Install ripgrep (https://github.com/BurntSushi/ripgrep) \
                 or use the developer shell extension with grep instead.",
            )
        })?;

        let cwd = std::env::current_dir()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_default();

        let mut args: Vec<String> = Vec::new();
        // Always emit line numbers for structured output
        args.push("--line-number".into());

        if p.fixed_strings {
            args.push("--fixed-strings".into());
        }
        if !p.case_sensitive {
            args.push("--ignore-case".into());
        }
        if let Some(ctx) = p.context_lines {
            args.push(format!("--context={}", ctx));
        }
        if let Some(ref ft) = p.file_type {
            args.push(format!("--type={}", ft));
        }
        if let Some(ref g) = p.glob {
            args.push(format!("--glob={}", g));
        }
        let max = p.max_results.unwrap_or(50);
        args.push(format!("--max-count={}", max));

        args.push(p.pattern.clone());
        args.push(p.path.as_deref().unwrap_or(&cwd).to_string());

        let str_args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();

        let mut cmd = Command::new("rg");
        cmd.args(&str_args);

        let output = cmd
            .output()
            .await
            .map_err(|e| internal_error(format!("failed to launch rg: {}", e)))?;

        let text = if output.status.code() == Some(1) {
            // exit code 1 means no matches — that's fine
            "(no matches found)".to_string()
        } else if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(internal_error(format!("rg failed: {}", stderr)));
        } else {
            let raw = String::from_utf8_lossy(&output.stdout).into_owned();
            truncate_if_needed(raw, "ripgrep output")
        };

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }

    /// Perform a full HTTP request (GET, POST, PUT, PATCH, DELETE, HEAD) with custom headers
    /// and an optional body. Returns status code, response headers, and the response body.
    /// Unlike computercontroller's web_scrape, this supports all methods and inline responses.
    #[tool(
        name = "http_request",
        description = "Perform an HTTP request (any method) with custom headers and body. Returns status, headers, and response body."
    )]
    pub async fn http_request(
        &self,
        params: Parameters<HttpRequestParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;

        let method = match p.method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::HEAD => Method::HEAD,
        };

        let timeout = std::time::Duration::from_secs(p.timeout_secs.unwrap_or(30));
        let mut builder = self.http_client.request(method, &p.url).timeout(timeout);

        if let Some(headers) = p.headers {
            for (k, v) in headers {
                let name = reqwest::header::HeaderName::from_bytes(k.as_bytes())
                    .map_err(|e| internal_error(format!("invalid header name `{}`: {}", k, e)))?;
                let value = reqwest::header::HeaderValue::from_str(&v).map_err(|e| {
                    internal_error(format!("invalid header value for `{}`: {}", k, e))
                })?;
                builder = builder.header(name, value);
            }
        }

        if let Some(body) = p.body {
            builder = builder.body(body);
        }

        let resp = builder
            .send()
            .await
            .map_err(|e| internal_error(format!("request failed: {}", e)))?;

        let status = resp.status();
        let mut summary = format!(
            "HTTP {} {}\n",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );

        for (name, value) in resp.headers() {
            summary.push_str(&format!(
                "{}: {}\n",
                name,
                value.to_str().unwrap_or("<binary>")
            ));
        }
        summary.push('\n');

        let body_bytes = resp
            .bytes()
            .await
            .map_err(|e| internal_error(format!("failed to read response body: {}", e)))?;
        let body = String::from_utf8_lossy(&body_bytes).into_owned();
        summary.push_str(&truncate_if_needed(body, "response body"));

        Ok(CallToolResult::success(vec![Content::text(summary)]))
    }

    /// Parse a JSON string and optionally extract a value using an RFC 6901 JSON Pointer
    /// (e.g. "/users/0/name"). Returns a pretty-printed or compact JSON result.
    #[tool(
        name = "json_query",
        description = "Parse a JSON string and optionally extract a value using an RFC 6901 JSON Pointer (e.g. /users/0/name)."
    )]
    pub async fn json_query(
        &self,
        params: Parameters<JsonQueryParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;

        let value: serde_json::Value = serde_json::from_str(&p.json)
            .map_err(|e| internal_error(format!("invalid JSON: {}", e)))?;

        let target = if let Some(ref ptr) = p.pointer {
            value
                .pointer(ptr)
                .ok_or_else(|| internal_error(format!("JSON pointer '{}' not found", ptr)))?
                .clone()
        } else {
            value
        };

        let result = if p.keys_only {
            match &target {
                serde_json::Value::Object(map) => {
                    let keys: Vec<&str> = map.keys().map(|k| k.as_str()).collect();
                    serde_json::to_string_pretty(&keys)
                        .map_err(|e| internal_error(e.to_string()))?
                }
                serde_json::Value::Array(arr) => {
                    let indices: Vec<usize> = (0..arr.len()).collect();
                    serde_json::to_string_pretty(&indices)
                        .map_err(|e| internal_error(e.to_string()))?
                }
                _ => {
                    return Err(internal_error(
                        "keys_only is only valid for JSON objects and arrays",
                    ))
                }
            }
        } else if p.pretty {
            serde_json::to_string_pretty(&target).map_err(|e| internal_error(e.to_string()))?
        } else {
            serde_json::to_string(&target).map_err(|e| internal_error(e.to_string()))?
        };

        Ok(CallToolResult::success(vec![Content::text(result)]))
    }

    /// Query environment variables, system facts, disk usage, or running processes.
    /// Provides a structured alternative to ad-hoc shell commands for common inspection needs.
    #[tool(
        name = "environment_info",
        description = "Query environment variables, system facts, disk usage, or running processes."
    )]
    pub async fn environment_info(
        &self,
        params: Parameters<EnvironmentInfoParams>,
    ) -> Result<CallToolResult, ErrorData> {
        let p = params.0;

        let text = match p.command {
            EnvInfoCommand::EnvVars => {
                let prefix = p.filter.as_deref().unwrap_or("").to_uppercase();
                let mut vars: Vec<(String, String)> = std::env::vars()
                    .filter(|(k, _)| prefix.is_empty() || k.to_uppercase().starts_with(&prefix))
                    .collect();
                vars.sort_by(|a, b| a.0.cmp(&b.0));
                vars.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            }

            EnvInfoCommand::System => {
                let os = std::env::consts::OS;
                let arch = std::env::consts::ARCH;
                let cpus = std::thread::available_parallelism()
                    .map(|n| n.to_string())
                    .unwrap_or_else(|_| "unknown".into());
                let user = std::env::var("USER")
                    .or_else(|_| std::env::var("USERNAME"))
                    .unwrap_or_else(|_| "unknown".into());
                let hostname = run_command("hostname", &[], None)
                    .await
                    .unwrap_or_else(|_| "unknown".into())
                    .trim()
                    .to_string();

                format!(
                    "OS: {}\nArch: {}\nHostname: {}\nUser: {}\nCPUs: {}",
                    os, arch, hostname, user, cpus
                )
            }

            EnvInfoCommand::DiskUsage => {
                let target = p.path.as_deref().unwrap_or(".");

                #[cfg(unix)]
                {
                    run_command("df", &["-h", target], None).await?
                }
                #[cfg(windows)]
                {
                    run_command("cmd", &["/C", &format!("dir {}", target)], None).await?
                }
                #[cfg(not(any(unix, windows)))]
                {
                    return Err(internal_error(
                        "DiskUsage is not supported on this platform",
                    ));
                }
            }

            EnvInfoCommand::Processes => {
                #[cfg(unix)]
                {
                    let raw = run_command("ps", &["aux"], None).await?;
                    // Keep the header + first 60 process lines to avoid massive output
                    let lines: Vec<&str> = raw.lines().take(61).collect();
                    let truncated = lines.len() < raw.lines().count();
                    let mut out = lines.join("\n");
                    if truncated {
                        out.push_str("\n[process list truncated at 60 entries]");
                    }
                    out
                }
                #[cfg(windows)]
                {
                    run_command("tasklist", &[], None).await?
                }
                #[cfg(not(any(unix, windows)))]
                {
                    return Err(internal_error(
                        "Processes is not supported on this platform",
                    ));
                }
            }
        };

        Ok(CallToolResult::success(vec![Content::text(text)]))
    }
}

// ── ServerHandler ────────────────────────────────────────────────────────────

#[tool_handler(router = self.tool_router)]
impl ServerHandler for SuperpowersServer {
    fn get_info(&self) -> ServerInfo {
        InitializeResult::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::new(
                "goose-superpowers",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Use the superpowers extension for structured developer productivity tools:\n\
                 - git_log / git_diff / git_blame: structured git operations with typed params\n\
                 - ripgrep_search: fast code search with filtering by type, glob, and context lines\n\
                 - http_request: full HTTP requests (all methods, custom headers, request bodies)\n\
                 - json_query: parse JSON and extract values via RFC 6901 pointers\n\
                 - environment_info: env vars, system facts, disk usage, and process list\n\
                 Prefer these tools over raw shell commands when you need structured output \
                 or have complex query parameters.",
            )
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_server_creation() {
        let server = SuperpowersServer::new();
        let info = server.get_info();
        assert_eq!(info.server_info.name, "goose-superpowers");
        assert!(info.instructions.is_some());
        assert!(info.instructions.unwrap().contains("superpowers"));
    }

    #[tokio::test]
    async fn test_json_query_basic() {
        let server = SuperpowersServer::new();
        let result = server
            .json_query(Parameters(JsonQueryParams {
                json: r#"{"a": {"b": 42}}"#.to_string(),
                pointer: Some("/a/b".to_string()),
                pretty: false,
                keys_only: false,
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert_eq!(text.trim(), "42");
    }

    #[tokio::test]
    async fn test_json_query_pretty() {
        let server = SuperpowersServer::new();
        let result = server
            .json_query(Parameters(JsonQueryParams {
                json: r#"{"x":1}"#.to_string(),
                pointer: None,
                pretty: true,
                keys_only: false,
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert!(text.contains('\n')); // pretty-printed
    }

    #[tokio::test]
    async fn test_json_query_keys_only() {
        let server = SuperpowersServer::new();
        let result = server
            .json_query(Parameters(JsonQueryParams {
                json: r#"{"foo":1,"bar":2}"#.to_string(),
                pointer: None,
                pretty: false,
                keys_only: true,
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert!(text.contains("foo") && text.contains("bar"));
    }

    #[tokio::test]
    async fn test_json_query_invalid_json() {
        let server = SuperpowersServer::new();
        let result = server
            .json_query(Parameters(JsonQueryParams {
                json: "not-json".to_string(),
                pointer: None,
                pretty: false,
                keys_only: false,
            }))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("invalid JSON"));
    }

    #[tokio::test]
    async fn test_json_query_pointer_not_found() {
        let server = SuperpowersServer::new();
        let result = server
            .json_query(Parameters(JsonQueryParams {
                json: r#"{"a": 1}"#.to_string(),
                pointer: Some("/nonexistent".to_string()),
                pretty: false,
                keys_only: false,
            }))
            .await;
        assert!(result.is_err());
        assert!(result.unwrap_err().message.contains("not found"));
    }

    #[tokio::test]
    async fn test_environment_info_env_vars() {
        let server = SuperpowersServer::new();
        let result = server
            .environment_info(Parameters(EnvironmentInfoParams {
                command: EnvInfoCommand::EnvVars,
                filter: Some("PATH".to_string()),
                path: None,
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert!(text.contains("PATH"));
    }

    #[tokio::test]
    async fn test_environment_info_system() {
        let server = SuperpowersServer::new();
        let result = server
            .environment_info(Parameters(EnvironmentInfoParams {
                command: EnvInfoCommand::System,
                filter: None,
                path: None,
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert!(text.contains("OS:"));
    }

    #[tokio::test]
    async fn test_git_log_in_repo() {
        let server = SuperpowersServer::new();
        let result = server
            .git_log(Parameters(GitLogParams {
                path: Some(env!("CARGO_MANIFEST_DIR").to_string()),
                limit: Some(3),
                author: None,
                since: None,
                until: None,
                file: None,
                format: Some("oneline".to_string()),
            }))
            .await;
        assert!(result.is_ok());
        let text = result.unwrap().content[0].as_text().unwrap().text.clone();
        assert!(!text.is_empty());
    }

    #[tokio::test]
    async fn test_git_diff_stat_only() {
        let server = SuperpowersServer::new();
        let result = server
            .git_diff(Parameters(GitDiffParams {
                path: Some(env!("CARGO_MANIFEST_DIR").to_string()),
                base: Some("HEAD~1".to_string()),
                head: Some("HEAD".to_string()),
                file: None,
                stat_only: true,
            }))
            .await;
        // May succeed or fail depending on git history; just check it doesn't panic
        let _ = result;
    }
}
