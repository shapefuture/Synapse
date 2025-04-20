//! Language Server Protocol core for Synapse: Phase 3 boot.
use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types as lsp;
use parser_core::parse_str;
use type_checker_l2::check_and_annotate_graph_v2_with_effects_check;

pub async fn run_lsp_server() -> anyhow::Result<()> {
    let (conn, io_threads) = Connection::stdio();
    let initialization_params = conn.initialize(serde_json::json!({}))?;

    let mut files: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    for msg in &conn {
        match msg {
            Message::Request(req) if conn.handle_shutdown(&req).unwrap_or(false) => break,
            Message::Request(req) => {
                let req_id = req.id.clone();
                match req.method.as_str() {
                    "textDocument/hover" => {
                        // Minimal: always returns "TODO: type info"
                        let resp = lsp::Hover {
                            contents: lsp::HoverContents::Scalar(lsp::MarkedString::String(
                                "TODO: type info".to_string(),
                            )),
                            range: None,
                        };
                        let resp = Response::new_ok(req_id, serde_json::to_value(resp)?);
                        conn.sender.send(Message::Response(resp))?;
                    }
                    _ => {
                        let resp = Response::new_err(req_id, -32601, "Not implemented");
                        conn.sender.send(Message::Response(resp))?;
                    }
                }
            }
            Message::Notification(note) => {
                if note.method == "textDocument/didOpen" || note.method == "textDocument/didChange" {
                    if let Ok(params) = serde_json::from_value::<lsp::DidOpenTextDocumentParams>(note.params.clone())
                        .or_else(|_| serde_json::from_value::<lsp::DidChangeTextDocumentParams>(note.params.clone()))
                    {
                        let uri = params.text_document.uri.to_string();
                        let text = if let Some(s) = params.text_document.text.as_ref() {
                            s.clone()
                        } else if let Some(change) = params.content_changes.get(0) {
                            change.text.clone()
                        } else {
                            "".to_string()
                        };
                        files.insert(uri.clone(), text);
                        // TODO: On change, parse and type check, send diagnostics
                    }
                }
            }
            Message::Response(_) => {}
        }
    }
    io_threads.join()?;
    Ok(())
}