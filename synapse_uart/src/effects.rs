//! UART Effect System
//!
//! Provides first-class effect capabilities, handlers, and runtime enforcement.
//! Integrates with static type system effect annotations.

use crate::{RuntimeError, Result};
use std::collections::{HashSet, HashMap};
use std::sync::{Arc, RwLock, Mutex};
use std::any::{Any, TypeId};
use std::thread_local;
use std::fmt;

/// Effect capability token
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct EffectCap {
    /// Capability name
    name: String,
    
    /// Capability type (for capability system)
    cap_type: CapabilityType,
}

impl EffectCap {
    /// Create a new effect capability
    pub fn new<S: Into<String>>(name: S, cap_type: CapabilityType) -> Self {
        Self {
            name: name.into(),
            cap_type,
        }
    }
    
    /// Create IO capability
    pub fn io() -> Self {
        Self::new("IO", CapabilityType::IO)
    }
    
    /// Create State capability
    pub fn state() -> Self {
        Self::new("State", CapabilityType::State)
    }
    
    /// Create Exception capability
    pub fn exception() -> Self {
        Self::new("Exception", CapabilityType::Exception)
    }
    
    /// Create Network capability
    pub fn network() -> Self {
        Self::new("Network", CapabilityType::Network)
    }
    
    /// Create Custom capability
    pub fn custom<S: Into<String>>(name: S) -> Self {
        Self::new(name, CapabilityType::Custom)
    }
    
    /// Get capability name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get capability type
    pub fn cap_type(&self) -> CapabilityType {
        self.cap_type
    }
}

/// Capability types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityType {
    IO,
    State,
    Exception,
    Network,
    Custom,
}

impl fmt::Display for CapabilityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO => write!(f, "IO"),
            Self::State => write!(f, "State"),
            Self::Exception => write!(f, "Exception"),
            Self::Network => write!(f, "Network"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Effect handler signature
pub type EffectHandlerFn = Box<dyn Fn(&EffectInvocation) -> Result<Box<dyn Any>> + Send + Sync>;

/// Effect handler registration
struct EffectHandler {
    /// Handler function
    handler: EffectHandlerFn,
    
    /// Handler priority (lower is higher)
    priority: u8,
}

/// Effect invocation
pub struct EffectInvocation {
    /// Effect name
    effect_name: String,
    
    /// Operation name
    operation: String,
    
    /// Parameters
    parameters: Vec<Box<dyn Any + Send>>,
}

impl EffectInvocation {
    /// Create a new effect invocation
    pub fn new<S: Into<String>, T: Into<String>>(effect_name: S, operation: T) -> Self {
        Self {
            effect_name: effect_name.into(),
            operation: operation.into(),
            parameters: Vec::new(),
        }
    }
    
    /// Add a parameter
    pub fn with_param<T: 'static + Send>(mut self, param: T) -> Self {
        self.parameters.push(Box::new(param));
        self
    }
    
    /// Get effect name
    pub fn effect_name(&self) -> &str {
        &self.effect_name
    }
    
    /// Get operation name
    pub fn operation(&self) -> &str {
        &self.operation
    }
    
    /// Get parameter as specific type
    pub fn param<T: 'static>(&self, index: usize) -> Option<&T> {
        if index >= self.parameters.len() {
            return None;
        }
        
        self.parameters[index].downcast_ref::<T>()
    }
    
    /// Get parameter count
    pub fn param_count(&self) -> usize {
        self.parameters.len()
    }
}

/// Thread-local effect capability set
pub struct EffectSet {
    /// Effect capabilities in this set
    caps: HashSet<EffectCap>,
}

impl EffectSet {
    /// Create a new effect set from capabilities
    pub fn new(caps: &[EffectCap]) -> Self {
        Self {
            caps: caps.iter().cloned().collect(),
        }
    }
    
    /// Check if a capability is available
    pub fn has_capability(&self, cap: &EffectCap) -> bool {
        self.caps.contains(cap)
    }
    
    /// Get all capabilities
    pub fn capabilities(&self) -> impl Iterator<Item = &EffectCap> {
        self.caps.iter()
    }
}

thread_local! {
    static CURRENT_EFFECTS: std::cell::RefCell<Option<EffectSet>> = std::cell::RefCell::new(None);
}

/// Effect system configuration
#[derive(Debug, Clone)]
pub struct EffectConfig {
    /// Strict effect checking enabled
    pub strict_effects: bool,
    
    /// Default capabilities (if strict_effects is false)
    pub default_capabilities: Vec<EffectCap>,
    
    /// Capability scoping enabled
    pub capability_scoping: bool,
}

impl Default for EffectConfig {
    fn default() -> Self {
        Self {
            strict_effects: true,
            default_capabilities: vec![
                EffectCap::io(),
                EffectCap::state(),
                EffectCap::exception(),
            ],
            capability_scoping: true,
        }
    }
}

