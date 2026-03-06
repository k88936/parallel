use agent_client_protocol as acp;
use chrono::Utc;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

use parallel_protocol::{AgentMessage, MessageType};

struct RunningTerminal {
    process: Mutex<tokio::process::Child>,
    output: Arc<RwLock<String>>,
}

pub struct ACPClient {
    workdir: PathBuf,
    terminals: RwLock<HashMap<acp::TerminalId, Arc<RunningTerminal>>>,
    messages: Arc<RwLock<Vec<AgentMessage>>>,
    current_message_type: RwLock<Option<MessageType>>,
}

impl ACPClient {
    pub fn new(workdir: PathBuf) -> Self {
        Self {
            workdir,
            terminals: RwLock::new(HashMap::new()),
            messages: Arc::new(RwLock::new(Vec::new())),
            current_message_type: RwLock::new(None),
        }
    }

    pub async fn get_messages(&self) -> Vec<AgentMessage> {
        self.messages.read().await.clone()
    }

    pub async fn clear_messages(&self) {
        self.messages.write().await.clear();
    }

    async fn add_message(&self, role: &str, message_type: MessageType, content: String) {
        let mut messages = self.messages.write().await;
        let mut current_type = self.current_message_type.write().await;

        if let Some(ref last_type) = *current_type {
            if last_type == &message_type {
                if let Some(last_message) = messages.last_mut() {
                    last_message.content.push_str(&content);
                    return;
                }
            }
        }

        *current_type = Some(message_type.clone());
        messages.push(AgentMessage {
            timestamp: Utc::now(),
            role: role.to_string(),
            message_type,
            content,
        });
    }

    fn resolve_path(&self, path: &PathBuf) -> PathBuf {
        if path.is_absolute() {
            path.clone()
        } else {
            self.workdir.join(path)
        }
    }
}

