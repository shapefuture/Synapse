use tonic::{transport::Server, Request, Response, Status};
use crate::proto::{
    synapseai_server::{Synapseai, SynapseaiServer},
    ParseTextRequest, ParseTextResponse,
    CheckAsgRequest, CheckAsgResponse,
    QueryTypeRequest, QueryTypeResponse,
    DiagnosticsRequest, DiagnosticsResponse,
};
use parser_core::parse_str;
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, Type};
use asg_core::{AsgGraph};
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
            Err(e) => Err(Status::internal(format!("Parse error: {}", e))),
        }
    }
    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg: AsgGraph = serde_json::from_str(&asg_json).map_err(|e| Status::invalid_argument(format!("JSON decode: {}", e)))?;
        match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[]) {
            Ok(()) => Ok(Response::new(CheckAsgResponse { result: "OK".to_owned() })),
            Err(e) => Ok(Response::new(CheckAsgResponse { result: format!("Type error: {}", e) })),
        }
    }
    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        let req = request.into_inner();
        let asg: AsgGraph = serde_json::from_str(&req.asg_json).map_err(|e| Status::invalid_argument(format!("JSON decode: {}", e)))?;
        // Use node_id for the query
        if let Some(node) = asg.nodes.get(&req.node_id) {
            // For demo, always Int
            let ty = Type::Int;
            Ok(Response::new(QueryTypeResponse { type_string: format!("{:?}", ty) }))
        } else {
            Err(Status::not_found("Node ID not found"))
        }
    }
    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg: AsgGraph = serde_json::from_str(&asg_json).map_err(|e| Status::invalid_argument(format!("JSON decode: {}", e)))?;
        match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[]) {
            Ok(()) => Ok(Response::new(DiagnosticsResponse { errors: vec![] })),
            Err(e) => Ok(Response::new(DiagnosticsResponse { errors: vec![format!("{}", e)] })),
        }
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