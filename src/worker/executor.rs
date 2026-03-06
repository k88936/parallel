use agent_client_protocol as acp;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info, instrument, trace, warn};

#[derive(Debug)]
struct RunningTerminal {
    process: Mutex<tokio::process::Child>,
    output: Arc<RwLock<String>>,
}

#[derive(Debug)]
pub struct Executor {
    workdir: PathBuf,
    terminals: RwLock<HashMap<acp::TerminalId, Arc<RunningTerminal>>>,
}

impl Executor {
    #[instrument(skip_all, fields(workdir = %workdir.display()))]
    pub fn new(workdir: PathBuf) -> Self {
        debug!("Creating new TaskClient");
        Self {
            workdir,
            terminals: RwLock::new(HashMap::new()),
        }
    }

    fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        let resolved = if path.is_absolute() {
            path.clone()
        } else {
            self.workdir.join(path)
        };
        trace!(requested = %path.display(), resolved = %resolved.display(), "Path resolved");
        resolved
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for Executor {
    #[instrument(skip(self), fields(session_id = %args.session_id))]
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        let options_count = args.options.len();
        debug!(
            tool_call_id = ?args.tool_call.tool_call_id,
            options_count = options_count,
            "Permission request received"
        );

        for (i, opt) in args.options.iter().enumerate() {
            trace!(
                option_index = i,
                option_id = %opt.option_id,
                option_name = %opt.name,
                option_kind = ?opt.kind,
                "Permission option available"
            );
        }

        let option = args.options.first();
        let outcome = if let Some(opt) = option {
            info!(
                selected_option_id = %opt.option_id,
                selected_option_name = %opt.name,
                "Auto-approving permission request"
            );
            acp::RequestPermissionOutcome::Selected(
                acp::SelectedPermissionOutcome::new(opt.option_id.clone())
            )
        } else {
            warn!("No options available, cancelling permission request");
            acp::RequestPermissionOutcome::Cancelled
        };

        Ok(acp::RequestPermissionResponse::new(outcome))
    }

