//! Hash computation for ASG nodes and graphs.

use blake3::Hasher;

use crate::generated;

/// The type used for hash digests (32 bytes for BLAKE3).
pub type HashDigest = [u8; 32];

/// Computes a hash for an ASG node.
pub fn hash_node(node: &generated::AsgNode) -> HashDigest {
    let canonical_bytes = canonicalize_node(node);
    *blake3::hash(&canonical_bytes).as_bytes()
}

/// Produces a canonical binary representation of a node for hashing.
pub fn canonicalize_node(node: &generated::AsgNode) -> Vec<u8> {
    let mut hasher = Hasher::new();
    
    // Hash the node type
    hasher.update(&[node.r#type as u8]);
    
    // Hash the content based on the node type
    if let Some(content) = &node.content {
        match content {
            generated::asg_node::Content::TermVariable(var) => {
                hasher.update(b"TermVariable");
                hasher.update(var.name.as_bytes());
                hasher.update(&var.definition_node_id.to_le_bytes());
            }
            generated::asg_node::Content::TermLambda(lambda) => {
                hasher.update(b"TermLambda");
                hasher.update(&lambda.binder_variable_node_id.to_le_bytes());
                hasher.update(&lambda.body_node_id.to_le_bytes());
                hasher.update(&lambda.type_annotation_id.to_le_bytes());
            }
            generated::asg_node::Content::TermApplication(app) => {
                hasher.update(b"TermApplication");
                hasher.update(&app.function_node_id.to_le_bytes());
                hasher.update(&app.argument_node_id.to_le_bytes());
            }
            generated::asg_node::Content::LiteralInt(lit) => {
                hasher.update(b"LiteralInt");
                hasher.update(&lit.value.to_le_bytes());
            }
            generated::asg_node::Content::LiteralBool(lit) => {
                hasher.update(b"LiteralBool");
                hasher.update(&[lit.value as u8]);
            }
            generated::asg_node::Content::PrimitiveOp(op) => {
                hasher.update(b"PrimitiveOp");
                hasher.update(op.op_name.as_bytes());
                for arg_id in &op.argument_node_ids {
                    hasher.update(&arg_id.to_le_bytes());
                }
            }
            generated::asg_node::Content::TermRef(r) => {
                hasher.update(b"TermRef");
                hasher.update(&r.init_value_node_id.to_le_bytes());
            }
            generated::asg_node::Content::TermDeref(d) => {
                hasher.update(b"TermDeref");
                hasher.update(&d.ref_node_id.to_le_bytes());
            }
            generated::asg_node::Content::TermAssign(a) => {
                hasher.update(b"TermAssign");
                hasher.update(&a.ref_node_id.to_le_bytes());
                hasher.update(&a.value_node_id.to_le_bytes());
            }
            generated::asg_node::Content::EffectPerform(e) => {
                hasher.update(b"EffectPerform");
                hasher.update(e.effect_name.as_bytes());
                hasher.update(&e.value_node_id.to_le_bytes());
            }
            generated::asg_node::Content::ProofObligation(p) => {
                hasher.update(b"ProofObligation");
                hasher.update(p.description.as_bytes());
                hasher.update(&p.related_code_node_id.to_le_bytes());
                hasher.update(&[p.status as u8]);
            }
            generated::asg_node::Content::TypeNode(t) => {
                hasher.update(b"TypeNode");
                hasher.update(&t.node_id.to_le_bytes());
                hasher.update(&[t.type_kind as u8]);
                
                // Add specific type data
                if let Some(content) = &t.content {
                    match content {
                        generated::type_node::Content::TypeInt(_) => {
                            hasher.update(b"TypeInt");
                        }
                        generated::type_node::Content::TypeBool(_) => {
                            hasher.update(b"TypeBool");
                        }
                        generated::type_node::Content::TypeFunction(f) => {
                            hasher.update(b"TypeFunction");
                            hasher.update(&f.parameter_type_id.to_le_bytes());
                            hasher.update(&f.return_type_id.to_le_bytes());
                        }
                        generated::type_node::Content::TypeRef(r) => {
                            hasher.update(b"TypeRef");
                            hasher.update(&r.element_type_id.to_le_bytes());
                        }
                        generated::type_node::Content::TypeUnit(_) => {
                            hasher.update(b"TypeUnit");
                        }
                    }
                }
            }
            generated::asg_node::Content::Metadata(m) => {
                hasher.update(b"Metadata");
                hasher.update(&m.node_id.to_le_bytes());
                
                // Hash source location if present
                if let Some(loc) = &m.source_location {
                    hasher.update(loc.filename.as_bytes());
                    hasher.update(&loc.start_line.to_le_bytes());
                    hasher.update(&loc.start_col.to_le_bytes());
                    hasher.update(&loc.end_line.to_le_bytes());
                    hasher.update(&loc.end_col.to_le_bytes());
                }
                
                // Hash annotations
                for annotation_id in &m.annotation_ids {
                    hasher.update(&annotation_id.to_le_bytes());
                }
            }
        }
    }
    
    hasher.finalize().as_bytes().to_vec()
}

/// Computes a hash for an entire ASG.
pub fn hash_graph(graph: &crate::AsgGraph) -> HashDigest {
    let mut hasher = Hasher::new();
    
    // Get a sorted list of nodes for consistent hashing
    let mut node_ids: Vec<u64> = graph.nodes().keys().cloned().collect();
    node_ids.sort();
    
    // Hash each node in order
    for node_id in node_ids {
        if let Some(node) = graph.get_node(node_id) {
            let node_hash = hash_node(node);
            hasher.update(&node_hash);
        }
    }
    
    // Also hash the root node ID if present
    if let Some(root_id) = graph.root_id() {
        hasher.update(&root_id.to_le_bytes());
    }
    
    *hasher.finalize().as_bytes()
}