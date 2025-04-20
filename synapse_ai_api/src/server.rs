use tonic::{transport::Server, Request, Response, Status};
use crate::proto::{synapseai_server::{Synapseai, SynapseaiServer}, 
    ParseTextRequest, ParseTextResponse,
    CheckAsgRequest, CheckAsgResponse,
    QueryTypeRequest, QueryTypeResponse,
    DiagnosticsRequest, DiagnosticsResponse
};
use parser_core::parse_str;
use type_checker_l2::{check_and_annotate_graph_v2_with_effects_check, Type};
use asg_core::{AsgGraph};
use std::collections::HashMap;

#[derive(Default)]
pub struct SynapseAIService {}

fn pretty_type(ty: &Type) -> String {
    use Type::*;
    match ty {
        Int => "Int".to_string(),
        Bool => "Bool".to_string(),
        Function(a, b) => format!("fn({}) -> {}", pretty_type(a), pretty_type(b)),
        Ref(inner) => format!("&{}", pretty_type(inner)),
        ForAll(vars, t) => format!("forall {}. {}", vars.iter().map(|x| format!("T{}", x)).collect::<Vec<_>>().join(", "), pretty_type(t)),
        ADT(name, args) if args.is_empty() => name.clone(),
        ADT(name, args) => format!("{}<{}>", name, args.iter().map(pretty_type).collect::<Vec<_>>().join(", ")),
        Var(v) => format!("T{}", v),
    }
}

#[tonic::async_trait]
impl Synapseai for SynapseAIService {
    async fn parse_text(&self, request: Request<ParseTextRequest>) -> Result<Response<ParseTextResponse>, Status> {
        let text = request.into_inner().text;
        match parse_str(&text) {
            Ok(asg) => {
                let asg_json = serde_json::to_string(&asg).unwrap();
                Ok(Response::new(ParseTextResponse { asg_json }))
            }
            Err(err) => Err(Status::invalid_argument(format!("Parse error: {err}")))
        }
    }
    async fn check_asg(&self, request: Request<CheckAsgRequest>) -> Result<Response<CheckAsgResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg : AsgGraph = match serde_json::from_str(&asg_json) {
            Ok(g) => g,
            Err(e) => return Err(Status::invalid_argument(format!("ASG parse fail: {e}")))
        };
        match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
            Ok(_) => Ok(Response::new(CheckAsgResponse { result: "Type/effect check: OK".to_string() })),
            Err(err) => Ok(Response::new(CheckAsgResponse { result: format!("Type/effect error: {err:?}") })),
        }
    }
    async fn query_type(&self, request: Request<QueryTypeRequest>) -> Result<Response<QueryTypeResponse>, Status> {
        let req = request.into_inner();
        let asg: AsgGraph = serde_json::from_str(&req.asg_json).map_err(|e| Status::invalid_argument(format!("ASG parse fail: {e}")))?;
        // Minimal demo: always return Int for any node
        Ok(Response::new(QueryTypeResponse { type_string: "Int".into() })) // TODO: lookup node type if available
    }
    async fn get_diagnostics(&self, request: Request<DiagnosticsRequest>) -> Result<Response<DiagnosticsResponse>, Status> {
        let asg_json = request.into_inner().asg_json;
        let mut asg : AsgGraph = match serde_json::from_str(&asg_json) {
            Ok(g) => g,
            Err(e) => return Ok(Response::new(DiagnosticsResponse { errors: vec![format!("ASG parse fail: {e}")] }))
        };
        let mut errors = vec![];
        match check_and_annotate_graph_v2_with_effects_check(&mut asg, &[] as &[&str]) {
            Ok(_) => {},
            Err(err) => errors.push(format!("{:?}", err))
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