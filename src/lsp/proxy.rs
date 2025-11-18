use std::process::Stdio;
use std::sync::Arc;

use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

// The proxy lsp that spawns vtsls or ts_server and forwards the `textDocument/publishDiagnostics`
// to the lsp client

pub enum TsLsp {
    Vtsls,
    TsServer,
}

pub struct LspProxy {
    pub ts_lsp: TsLsp,
}

// The Backend struct that will implement the LanguageServer trait.
// It holds the client for communicating with the editor and a handle
// to the stdin of the real LSP server process.
struct Backend {
    client: Client,
    downstream_stdin: Arc<Mutex<tokio::process::ChildStdin>>,
}

impl LspProxy {
    pub fn new(ts_lsp: TsLsp) -> Self {
        LspProxy { ts_lsp }
    }

    #[tokio::main]
    pub async fn start_as_proxy(self) {
        let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

        let command_name = match self.ts_lsp {
            TsLsp::Vtsls => "vtsls",
            // Use `typescript-language-server` with the `--stdio` flag
            TsLsp::TsServer => "typescript-language-server",
        };
        let command_args = match self.ts_lsp {
            TsLsp::Vtsls => vec!["--stdio"],
            TsLsp::TsServer => vec!["--stdio"],
        };

        let mut child = Command::new(command_name)
            .args(&command_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("Failed to start downstream LSP server");

        let downstream_stdin = Arc::new(Mutex::new(
            child.stdin.take().expect("Failed to get child stdin"),
        ));
        let downstream_stdout = child.stdout.take().expect("Failed to get child stdout");
        let downstream_stderr = child.stderr.take().expect("Failed to get child stderr");

        let (service, socket) = LspService::build(|client| {
            let lsp_client = client.clone();

            // Task to log stderr from the downstream server
            tokio::spawn(async move {
                let mut reader = BufReader::new(downstream_stderr);
                let mut buffer = String::new();
                loop {
                    match reader.read_line(&mut buffer).await {
                        Ok(0) => break, // EOF
                        Ok(_) => {
                            lsp_client
                                .log_message(
                                    MessageType::LOG,
                                    format!("Downstream: {}", buffer.trim()),
                                )
                                .await;
                            buffer.clear();
                        }
                        Err(e) => {
                            lsp_client
                                .log_message(
                                    MessageType::ERROR,
                                    format!("Error reading downstream stderr: {}", e),
                                )
                                .await;
                            break;
                        }
                    }
                }
            });

            // This is the core task that reads from the real LSP server,
            // intercepts messages, and forwards them to the client.
            let lsp_client = client.clone();
            tokio::spawn(async move {
                let mut reader = BufReader::new(downstream_stdout);
                loop {
                    match read_lsp_message(&mut reader).await {
                        Ok(Some(body)) => {
                            lsp_client
                                .log_message(MessageType::INFO, "[PROXY] Received message from downstream".to_string())
                                .await;
                            if let Ok(json) = serde_json::from_str::<Value>(&body) {
                                // Check if it's a publishDiagnostics notification
                                if json.get("method")
                                    == Some(&Value::String(
                                        "textDocument/publishDiagnostics".to_string(),
                                    ))
                                {
                                    lsp_client
                                        .log_message(MessageType::INFO, "Intercepted diagnostics!")
                                        .await;

                                     if let Ok(params) = 
                                        serde_json::from_value::<PublishDiagnosticsParams>(
                                            json["params"].clone(),
                                        )
                                    {
                                        // Add a general log to confirm we are intercepting diagnostics
                                        lsp_client
                                            .log_message(
                                                MessageType::INFO,
                                                format!("Intercepted {} diagnostics for {}", params.diagnostics.len(), params.uri),
                                            )
                                            .await;

                                        for diagnostic in &params.diagnostics {
                                            let is_ts2322 = if let Some(code) = &diagnostic.code {
                                                match code {
                                                    tower_lsp::lsp_types::NumberOrString::Number(num) => *num == 2322,
                                                    tower_lsp::lsp_types::NumberOrString::String(s) => s == "2322",
                                                }
                                            } else {
                                                false
                                            };

                                            if is_ts2322 {
                                                let ts_error = crate::parser::TsError {
                                                    file: params.uri.path().to_string(),
                                                    line: diagnostic.range.start.line as usize + 1,
                                                    column: diagnostic.range.start.character as usize + 1,
                                                    code: crate::parser::CommonErrors::TypeMismatch,
                                                    message: diagnostic.message.clone(),
                                                };
                                                let formatted_error = crate::formatter::fmt(&ts_error);
                                                lsp_client
                                                    .log_message(
                                                        MessageType::INFO,
                                                        format!("Formatted Error: {}", formatted_error),
                                                    )
                                                    .await;
                                            }
                                        }

                                        // We still forward the original diagnostics for now
                                        lsp_client
                                            .publish_diagnostics(
                                                params.uri,
                                                params.diagnostics,
                                                params.version,
                                            )
                                            .await;
                                    }
                                } else if json.get("method").is_some() {
                                    // TODO: Forwarding other notifications from server to client
                                    // is not straightforward with tower-lsp's client API.
                                    // You would need to match on the method name and call the
                                    // corresponding typed method on the `Client`.
                                } else {
                                    // TODO: Handle responses to requests initiated by the client.
                                    // This is the hardest part of proxying and requires mapping
                                    // request IDs.
                                }
                            }
                        }
                        Ok(None) => {
                            lsp_client
                                .log_message(MessageType::INFO, "Downstream LSP closed")
                                .await;
                            break;
                        }
                        Err(e) => {
                            lsp_client
                                .log_message(
                                    MessageType::ERROR,
                                    format!("Error reading from downstream: {}", e),
                                )
                                .await;
                            break;
                        }
                    }
                }
            });

            Backend {
                client,
                downstream_stdin: downstream_stdin.clone(),
            }
        })
        .finish();

        Server::new(stdin, stdout, socket).serve(service).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        // In a real implementation, you would forward the InitializeParams to the
        // downstream server and return its InitializeResult. For now, we fake it.
        self.client
            .log_message(MessageType::INFO, "Proxy LSP initializing.")
            .await;
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "ts-analyzer-proxy".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                // Proxy most capabilities. For simplicity, we'll start with a few.
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                // Add other capabilities you want to proxy from vtsls/ts_server
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Proxy LSP initialized!")
            .await;
    }

    // Forward notifications from the client to the downstream server.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(
                MessageType::INFO,
                format!("[PROXY] Forwarding didOpen for {}", params.text_document.uri),
            )
            .await;
        self.forward_notification("textDocument/didOpen", params)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.forward_notification("textDocument/didChange", params)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        self.forward_notification("textDocument/didSave", params)
            .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.forward_notification("textDocument/didClose", params)
            .await;
    }

    // TODO: Implement request forwarding (e.g., for hover, completion)
    // This is more complex as it requires matching request IDs.

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        self.client
            .log_message(MessageType::INFO, "Proxy LSP shutting down.")
            .await;
        Ok(())
    }
}

