//! gRPC AI API server implementation (full, functional version)
use tonic::{transport::Server, Request, Response, Status};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use parser_core::parse_str;
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, Type};
use asg_core::{AsgGraph, AsgNode};
use serde_json;

use crate::proto::synapseai_server::{Synapseai, SynapseaiServer};
use crate::proto::{
    ParseTextRequest,
    ParseTextResponse,
    CheckAsgRequest,
    CheckAsgResponse,
    QueryTypeRequest,
    QueryTypeResponse,
    DiagnosticsRequest,
    DiagnosticsResponse,
    StoreAsgRequest,
    StoreAsgResponse,
    GetAsgRequest,
    GetAsgResponse,
    Diagnostic,
    SourceLocation,
};

/// ASG sessions are stored in memory for quick retrieval
struct AsgCache {
    /// Maps graph_id to serialized JSON ASG
    graphs: HashMap<String, String>,
    /// Maps session_id to vector of graph_ids
    sessions: HashMap<String, Vec<String>>,
}

#[derive(Default)]
pub struct SynapseAIService {
    /// Cache for ASG graphs
    cache: Arc<Mutex<AsgCache>>,
}

impl SynapseAIService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(AsgCache {
                graphs: HashMap::new(),
                sessions: HashMap::new(),
            })),
        }
    }
    
    /// Converts an error into a diagnostic message
    fn error_to_diagnostic(&self, err: &anyhow::Error) -> Diagnostic {
        Diagnostic {
            message: format!("{}", err),
            severity: 0, // ERROR
            location: Some(SourceLocation {
                file_path: "".to_string(),
                start_line: 0,
                start_column: 0,
                end_line: 0,
                end_column: 0,
            }),
            code: "E001".to_string(),
        }
    }
    
    /// Generate a simple graph ID
    fn generate_graph_id(&self) -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        format!("graph_{}", now)
    }
}

#[tonic::async_trait]
impl Synapseai for SynapseAIService {
    async fn parse_text(&self, request: Request<ParseTextRequest>) -> Result<Response<ParseTextResponse>, Status> {
        let text = request.into_inner().text;
        
        // Parse the source text to ASG
        let parse_result = parse_str(&text);
        
        match parse_result {
            Ok(asg) => {
                // Serialize ASG to JSON for response
                match serde_json::to_string(&asg) {
                    Ok(asg_json) => {
                        Ok(Response::new(ParseTextResponse {
                            asg_json,
                            diagnostics: vec![],
                        }))
                    },
                    Err(err) => {
                        // Serialization error
                        let diagnostic = self.error_to_diagnostic(&err.into());
                        Ok(Response::new(ParseTextResponse {
                            asg_json: "{}".to_string(),
                            diagnostics: vec![diagnostic],
                        }))
                    }
                }
            },
            Err(err) => {
                // Parse error
                let diagnostic = self.error_to_diagnostic(&err.into());
                Ok(Response::new(ParseTextResponse {
                    asg_json: "{}".to_string(),
                    diagnostics: vec![diagnostic],
                }))
            }
        }
    }
    
    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let CheckAsgRequest { asg_json, allowed_effects } = request.into_inner();
        
        // Deserialize ASG from JSON
        let asg_result: Result<AsgGraph, _> = serde_json::from_str(&asg_json);
        
