use tonic::{transport::Server, Request, Response, Status};
use crate::proto::{synapseai_server::{Synapseai, SynapseaiServer}, ParseTextRequest, ParseTextResponse};

#[derive(Default)]
pub struct SynapseAIService {}

#[tonic::async_trait]
impl Synapseai for SynapseAIService {
    async fn parse_text(&self, request: Request<ParseTextRequest>) -> Result<Response<ParseTextResponse>, Status> {
        let text = request.into_inner().text;
        // TODO: implement ASG parse + response
        Ok(Response::new(ParseTextResponse {
            asg_json: "{}".to_string()
        }))
    }
    // TODO: implement check_asg, query_type, diagnostics
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