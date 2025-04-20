use tonic::{transport::Server, Request, Response, Status};
use crate::proto::{
    synapseai_server::{Synapseai, SynapseaiServer},
    ParseTextRequest, ParseTextResponse,
    CheckAsgRequest, CheckAsgResponse,
    QueryTypeRequest, QueryTypeResponse,
    DiagnosticsRequest, DiagnosticsResponse
};
use parser_core::parse_str;
use asg_core::{AsgGraph};
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, Type};
use serde_json;

#[derive(Default)]
pub struct SynapseAIService {}

#[tonic::async_trait]
impl Synapseai for SynapseAIService {
    async fn parse_text(&self, request: Request<ParseTextRequest>) -> Result<Response<ParseTextResponse>, Status> {
        let text = request.into_inner().text;
        match parse_str(&text) {
            Ok(asg) => {
                let asg_json = serde_json::to_string(&asg).unwrap();
                Ok(Response::new(ParseTextResponse { asg_json }))
            }
            Err(e) => Err(Status::invalid_argument(format!("Parse error: {}", e))),
        }
    }

    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg: AsgGraph = serde_json::from_str(&asg_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
            Ok(_) => Ok(Response::new(CheckAsgResponse { result: "OK".to_string() })),
            Err(e) => Ok(Response::new(CheckAsgResponse { result: format!("Type check error: {}", e) })),
        }
    }

    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        // Demo: always "Int" for now
        let asg_json = request.get_ref().asg_json.clone();
        let node_id = request.get_ref().node_id;
        let asg: AsgGraph = serde_json::from_str(&asg_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        // In real code: look up type annotations/typecheck map
        Ok(Response::new(QueryTypeResponse { type_string: format!("type of node {}: Int", node_id) }))
    }

    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg: AsgGraph = serde_json::from_str(&asg_json).map_err(|e| Status::invalid_argument(e.to_string()))?;
        let mut errors = Vec::new();
        if let Err(e) = check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
            errors.push(format!("{}", e));
        }
        Ok(Response::new(DiagnosticsResponse { errors }))
    }
}

pub async fn run_grpc_server() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse()?;
    let svc = SynapseAIService::default();
    Server::builder()
        .add_service(SynapseaiServer::new(svc))
        .serve(addr)
        .await?;
    Ok(())
}