impl Backend {
    // Helper to serialize and forward a notification to the downstream server.
    async fn forward_notification<P: serde::Serialize>(&self, method: &str, params: P) {
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        if let Ok(body) = serde_json::to_string(&notification) {
            let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
            let mut stdin = self.downstream_stdin.lock().await;
            if let Err(e) = stdin.write_all(message.as_bytes()).await {
                self.client
                    .log_message(
                        MessageType::ERROR,
                        format!("Failed to forward notification: {}", e),
                    )
                    .await;
            }
        }
    }
}

/// Reads a single LSP message from the reader.
async fn read_lsp_message<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
) -> anyhow::Result<Option<String>> {
    let mut content_length = 0;
    let mut buffer = String::new();

    // Read headers
    loop {
        buffer.clear();
        if reader.read_line(&mut buffer).await? == 0 {
            return Ok(None); // Connection closed
        }
        if buffer.trim().is_empty() {
            break; // End of headers
        }
        if let Some(len_str) = buffer.strip_prefix("Content-Length: ") {
            content_length = len_str.trim().parse()?;
        }
    }

    if content_length == 0 {
        return Err(anyhow::anyhow!("Missing Content-Length header"));
    }

    // Read body
    let mut body = vec![0; content_length];
    reader.read_exact(&mut body).await?;
    Ok(Some(String::from_utf8(body)?))
}

mod tests {
    #[tokio::test]
    async fn test_proxy_vtsls_startup() {
        use super::{LspProxy, TsLsp};

        let proxy = LspProxy::new(TsLsp::Vtsls);
        tokio::spawn(async move {
            proxy.start_as_proxy();
        });
        // 1 sec buffer time to let it start
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    #[tokio::test]
    async fn test_proxy_ts_server_startup() {
        use super::{LspProxy, TsLsp};

        let proxy = LspProxy::new(TsLsp::TsServer);
        tokio::spawn(async move {
            proxy.start_as_proxy();
        });
        // 1 sec buffer time to let it start
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}