//! Language Server Protocol core for Synapse: Diagnostics, Hover, Phase 3 core.
use lsp_server::{Connection, Message, Request, RequestId, Response};
use lsp_types as lsp;
use parser_core::parse_str;
use type_checker_l2::check_and_annotate_graph_v2_with_effects_check;
use std::collections::HashMap;

/// Maps problems to LSP diagnostics. Layer for parse/type/effect errors.
fn diagnostics_from_error(err: &anyhow::Error) -> Vec<lsp::Diagnostic> {
    vec![lsp::Diagnostic {
        range: lsp::Range {
            start: lsp::Position { line: 0, character: 0 },
            end: lsp::Position { line: 0, character: 1 },
        },
        severity: Some(lsp::DiagnosticSeverity::ERROR),
        code: None,
        code_description: None,
        source: Some("synapse".to_string()),
        message: format!("{}", err),
        related_information: None,
        tags: None,
        data: None,
    }]
}

pub async fn run_lsp_server() -> anyhow::Result<()> {
    let (conn, io_threads) = Connection::stdio();
    let _init_params = conn.initialize(serde_json::json!({}))?;
    let mut files: HashMap<String, String> = HashMap::new();

    for msg in &conn {
        match msg {
            Message::Request(req) if conn.handle_shutdown(&req).unwrap_or(false) => break,
            Message::Request(req) => {
                let req_id = req.id.clone();
                match req.method.as_str() {
                    "textDocument/hover" => {
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
                    // Support both open and change params
                    let uri = if note.method == "textDocument/didOpen" {
                        serde_json::from_value::<lsp::DidOpenTextDocumentParams>(note.params.clone())
                            .map(|p| (p.text_document.uri.to_string(), p.text_document.text))
                            .ok()
                    } else {
                        serde_json::from_value::<lsp::DidChangeTextDocumentParams>(note.params.clone())
                            .ok().and_then(|p| {
                                p.content_changes.get(0).map(|c| (p.text_document.uri.to_string(), c.text.clone()))
                            })
                    };
                    if let Some((uri, text)) = uri {
                        files.insert(uri.clone(), text.clone());
                        // Parse + type check
                        let diagnostics = match parse_str(&text) {
                            Ok(mut asg) => {
                                match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                                    Ok(_) => vec![], // OK: no errors
                                    Err(e) => diagnostics_from_error(&anyhow::anyhow!(e))
                                }
                            }
                            Err(e) => diagnostics_from_error(&e.into())
                        };
                        let lsp_diag = lsp::PublishDiagnosticsParams {
                            uri: lsp::Url::parse(&uri).unwrap(),
                            diagnostics,
                            version: None,
                        };
                        conn.sender.send(Message::Notification(
                            lsp_server::Notification::new("textDocument/publishDiagnostics".into(), lsp_diag).into(),
                        ))?;
                    }
                }
            }
            Message::Response(_) => {}
        }
    }
    io_threads.join()?;
    Ok(())
}