#[async_trait::async_trait(?Send)]
impl acp::Client for ACPClient {
    async fn request_permission(
        &self,
        args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        let option = args.options.first();
        let outcome = if let Some(opt) = option {
            tracing::info!("Auto-approving permission: {}", opt.name);
            acp::RequestPermissionOutcome::Selected(acp::SelectedPermissionOutcome::new(
                opt.option_id.clone(),
            ))
        } else {
            acp::RequestPermissionOutcome::Cancelled
        };

        Ok(acp::RequestPermissionResponse::new(outcome))
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        match &args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                match content {
                    acp::ContentBlock::Text(text) => {
                        print!("{}", text.text);
                        self.add_message("assistant", MessageType::Text, text.text.clone())
                            .await;
                    }
                    acp::ContentBlock::Image(img) => {
                        self.add_message(
                            "assistant",
                            MessageType::Image,
                            format!("[Image: {}]", img.mime_type),
                        )
                        .await;
                    }
                    acp::ContentBlock::Resource(res) => {
                        let uri = match &res.resource {
                            acp::EmbeddedResourceResource::TextResourceContents(text_res) => {
                                text_res.uri.clone()
                            }
                            acp::EmbeddedResourceResource::BlobResourceContents(blob_res) => {
                                blob_res.uri.clone()
                            }
                            _ => "unknown".to_string(),
                        };
                        self.add_message(
                            "assistant",
                            MessageType::Resource,
                            format!("[Resource: {}]", uri),
                        )
                        .await;
                    }
                    _ => {}
                }
            }
            acp::SessionUpdate::ToolCall(update) => {
                tracing::info!("Tool call: {}", update.title);
                self.add_message(
                    "assistant",
                    MessageType::ToolCall,
                    format!("[Tool Call: {}]", update.title),
                )
                .await;
            }
            acp::SessionUpdate::Plan(plan) => {
                let plan_content = plan
                    .entries
                    .iter()
                    .map(|e| format!("- {}", e.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                self.add_message(
                    "assistant",
                    MessageType::Plan,
                    format!("[Plan]\n{}", plan_content),
                )
                .await;
            }
            acp::SessionUpdate::AgentThoughtChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    self.add_message("assistant", MessageType::Thought, text.text.clone())
                        .await;
                }
            }
            acp::SessionUpdate::UserMessageChunk(chunk) => {
                if let acp::ContentBlock::Text(text) = &chunk.content {
                    self.add_message("user", MessageType::UserMessage, text.text.clone())
                        .await;
                }
            }
            _ => {}
        }
        Ok(())
    }

    async fn write_text_file(
        &self,
        args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        let path = self.resolve_path(&args.path);

        if let Some(parent) = path.parent() {
            let _ = tokio::fs::create_dir_all(parent).await;
        }

        match tokio::fs::write(&path, &args.content).await {
            Ok(()) => Ok(acp::WriteTextFileResponse::new()),
            Err(e) => {
                Err(acp::Error::invalid_params().data(format!("Failed to write file: {}", e)))
            }
        }
    }

    async fn read_text_file(
        &self,
        args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        let path = self.resolve_path(&args.path);

        match tokio::fs::read_to_string(&path).await {
            Ok(content) => Ok(acp::ReadTextFileResponse::new(content)),
            Err(e) => Err(acp::Error::invalid_params().data(format!("Failed to read file: {}", e))),
        }
    }

    async fn create_terminal(
        &self,
        args: acp::CreateTerminalRequest,
    ) -> acp::Result<acp::CreateTerminalResponse> {
        let terminal_id = acp::TerminalId::new(uuid::Uuid::new_v4().to_string());
        let cwd = args.cwd.as_ref().map(|p| self.resolve_path(p));

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
                .map(|e| (e.name.clone(), e.value.clone()))
                .collect();
            cmd.envs(envs);
        }

        let mut child = cmd.spawn().map_err(|e| {
            acp::Error::invalid_params().data(format!("Failed to spawn command: {}", e))
        })?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let output: Arc<RwLock<String>> = Arc::new(RwLock::new(String::new()));
        let output_for_task = output.clone();

        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            let mut stdout = stdout;
            let mut stderr = stderr;
            let mut stdout_buf = [0u8; 1024];
            let mut stderr_buf = [0u8; 1024];

            loop {
                tokio::select! {
                    Ok(n) = stdout.read(&mut stdout_buf) => {
                        if n == 0 { break; }
                        let s = String::from_utf8_lossy(&stdout_buf[..n]);
                        let mut out = output_for_task.write().await;
                        out.push_str(&s);
                    }
                    Ok(n) = stderr.read(&mut stderr_buf) => {
                        if n == 0 { break; }
                        let s = String::from_utf8_lossy(&stderr_buf[..n]);
                        let mut out = output_for_task.write().await;
                        out.push_str(&s);
                    }
                    else => break,
                }
            }
        });

        let terminal = Arc::new(RunningTerminal {
            process: Mutex::new(child),
            output,
        });

        let mut terminals = self.terminals.write().await;
        terminals.insert(terminal_id.clone(), terminal);

        Ok(acp::CreateTerminalResponse::new(terminal_id))
    }

    async fn terminal_output(
        &self,
        args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        let terminals = self.terminals.read().await;
        let terminal = terminals
            .get(&args.terminal_id)
            .ok_or_else(|| acp::Error::invalid_params().data("Terminal not found"))?;

        let output = terminal.output.read().await.clone();

        let exit_status = {
            let mut process = terminal.process.lock().await;
            if let Ok(Some(status)) = process.try_wait() {
                Some(acp::TerminalExitStatus::new().exit_code(status.code().unwrap_or(-1) as u32))
            } else {
                None
            }
        };

        Ok(acp::TerminalOutputResponse::new(output, false).exit_status(exit_status))
    }

    async fn release_terminal(
        &self,
        args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        let mut terminals = self.terminals.write().await;
        if let Some(terminal) = terminals.remove(&args.terminal_id) {
            let mut process = terminal.process.lock().await;
            let _ = process.kill().await;
        }

        Ok(acp::ReleaseTerminalResponse::new())
    }

    async fn wait_for_terminal_exit(
        &self,
        args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        let terminals = self.terminals.read().await;
        let terminal = terminals
            .get(&args.terminal_id)
            .ok_or_else(|| acp::Error::invalid_params().data("Terminal not found"))?;

        let mut process = terminal.process.lock().await;
        let status = process.wait().await;

        let exit_status = match status {
            Ok(s) => acp::TerminalExitStatus::new().exit_code(s.code().unwrap_or(-1) as u32),
            Err(_) => acp::TerminalExitStatus::new(),
        };

        Ok(acp::WaitForTerminalExitResponse::new(exit_status))
    }

    async fn kill_terminal_command(
        &self,
        args: acp::KillTerminalCommandRequest,
    ) -> acp::Result<acp::KillTerminalCommandResponse> {
        let terminals = self.terminals.read().await;
        if let Some(terminal) = terminals.get(&args.terminal_id) {
            let mut process = terminal.process.lock().await;
            let _ = process.kill().await;
        }

        Ok(acp::KillTerminalCommandResponse::new())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Ok(())
    }
}