    #[instrument(skip(self), fields(session_id = %args.session_id))]
    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        match &args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                if let acp::ContentBlock::Text(text) = content {
                    let preview = if text.text.len() > 50 {
                        format!("{}...", &text.text[..50])
                    } else {
                        text.text.clone()
                    };
                    trace!(content_preview = %preview, "Agent message chunk");
                    print!("{}", text.text);
                }
            }
            acp::SessionUpdate::ToolCall(update) => {
                info!(
                    tool_call_id = %update.tool_call_id,
                    "Tool call initiated"
                );
            }
            acp::SessionUpdate::ToolCallUpdate(update) => {
                debug!(
                    tool_call_id = %update.tool_call_id,
                    "Tool call update"
                );
            }
            acp::SessionUpdate::Plan(plan) => {
                info!(
                    plan_entries = plan.entries.len(),
                    "Agent execution plan received"
                );
                for (i, entry) in plan.entries.iter().enumerate() {
                    debug!(
                        entry_index = i,
                        entry_content = ?entry.content,
                        "Plan entry"
                    );
                }
            }
            acp::SessionUpdate::CurrentModeUpdate(update) => {
                info!(mode_id = %update.current_mode_id, "Session mode changed");
            }
            acp::SessionUpdate::AvailableCommandsUpdate(update) => {
                debug!(commands_count = update.available_commands.len(), "Available commands updated");
            }
            acp::SessionUpdate::ConfigOptionUpdate(update) => {
                debug!(options_count = update.config_options.len(), "Config options updated");
            }
            acp::SessionUpdate::AgentThoughtChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let preview: String = text.text.chars().take(50).collect();
                    trace!(thought_preview = %preview, "Agent thought");
                }
            }
            acp::SessionUpdate::UserMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    let preview: String = text.text.chars().take(50).collect();
                    trace!(message_preview = %preview, "User message");
                }
            }
            _ => {
                trace!("Unhandled session update type");
            }
        }
        Ok(())
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, path = %args.path.display()))]
    async fn write_text_file(
        &self,
        args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        let path = self.resolve_path(&args.path);
        let content_len = args.content.len();

        debug!(
            file = %path.display(),
            bytes = content_len,
            "Writing text file"
        );

        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                warn!(parent = %parent.display(), error = %e, "Failed to create parent directory");
            }
        }

        let start = std::time::Instant::now();
        match tokio::fs::write(&path, &args.content).await {
            Ok(()) => {
                info!(
                    file = %path.display(),
                    bytes = content_len,
                    elapsed_ms = start.elapsed().as_millis(),
                    "File written successfully"
                );
                Ok(acp::WriteTextFileResponse::new())
            }
            Err(e) => {
                error!(
                    file = %path.display(),
                    bytes = content_len,
                    error = %e,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Failed to write file"
                );
                Err(acp::Error::invalid_params().data(format!("Failed to write file: {}", e)))
            }
        }
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, path = %args.path.display()))]
    async fn read_text_file(
        &self,
        args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        let path = self.resolve_path(&args.path);
        let line_info = args.line.map(|l| format!(" from line {}", l)).unwrap_or_default();
        let limit_info = args.limit.map(|l| format!(" limit {}", l)).unwrap_or_default();

        debug!(file = %path.display(), "Reading text file{}{}", line_info, limit_info);

        let start = std::time::Instant::now();
        match tokio::fs::read_to_string(&path).await {
            Ok(content) => {
                let len = content.len();
                let lines = content.lines().count();
                info!(
                    file = %path.display(),
                    bytes = len,
                    lines = lines,
                    elapsed_ms = start.elapsed().as_millis(),
                    "File read successfully"
                );
                Ok(acp::ReadTextFileResponse::new(content))
            }
            Err(e) => {
                error!(
                    file = %path.display(),
                    error = %e,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Failed to read file"
                );
                Err(acp::Error::invalid_params().data(format!("Failed to read file: {}", e)))
            }
        }
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, command = %args.command))]
    async fn create_terminal(
        &self,
        args: acp::CreateTerminalRequest,
    ) -> acp::Result<acp::CreateTerminalResponse> {
        let terminal_id = acp::TerminalId::new(uuid::Uuid::new_v4().to_string());
        let cwd = args.cwd.as_ref().map(|p| self.resolve_path(p));
        let full_command = if args.args.is_empty() {
            args.command.clone()
        } else {
            format!("{} {}", args.command, args.args.join(" "))
        };

        debug!(
            terminal_id = %terminal_id,
            command = %full_command,
            cwd = ?cwd.as_ref().map(|p| p.display()),
            env_count = args.env.len(),
            "Creating terminal"
        );

        let mut cmd = tokio::process::Command::new(&args.command);
        cmd.args(&args.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true);

        if let Some(cwd) = &cwd {
            cmd.current_dir(cwd);
        }

        if !args.env.is_empty() {
            let envs: Vec<(String, String)> = args
                .env
                .iter()
                .map(|e| {
                    trace!(env_name = %e.name, "Setting environment variable");
                    (e.name.clone(), e.value.clone())
                })
                .collect();
            cmd.envs(envs);
        }

        let start = std::time::Instant::now();
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                error!(
                    command = %full_command,
                    error = %e,
                    "Failed to spawn terminal command"
                );
                return Err(acp::Error::invalid_params()
                    .data(format!("Failed to spawn command: {}", e)));
            }
        };

        let pid = child.id();
        info!(
            terminal_id = %terminal_id,
            pid = ?pid,
            command = %full_command,
            elapsed_ms = start.elapsed().as_millis(),
            "Terminal process spawned"
        );

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let output: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
        let output_for_task = output.clone();
        let terminal_id_for_task = terminal_id.to_string();

        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stdout = stdout;
            let mut stderr = stderr;
            let mut stdout_buf = [0u8; 1024];
            let mut stderr_buf = [0u8; 1024];

            trace!(terminal_id = %terminal_id_for_task, "Output capture task started");

            loop {
                tokio::select! {
                    Ok(n) = stdout.read(&mut stdout_buf) => {
                        if n == 0 {
                            trace!(terminal_id = %terminal_id_for_task, "stdout EOF");
                            break;
                        }
                        let s = String::from_utf8_lossy(&stdout_buf[..n]);
                        trace!(terminal_id = %terminal_id_for_task, bytes = n, "stdout received");
                        let mut out = output_for_task.write().await;
                        out.push_str(&s);
                    }
                    Ok(n) = stderr.read(&mut stderr_buf) => {
                        if n == 0 {
                            trace!(terminal_id = %terminal_id_for_task, "stderr EOF");
                            break;
                        }
                        let s = String::from_utf8_lossy(&stderr_buf[..n]);
                        trace!(terminal_id = %terminal_id_for_task, bytes = n, "stderr received");
                        let mut out = output_for_task.write().await;
                        out.push_str(&s);
                    }
                    else => break,
                }
            }

            debug!(terminal_id = %terminal_id_for_task, "Output capture task finished");
        });

        let terminal = Arc::new(RunningTerminal {
            process: Mutex::new(child),
            output,
        });

        let mut terminals = self.terminals.write().await;
        terminals.insert(terminal_id.clone(), terminal);
        debug!(terminal_id = %terminal_id, active_terminals = terminals.len(), "Terminal registered");

        Ok(acp::CreateTerminalResponse::new(terminal_id))
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, terminal_id = %args.terminal_id))]
    async fn terminal_output(
        &self,
        args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        trace!("Fetching terminal output");

        let terminals = self.terminals.read().await;
        let terminal = terminals
            .get(&args.terminal_id)
            .ok_or_else(|| {
                error!(terminal_id = %args.terminal_id, "Terminal not found");
                acp::Error::invalid_params().data("Terminal not found")
            })?;

        let output = terminal.output.read().await.clone();
        let output_len = output.len();

        let exit_status = {
            let mut process = terminal.process.lock().await;
            if let Ok(Some(status)) = process.try_wait() {
                let code = status.code();
                debug!(
                    terminal_id = %args.terminal_id,
                    exit_code = code,
                    output_bytes = output_len,
                    "Process has exited"
                );
                Some(acp::TerminalExitStatus::new().exit_code(code.unwrap_or(-1) as u32))
            } else {
                trace!(
                    terminal_id = %args.terminal_id,
                    output_bytes = output_len,
                    "Process still running"
                );
                None
            }
        };

        debug!(
            terminal_id = %args.terminal_id,
            output_bytes = output_len,
            has_exited = exit_status.is_some(),
            "Terminal output retrieved"
        );

        Ok(acp::TerminalOutputResponse::new(output, false).exit_status(exit_status))
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, terminal_id = %args.terminal_id))]
    async fn release_terminal(
        &self,
        args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        debug!("Releasing terminal");

        let mut terminals = self.terminals.write().await;
        if let Some(terminal) = terminals.remove(&args.terminal_id) {
            let mut process = terminal.process.lock().await;
            let pid = process.id();
            match process.kill().await {
                Ok(()) => info!(terminal_id = %args.terminal_id, pid = ?pid, "Terminal process killed and released"),
                Err(e) => warn!(terminal_id = %args.terminal_id, pid = ?pid, error = %e, "Failed to kill process during release"),
            }
        } else {
            warn!(terminal_id = %args.terminal_id, "Terminal not found for release");
        }

        debug!(active_terminals = terminals.len(), "Terminal released");

        Ok(acp::ReleaseTerminalResponse::new())
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, terminal_id = %args.terminal_id))]
    async fn wait_for_terminal_exit(
        &self,
        args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        debug!("Waiting for terminal exit");

        let terminals = self.terminals.read().await;
        let terminal = terminals
            .get(&args.terminal_id)
            .ok_or_else(|| {
                error!(terminal_id = %args.terminal_id, "Terminal not found");
                acp::Error::invalid_params().data("Terminal not found")
            })?;

        let mut process = terminal.process.lock().await;
        let pid = process.id();

        let start = std::time::Instant::now();
        let status = process.wait().await;
        let elapsed = start.elapsed();

        let exit_status = match status {
            Ok(s) => {
                let code = s.code();
                info!(
                    terminal_id = %args.terminal_id,
                    pid = ?pid,
                    exit_code = code,
                    elapsed_ms = elapsed.as_millis(),
                    "Process exited"
                );
                acp::TerminalExitStatus::new().exit_code(code.unwrap_or(-1) as u32)
            }
            Err(e) => {
                error!(
                    terminal_id = %args.terminal_id,
                    pid = ?pid,
                    error = %e,
                    "Failed to wait for process"
                );
                acp::TerminalExitStatus::new()
            }
        };

        Ok(acp::WaitForTerminalExitResponse::new(exit_status))
    }

    #[instrument(skip(self), fields(session_id = %args.session_id, terminal_id = %args.terminal_id))]
    async fn kill_terminal_command(
        &self,
        args: acp::KillTerminalCommandRequest,
    ) -> acp::Result<acp::KillTerminalCommandResponse> {
        debug!("Killing terminal command");

        let terminals = self.terminals.read().await;
        if let Some(terminal) = terminals.get(&args.terminal_id) {
            let mut process = terminal.process.lock().await;
            let pid = process.id();
            match process.kill().await {
                Ok(()) => info!(terminal_id = %args.terminal_id, pid = ?pid, "Terminal process killed"),
                Err(e) => warn!(terminal_id = %args.terminal_id, pid = ?pid, error = %e, "Failed to kill process"),
            }
        } else {
            warn!(terminal_id = %args.terminal_id, "Terminal not found for kill");
        }

        Ok(acp::KillTerminalCommandResponse::new())
    }

    async fn ext_method(&self, args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        warn!(method = %args.method, "Extension method not supported");
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, args: acp::ExtNotification) -> acp::Result<()> {
        warn!(method = %args.method, "Extension notification not supported");
        Ok(())
    }
}