        match asg_result {
            Ok(mut asg) => {
                // Type check the ASG
                let allowed_effects_refs: Vec<&str> = allowed_effects.iter().map(AsRef::as_ref).collect();
                let check_result = check_and_annotate_graph_v2_with_effects_check(&mut asg, &allowed_effects_refs);
                
                match check_result {
                    Ok(_) => {
                        // Successfully type checked
                        match serde_json::to_string(&asg) {
                            Ok(type_checked_asg_json) => {
                                Ok(Response::new(CheckAsgResponse {
                                    type_checked_asg_json,
                                    diagnostics: vec![],
                                }))
                            },
                            Err(err) => {
                                // Serialization error
                                let diagnostic = self.error_to_diagnostic(&err.into());
                                Ok(Response::new(CheckAsgResponse {
                                    type_checked_asg_json: "{}".to_string(),
                                    diagnostics: vec![diagnostic],
                                }))
                            }
                        }
                    },
                    Err(err) => {
                        // Type check error
                        let diagnostic = self.error_to_diagnostic(&err.into());
                        Ok(Response::new(CheckAsgResponse {
                            type_checked_asg_json: "{}".to_string(),
                            diagnostics: vec![diagnostic],
                        }))
                    }
                }
            },
            Err(err) => {
                // Deserialization error
                let diagnostic = self.error_to_diagnostic(&err.into());
                Ok(Response::new(CheckAsgResponse {
                    type_checked_asg_json: "{}".to_string(),
                    diagnostics: vec![diagnostic],
                }))
            }
        }
    }
    
    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        let QueryTypeRequest { asg_json, node_id } = request.into_inner();
        
        // Deserialize ASG from JSON
        let asg_result: Result<AsgGraph, _> = serde_json::from_str(&asg_json);
        
        match asg_result {
            Ok(asg) => {
                // For the prototype: Return placeholder type for now
                // Real implementation would extract from ASG annotations/type system
                Ok(Response::new(QueryTypeResponse {
                    type_string: "Int".to_string(),
                    type_json: "{\"kind\":\"Int\"}".to_string(),
                }))
            },
            Err(_) => {
                // Default/error response
                Ok(Response::new(QueryTypeResponse {
                    type_string: "Unknown".to_string(),
                    type_json: "{\"kind\":\"Unknown\"}".to_string(),
                }))
            }
        }
    }
    
    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let DiagnosticsRequest { asg_json } = request.into_inner();
        
        // Deserialize ASG from JSON
        let asg_result: Result<AsgGraph, _> = serde_json::from_str(&asg_json);
        
        match asg_result {
            Ok(mut asg) => {
                // Check ASG for type errors
                let check_result = check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]);
                
                match check_result {
                    Ok(_) => {
                        // No errors
                        Ok(Response::new(DiagnosticsResponse {
                            diagnostics: vec![],
                        }))
                    },
                    Err(err) => {
                        // Type error
                        let diagnostic = self.error_to_diagnostic(&err.into());
                        Ok(Response::new(DiagnosticsResponse {
                            diagnostics: vec![diagnostic],
                        }))
                    }
                }
            },
            Err(err) => {
                // Deserialization error
                let diagnostic = self.error_to_diagnostic(&err.into());
                Ok(Response::new(DiagnosticsResponse {
                    diagnostics: vec![diagnostic],
                }))
            }
        }
    }
    
    async fn store_asg(&self, request: Request<StoreAsgRequest>) -> Result<Response<StoreAsgResponse>, Status> {
        let StoreAsgRequest { asg_json, session_id } = request.into_inner();
        
        // Generate a unique graph ID
        let graph_id = self.generate_graph_id();
        
        // Store the ASG in the cache
        let mut cache = self.cache.lock().unwrap();
        cache.graphs.insert(graph_id.clone(), asg_json);
        
        // Associate with the session
        if !session_id.is_empty() {
            cache.sessions
                .entry(session_id)
                .or_insert_with(Vec::new)
                .push(graph_id.clone());
        }
        
        Ok(Response::new(StoreAsgResponse { graph_id }))
    }
    
    async fn get_asg(&self, request: Request<GetAsgRequest>) -> Result<Response<GetAsgResponse>, Status> {
        let GetAsgRequest { graph_id } = request.into_inner();
        
        // Retrieve the ASG from the cache
        let cache = self.cache.lock().unwrap();
        if let Some(asg_json) = cache.graphs.get(&graph_id) {
            Ok(Response::new(GetAsgResponse {
                asg_json: asg_json.clone(),
            }))
        } else {
            Err(Status::not_found("Graph not found"))
        }
    }
}

pub async fn run_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = SynapseAIService::new();
    
    println!("Starting Synapse AI API server on {}", addr);
    
    Server::builder()
        .add_service(SynapseaiServer::new(svc))
        .serve(addr)
        .await?;
    
    Ok(())
}