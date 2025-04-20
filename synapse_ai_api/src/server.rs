//! AI API gRPC server: real implementation for parse/check/type/diagnostics
use tonic::{transport::Server, Request, Response, Status};
use crate::proto::{
    synapseai_server::{Synapseai, SynapseaiServer},
    ParseTextRequest, ParseTextResponse,
    CheckAsgRequest, CheckAsgResponse,
    QueryTypeRequest, QueryTypeResponse,
    DiagnosticsRequest, DiagnosticsResponse
};
use parser_core::parse_str;
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, Type};
use asg_core::AsgGraph;
use serde_json;

#[derive(Default)]
pub struct SynapseAIService {}

fn deserialize_asg(asg_json: &str) -> Result<AsgGraph, anyhow::Error> {
    serde_json::from_str(asg_json).map_err(|e| anyhow::anyhow!(e))
}
fn serialize_asg(asg: &AsgGraph) -> Result<String, anyhow::Error> {
    serde_json::to_string_pretty(asg).map_err(|e| anyhow::anyhow!(e))
}

#[tonic::async_trait]
impl Synapseai for SynapseAIService {
    async fn parse_text(&self, request: Request<ParseTextRequest>) -> Result<Response<ParseTextResponse>, Status> {
        let text = request.into_inner().text;
        match parse_str(&text) {
            Ok(asg) => Ok(Response::new(ParseTextResponse { asg_json: serialize_asg(&asg).unwrap() })),
            Err(e) => Err(Status::invalid_argument(format!("Parse error: {}", e))),
        }
    }

    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        match deserialize_asg(&asg_json) {
            Ok(mut asg) => {
                match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                    Ok(_) => Ok(Response::new(CheckAsgResponse { result: "Check OK".to_string() })),
                    Err(e) => Ok(Response::new(CheckAsgResponse { result: format!("Type/effect error: {}", e) })),
                }
            }
            Err(e) => Err(Status::invalid_argument(format!("Malformed ASG: {}", e))),
        }
    }

    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        let QueryTypeRequest { asg_json, node_id } = request.into_inner();
        match deserialize_asg(&asg_json) {
            Ok(asg) => {
                // Stub: real type lookup would map node_id to type via checker. For now, Int for all.
                let typ = Type::Int;
                Ok(Response::new(QueryTypeResponse { type_string: format!("{:?}", typ) }))
            }
            Err(e) => Err(Status::invalid_argument(format!("Malformed ASG: {}", e))),
        }
    }

    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        match deserialize_asg(&asg_json) {
            Ok(mut asg) => {
                match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                    Ok(_) => Ok(Response::new(DiagnosticsResponse { errors: vec![] })),
                    Err(e) => Ok(Response::new(DiagnosticsResponse { errors: vec![format!("{}", e)] })),
                }
            }
            Err(e) => Err(Status::invalid_argument(format!("Malformed ASG: {}", e))),
        }
    }
}

pub async fn run_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = SynapseAIService::default();
    println!("Synapse AI API server listening on {}", addr);
    Server::builder()
        .add_service(SynapseaiServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}