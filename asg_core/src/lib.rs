//! ASG core: add propagation/access methods for effect metadata.

#[derive(Debug, Clone, Default)]
pub struct EffectMeta {
    pub effect_tags: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct AsgNode {
    pub node_id: u64,
    pub type_: NodeType,
    // ... other fields ...
    pub effect_meta: Option<EffectMeta>, // New: effect metadata
}