/// Effect system runtime
pub struct EffectSystem {
    /// Effect handlers
    handlers: RwLock<HashMap<String, Vec<EffectHandler>>>,
    
    /// Configuration
    config: EffectConfig,
}

impl EffectSystem {
    /// Create a new effect system with config
    pub fn new(config: EffectConfig) -> Result<Self> {
        Ok(Self {
            handlers: RwLock::new(HashMap::new()),
            config,
        })
    }
    
    /// Start the effect system
    pub fn start(&self) -> Result<()> {
        // Register built-in handlers
        self.register_handler("IO", "println", 0, Box::new(|inv| {
            if let Some(msg) = inv.param::<String>(0) {
                println!("{}", msg);
                Ok(Box::new(()))
            } else {
                Err(RuntimeError::EffectError("println requires a string parameter".to_string()))
            }
        }))?;
        
        Ok(())
    }
    
    /// Shutdown the effect system
    pub fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    /// Register an effect handler
    pub fn register_handler<S: Into<String>, T: Into<String>>(
        &self,
        effect_name: S,
        operation: T,
        priority: u8,
        handler: EffectHandlerFn,
    ) -> Result<()> {
        let effect_name = effect_name.into();
        let operation = operation.into();
        let key = format!("{}:{}", effect_name, operation);
        
        let mut handlers = self.handlers.write().unwrap();
        
        let effect_handlers = handlers.entry(key).or_insert_with(Vec::new);
        
        // Insert in priority order (lower number = higher priority)
        let handler = EffectHandler {
            handler,
            priority,
        };
        
        let pos = effect_handlers.iter()
            .position(|h| h.priority > priority)
            .unwrap_or(effect_handlers.len());
        
        effect_handlers.insert(pos, handler);
        
        Ok(())
    }
    
    /// Invoke an effect
    pub fn invoke(&self, invocation: EffectInvocation) -> Result<Box<dyn Any>> {
        // Check if current thread has the required capability
        let cap = EffectCap::new(invocation.effect_name(), CapabilityType::Custom);
        
        if self.config.strict_effects {
            let has_cap = CURRENT_EFFECTS.with(|effects| {
                if let Some(ref effects) = *effects.borrow() {
                    effects.has_capability(&cap)
                } else {
                    false
                }
            });
            
            if !has_cap {
                return Err(RuntimeError::CapabilityError(
                    format!("Missing capability for effect: {}", invocation.effect_name())
                ));
            }
        }
        
        // Find a handler
        let key = format!("{}:{}", invocation.effect_name(), invocation.operation());
        let handlers = self.handlers.read().unwrap();
        
        if let Some(effect_handlers) = handlers.get(&key) {
            if let Some(handler) = effect_handlers.first() {
                // Call the highest priority handler
                return (handler.handler)(&invocation);
            }
        }
        
        Err(RuntimeError::EffectError(format!(
            "No handler found for effect: {}:{}",
            invocation.effect_name(),
            invocation.operation()
        )))
    }
    
    /// Check if an effect is available
    pub fn has_effect<S: Into<String>, T: Into<String>>(
        &self,
        effect_name: S,
        operation: T,
    ) -> bool {
        let key = format!("{}:{}", effect_name.into(), operation.into());
        let handlers = self.handlers.read().unwrap();
        handlers.contains_key(&key)
    }
    
    /// Create a scope with specified effect capabilities
    pub fn with_effects<F, R>(&self, caps: &[EffectCap], f: F) -> Result<R>
    where
        F: FnOnce() -> Result<R>,
    {
        // Create effect set for this scope
        let effect_set = EffectSet::new(caps);
        
        // Save old effect set and set new one
        let old_effects = CURRENT_EFFECTS.with(|effects| {
            let old = effects.borrow().clone();
            *effects.borrow_mut() = Some(effect_set);
            old
        });
        
        // Execute function with new effect set
        let result = f();
        
        // Restore old effect set
        CURRENT_EFFECTS.with(|effects| {
            *effects.borrow_mut() = old_effects;
        });
        
        result
    }
}

/// Helper functions for effects
pub mod helpers {
    use super::*;
    
    /// Set current effect set
    pub fn set_current_effects(effects: EffectSet) {
        CURRENT_EFFECTS.with(|current| {
            *current.borrow_mut() = Some(effects);
        });
    }
    
    /// Get current effect set
    pub fn current_effects() -> Option<EffectSet> {
        CURRENT_EFFECTS.with(|current| {
            current.borrow().clone()
        })
    }
    
    /// Check if a capability is available
    pub fn has_capability(cap: &EffectCap) -> bool {
        CURRENT_EFFECTS.with(|effects| {
            if let Some(ref effects) = *effects.borrow() {
                effects.has_capability(cap)
            } else {
                false
            }
        })
    }
}