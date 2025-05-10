//! Synapse UART: Universal Adaptive Runtime (Phase 4)
//! 
//! A flexible, extensible runtime system for Synapse programs with:
//! - Adaptive task scheduling
//! - First-class effect handlers
//! - Quantitative memory management
//! - Fault tolerance mechanisms
//! - Seamless debugging integration

mod scheduler;
mod effects;
mod memory;
mod fault;
mod config;

pub use scheduler::*;
pub use effects::*;
pub use memory::*;
pub use fault::*;
pub use config::*;

use thiserror::Error;
use std::sync::Arc;

/// Runtime errors
#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Scheduler error: {0}")]
    SchedulerError(String),
    
    #[error("Effect error: {0}")]
    EffectError(String),
    
    #[error("Memory error: {0}")]
    MemoryError(String),
    
    #[error("Capability error: {0}")]
    CapabilityError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

/// Result type for runtime operations
pub type Result<T> = std::result::Result<T, RuntimeError>;

/// Main UART runtime instance
pub struct UartRuntime {
    /// Scheduler for task management
    scheduler: scheduler::Scheduler,
    
    /// Effect system for handling capabilities
    effect_system: effects::EffectSystem,
    
    /// Memory manager
    memory_manager: memory::MemoryManager,
    
    /// Fault tolerance manager
    fault_manager: fault::FaultManager,
    
    /// Runtime configuration
    config: config::RuntimeConfig,
    
    /// Debug trace integration (optional)
    debug_trace: Option<synapse_debugger::api::ThreadContext>,
}

impl UartRuntime {
    /// Create a new runtime with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(config::RuntimeConfig::default())
    }
    
    /// Create a new runtime with specific configuration
    pub fn with_config(config: config::RuntimeConfig) -> Result<Self> {
        let scheduler = scheduler::Scheduler::new(config.scheduler_config.clone())?;
        let effect_system = effects::EffectSystem::new(config.effect_config.clone())?;
        let memory_manager = memory::MemoryManager::new(config.memory_config.clone())?;
        let fault_manager = fault::FaultManager::new(config.fault_config.clone())?;
        
        // Initialize debug trace if enabled
        let debug_trace = if config.enable_debug_trace {
            match synapse_debugger::api::TraceManager::instance().current_thread_context() {
                Ok(ctx) => Some(ctx),
                Err(_) => None,
            }
        } else {
            None
        };
        
        Ok(Self {
            scheduler,
            effect_system,
            memory_manager,
            fault_manager,
            config,
            debug_trace,
        })
    }
    
    /// Start the runtime (blocking)
    pub fn start(&self) -> Result<()> {
        if let Some(ref debug) = self.debug_trace {
            // Record runtime start event
            let _ = debug.record_function_call("uart::start", std::collections::HashMap::new(), None);
        }
        
        // Initialize all subsystems
        self.scheduler.start()?;
        self.effect_system.start()?;
        self.memory_manager.start()?;
        self.fault_manager.start()?;
        
        // Main event loop
        self.scheduler.run_until_idle()?;
        
        if let Some(ref debug) = self.debug_trace {
            // Record runtime stop event
            let _ = debug.record_function_return(None, None, None);
        }
        
        Ok(())
    }
    
    /// Spawn a new task into the scheduler
    pub fn spawn<F>(&self, task: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        self.scheduler.spawn(task)
    }
    
    /// Spawn a task with specific effect capabilities
    pub fn spawn_with_effects<F>(&self, task: F, effects: Vec<EffectCap>) -> Result<TaskHandle>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        let task_wrapper = move || {
            // Create effect capability set
            let _effect_set = EffectSet::new(&effects);
            task()
        };
        
        self.scheduler.spawn(task_wrapper)
    }
    
    /// Get a reference to the effect system
    pub fn effect_system(&self) -> &effects::EffectSystem {
        &self.effect_system
    }
    
    /// Get a reference to the memory manager
    pub fn memory_manager(&self) -> &memory::MemoryManager {
        &self.memory_manager
    }
    
    /// Get a reference to the scheduler
    pub fn scheduler(&self) -> &scheduler::Scheduler {
        &self.scheduler
    }
    
    /// Get a reference to the fault manager
    pub fn fault_manager(&self) -> &fault::FaultManager {
        &self.fault_manager
    }
    
    /// Shut down the runtime gracefully
    pub fn shutdown(&self) -> Result<()> {
        self.scheduler.shutdown()?;
        self.effect_system.shutdown()?;
        self.memory_manager.shutdown()?;
        self.fault_manager.shutdown()?;
        Ok(())
    }
}

/// Global runtime accessor
pub fn global() -> &'static UartRuntime {
    use std::sync::OnceLock;
    static INSTANCE: OnceLock<UartRuntime> = OnceLock::new();
    
    INSTANCE.get_or_init(|| {
        UartRuntime::new().expect("Failed to initialize global UART runtime")
    })
}