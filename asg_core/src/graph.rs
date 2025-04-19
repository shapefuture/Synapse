//! Core implementation of the Abstract Semantic Graph structure.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::error::{Error, Result};
use crate::generated;

/// The primary data structure for representing Synapse code.
#[derive(Debug, Clone)]
pub struct AsgGraph {
    /// Map of node IDs to nodes
    nodes: HashMap<u64, generated::AsgNode>,
    /// Counter for generating unique node IDs
    next_id: AtomicU64,
    /// Root node ID of the graph (optional)
    root_id: Option<u64>,
}

impl AsgGraph {
    /// Creates a new, empty ASG.
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            next_id: AtomicU64::new(1), // Start at 1, reserving 0 as invalid/null
            root_id: None,
        }
    }
    
    /// Generates a new unique node ID.
    pub fn generate_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Adds a node to the graph with the specified content.
    /// Returns the ID of the newly created node.
    pub fn add_node(&mut self, node_type: generated::NodeType, content: impl Into<generated::asg_node::Content>) -> u64 {
        let node_id = self.generate_id();
        
        let mut node = generated::AsgNode {
            node_id,
            r#type: node_type as i32,
            content: Some(content.into()),
        };
        
        self.nodes.insert(node_id, node);
        node_id
    }
    
    /// Sets the root node ID for the graph.
    pub fn set_root(&mut self, node_id: u64) -> Result<()> {
        if !self.nodes.contains_key(&node_id) {
            return Err(Error::NodeNotFound(node_id));
        }
        self.root_id = Some(node_id);
        Ok(())
    }
    
    /// Gets the root node ID of the graph, if set.
    pub fn root_id(&self) -> Option<u64> {
        self.root_id
    }
    
    /// Retrieves a node by its ID.
    pub fn get_node(&self, node_id: u64) -> Option<&generated::AsgNode> {
        self.nodes.get(&node_id)
    }
    
    /// Retrieves a mutable reference to a node by its ID.
    pub fn get_node_mut(&mut self, node_id: u64) -> Option<&mut generated::AsgNode> {
        self.nodes.get_mut(&node_id)
    }
    
    /// Returns a reference to the node map.
    pub fn nodes(&self) -> &HashMap<u64, generated::AsgNode> {
        &self.nodes
    }
    
    /// Checks if a node exists with the given ID.
    pub fn contains_node(&self, node_id: u64) -> bool {
        self.nodes.contains_key(&node_id)
    }
    
    /// Removes a node from the graph. Returns the removed node if it existed.
    pub fn remove_node(&mut self, node_id: u64) -> Option<generated::AsgNode> {
        // If this was the root, unset it
        if self.root_id == Some(node_id) {
            self.root_id = None;
        }
        
        self.nodes.remove(&node_id)
    }
    
    /// Converts the internal graph structure to the protobuf representation.
    pub fn into_proto(self) -> generated::AsgGraph {
        generated::AsgGraph {
            nodes: self.nodes.into_values().collect(),
            root_node_id: self.root_id.unwrap_or(0),
        }
    }
    
    /// Creates an AsgGraph from a protobuf representation.
    pub fn from_proto(proto: generated::AsgGraph) -> Result<Self> {
        let mut graph = Self::new();
        
        // Insert all nodes into the map
        for node in proto.nodes {
            let node_id = node.node_id;
            if node_id >= graph.next_id.load(Ordering::SeqCst) {
                graph.next_id.store(node_id + 1, Ordering::SeqCst);
            }
            graph.nodes.insert(node_id, node);
        }
        
        // Set the root node if it's valid
        if proto.root_node_id != 0 && graph.nodes.contains_key(&proto.root_node_id) {
            graph.root_id = Some(proto.root_node_id);
        }
        
        Ok(graph)
    }
    
    // Helper methods for accessing specific node types
    
    /// Gets a lambda node by ID.
    pub fn get_lambda(&self, node_id: u64) -> Result<&generated::TermLambda> {
        let node = self.get_node(node_id).ok_or(Error::NodeNotFound(node_id))?;
        
        if let Some(generated::asg_node::Content::TermLambda(lambda)) = &node.content {
            Ok(lambda)
        } else {
            Err(Error::NodeTypeMismatch {
                expected: "TermLambda".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Gets a variable node by ID.
    pub fn get_variable(&self, node_id: u64) -> Result<&generated::TermVariable> {
        let node = self.get_node(node_id).ok_or(Error::NodeNotFound(node_id))?;
        
        if let Some(generated::asg_node::Content::TermVariable(var)) = &node.content {
            Ok(var)
        } else {
            Err(Error::NodeTypeMismatch {
                expected: "TermVariable".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    /// Gets an application node by ID.
    pub fn get_application(&self, node_id: u64) -> Result<&generated::TermApplication> {
        let node = self.get_node(node_id).ok_or(Error::NodeNotFound(node_id))?;
        
        if let Some(generated::asg_node::Content::TermApplication(app)) = &node.content {
            Ok(app)
        } else {
            Err(Error::NodeTypeMismatch {
                expected: "TermApplication".to_string(),
                found: format!("{:?}", node.content),
            })
        }
    }
    
    // Additional helper methods for other node types would follow a similar pattern
}

// Implement conversion from specific node content types to the generated oneof content type
impl From<generated::TermLambda> for generated::asg_node::Content {
    fn from(lambda: generated::TermLambda) -> Self {
        generated::asg_node::Content::TermLambda(lambda)
    }
}

impl From<generated::TermVariable> for generated::asg_node::Content {
    fn from(var: generated::TermVariable) -> Self {
        generated::asg_node::Content::TermVariable(var)
    }
}

impl From<generated::TermApplication> for generated::asg_node::Content {
    fn from(app: generated::TermApplication) -> Self {
        generated::asg_node::Content::TermApplication(app)
    }
}

impl From<generated::LiteralInt> for generated::asg_node::Content {
    fn from(lit: generated::LiteralInt) -> Self {
        generated::asg_node::Content::LiteralInt(lit)
    }
}

impl From<generated::LiteralBool> for generated::asg_node::Content {
    fn from(lit: generated::LiteralBool) -> Self {
        generated::asg_node::Content::LiteralBool(lit)
    }
}

impl From<generated::PrimitiveOp> for generated::asg_node::Content {
    fn from(op: generated::PrimitiveOp) -> Self {
        generated::asg_node::Content::PrimitiveOp(op)
    }
}

impl From<generated::TermRef> for generated::asg_node::Content {
    fn from(r: generated::TermRef) -> Self {
        generated::asg_node::Content::TermRef(r)
    }
}

impl From<generated::TermDeref> for generated::asg_node::Content {
    fn from(d: generated::TermDeref) -> Self {
        generated::asg_node::Content::TermDeref(d)
    }
}

impl From<generated::TermAssign> for generated::asg_node::Content {
    fn from(a: generated::TermAssign) -> Self {
        generated::asg_node::Content::TermAssign(a)
    }
}

impl From<generated::EffectPerform> for generated::asg_node::Content {
    fn from(e: generated::EffectPerform) -> Self {
        generated::asg_node::Content::EffectPerform(e)
    }
}

impl From<generated::ProofObligation> for generated::asg_node::Content {
    fn from(p: generated::ProofObligation) -> Self {
        generated::asg_node::Content::ProofObligation(p)
    }
}

impl From<generated::TypeNode> for generated::asg_node::Content {
    fn from(t: generated::TypeNode) -> Self {
        generated::asg_node::Content::TypeNode(t)
    }
}

impl From<generated::Metadata> for generated::asg_node::Content {
    fn from(m: generated::Metadata) -> Self {
        generated::asg_node::Content::Metadata(m)
    }
}