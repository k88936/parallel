use agent_client_protocol::{self as acp, Agent as _, KillTerminalRequest, KillTerminalResponse};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

struct MockedACPClient {}

#[async_trait::async_trait(?Send)]
impl acp::Client for MockedACPClient {
    async fn request_permission(
        &self,
        _args: acp::RequestPermissionRequest,
    ) -> acp::Result<acp::RequestPermissionResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn write_text_file(
        &self,
        _args: acp::WriteTextFileRequest,
    ) -> acp::Result<acp::WriteTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn read_text_file(
        &self,
        _args: acp::ReadTextFileRequest,
    ) -> acp::Result<acp::ReadTextFileResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn create_terminal(
        &self,
        _args: acp::CreateTerminalRequest,
    ) -> Result<acp::CreateTerminalResponse, acp::Error> {
        Err(acp::Error::method_not_found())
    }

    async fn terminal_output(
        &self,
        _args: acp::TerminalOutputRequest,
    ) -> acp::Result<acp::TerminalOutputResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn release_terminal(
        &self,
        _args: acp::ReleaseTerminalRequest,
    ) -> acp::Result<acp::ReleaseTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn wait_for_terminal_exit(
        &self,
        _args: acp::WaitForTerminalExitRequest,
    ) -> acp::Result<acp::WaitForTerminalExitResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn kill_terminal(
        &self,
        _args: KillTerminalRequest,
    ) -> agent_client_protocol::Result<KillTerminalResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn session_notification(
        &self,
        args: acp::SessionNotification,
    ) -> acp::Result<(), acp::Error> {
        match args.update {
            acp::SessionUpdate::AgentMessageChunk(acp::ContentChunk { content, .. }) => {
                let text = match content {
                    acp::ContentBlock::Text(text_content) => text_content.text,
                    acp::ContentBlock::Image(_) => "<image>".into(),
                    acp::ContentBlock::Audio(_) => "<audio>".into(),
                    acp::ContentBlock::ResourceLink(resource_link) => resource_link.uri,
                    acp::ContentBlock::Resource(_) => "<resource>".into(),
                    _ => "<unknown>".into(),
                };
                println!("| Agent: {text}");
            }
            _ => {} // Handle future variants gracefully
        }
        Ok(())
    }

    async fn ext_method(&self, _args: acp::ExtRequest) -> acp::Result<acp::ExtResponse> {
        Err(acp::Error::method_not_found())
    }

    async fn ext_notification(&self, _args: acp::ExtNotification) -> acp::Result<()> {
        Err(acp::Error::method_not_found())
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let command = std::env::args().collect::<Vec<_>>();
    let (outgoing, incoming, child) = match command.as_slice() {
        [_, program, args @ ..] => {
            let mut child = tokio::process::Command::new(program)
                .args(args.iter())
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .kill_on_drop(true)
                .spawn()?;
            (
                child.stdin.take().unwrap().compat_write(),
                child.stdout.take().unwrap().compat(),
                child,
            )
        }
        _ => anyhow::bail!("Usage: client AGENT_PROGRAM AGENT_ARG..."),
    };

    // The ClientSideConnection will spawn futures onto our Tokio runtime.
    // LocalSet and spawn_local are used because the futures from the
    // agent-client-protocol crate are not Send.
    let local_set = tokio::task::LocalSet::new();
    local_set
        .run_until(async move {
            // Set up the ExampleClient connected to the agent's stdio.
            let (conn, handle_io) =
                acp::ClientSideConnection::new(MockedACPClient {}, outgoing, incoming, |fut| {
                    tokio::task::spawn_local(fut);
                });

            // Handle I/O in the background.
            tokio::task::spawn_local(handle_io);

            // Connect to the agent and set up a session.
            conn.initialize(acp::InitializeRequest::new(acp::ProtocolVersion::V1)
                .client_capabilities(acp::ClientCapabilities::default())
                .client_info(acp::Implementation::new("example-client", "0.1.0")
                    .title("Example Client")))
            .await?;
            let response = conn
                .new_session(acp::NewSessionRequest::new(std::env::current_dir()?))
                .await?;

            // Send prompts to the agent until stdin is closed.
            let mut rl = rustyline::DefaultEditor::new()?;
            while let Ok(line) = rl.readline("> ") {
                let _result = conn
                    .prompt(acp::PromptRequest::new(response.session_id.clone(), vec![line.into()]))
                    .await;
            }

            drop(child);
            Ok(())
        })
        .await
}
