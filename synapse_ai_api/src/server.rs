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
            Err(e) => Ok(Response::new(ParseTextResponse {
                asg_json: format!(r#"{{"error":"{}"}}"#, e),
            })),
        }
    }
    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let asg: Result<asg_core::AsgGraph, _> = serde_json::from_str(&asg_json);
        match asg {
            Ok(mut asg) => match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                Ok(_) => Ok(Response::new(CheckAsgResponse { result: "OK".to_string() })),
                Err(e) => Ok(Response::new(CheckAsgResponse { result: format!("{}", e) })),
            }
            Err(e) => Ok(Response::new(CheckAsgResponse { result: format!("Bad ASG: {}", e) })),
        }
    }
    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        let req = request.into_inner();
        let asg: Result<asg_core::AsgGraph, _> = serde_json::from_str(&req.asg_json);
        if let Ok(asg) = asg {
            // In a full impl, run type check and return type for node_id
            // Here we fake demo with "Int", should look up real result
            Ok(Response::new(QueryTypeResponse { type_string: "Int".to_string() }))
        } else {
            Ok(Response::new(QueryTypeResponse { type_string: "<parse error>".to_string() }))
        }
    }
    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let asg: Result<asg_core::AsgGraph, _> = serde_json::from_str(&request.into_inner().asg_json);
        if let Ok(mut asg) = asg {
            match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
                Ok(_) => Ok(Response::new(DiagnosticsResponse { errors: vec![] })),
                Err(e) => Ok(Response::new(DiagnosticsResponse { errors: vec![format!("{}", e)] })),
            }
        } else {
            Ok(Response::new(DiagnosticsResponse { errors: vec!["ASG parse error".to_string()] }))
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