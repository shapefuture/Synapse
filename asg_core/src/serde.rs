//! Serialization and deserialization for ASG instances.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::generated;
use crate::graph::AsgGraph;

/// Saves an ASG to a binary file using Protocol Buffers.
pub fn save_asg_binary<P: AsRef<Path>>(graph: &AsgGraph, path: P) -> Result<()> {
    let proto_graph = graph.clone().into_proto();
    
    // Serialize to bytes using prost
    let mut buf = Vec::new();
    prost::Message::encode(&proto_graph, &mut buf)?;
    
    // Write to file
    let mut file = File::create(path)?;
    file.write_all(&buf)?;
    
    Ok(())
}

/// Loads an ASG from a binary file.
pub fn load_asg_binary<P: AsRef<Path>>(path: P) -> Result<AsgGraph> {
    // Read the file
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    
    // Deserialize from bytes using prost
    let proto_graph = generated::AsgGraph::decode(&buf[..])?;
    
    // Convert to AsgGraph
    AsgGraph::from_proto(proto_graph)
}

// JSON serialization for debugging purposes

/// Wrapper struct for JSON serialization, since Protocol Buffers doesn't directly support serde.
#[derive(Serialize, Deserialize)]
struct JsonNode {
    node_id: u64,
    node_type: String,
    content: serde_json::Value,
}

#[derive(Serialize, Deserialize)]
struct JsonGraph {
    nodes: Vec<JsonNode>,
    root_node_id: u64,
}

/// Saves an ASG to a JSON file for debugging.
pub fn save_asg_json<P: AsRef<Path>>(graph: &AsgGraph, path: P) -> Result<()> {
    let proto_graph = graph.clone().into_proto();
    
    // Convert to custom JSON format
    let json_nodes: Vec<JsonNode> = proto_graph.nodes.iter().map(|node| {
        let node_type = match node.r#type {
            1 => "TERM_VARIABLE",
            2 => "TERM_LAMBDA",
            3 => "TERM_APPLICATION",
            4 => "LITERAL_INT",
            5 => "LITERAL_BOOL",
            6 => "PRIMITIVE_OP",
            7 => "TERM_REF",
            8 => "TERM_DEREF",
            9 => "TERM_ASSIGN",
            10 => "EFFECT_PERFORM",
            11 => "PROOF_OBLIGATION",
            12 => "TYPE_NODE",
            50 => "METADATA",
            _ => "UNKNOWN",
        }.to_string();
        
        // Convert the content to JSON
        let content = match &node.content {
            // This is a simplified conversion for illustration - a real implementation would
            // need to handle all node types comprehensively
            Some(generated::asg_node::Content::TermVariable(var)) => {
                serde_json::json!({
                    "name": var.name,
                    "definition_node_id": var.definition_node_id,
                })
            }
            Some(generated::asg_node::Content::TermLambda(lambda)) => {
                serde_json::json!({
                    "binder_variable_node_id": lambda.binder_variable_node_id,
                    "body_node_id": lambda.body_node_id,
                    "type_annotation_id": lambda.type_annotation_id,
                })
            }
            Some(generated::asg_node::Content::LiteralInt(lit)) => {
                serde_json::json!({
                    "value": lit.value,
                })
            }
            // Handle other types similarly
            _ => serde_json::json!({ "type": "not_implemented_for_json" }),
        };
        
        JsonNode {
            node_id: node.node_id,
            node_type,
            content,
        }
    }).collect();
    
    let json_graph = JsonGraph {
        nodes: json_nodes,
        root_node_id: proto_graph.root_node_id,
    };
    
    // Write to file
    let file = File::create(path)?;
    serde_json::to_writer_pretty(file, &json_graph)?;
    
    Ok(())
}

/// Loads an ASG from a JSON file.
pub fn load_asg_json<P: AsRef<Path>>(path: P) -> Result<AsgGraph> {
    // This is a placeholder implementation - a real one would need to properly
    // convert from JSON back to ASG nodes
    Err(Error::Serialization("JSON loading not fully implemented".to_string()))
}