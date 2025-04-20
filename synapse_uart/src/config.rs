//! UART Runtime Configuration

use crate::scheduler::SchedulerConfig;
use crate::effects::EffectConfig;
use crate::memory::MemoryConfig;
use crate::fault::FaultConfig;

/// UART runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Scheduler configuration
    pub scheduler_config: SchedulerConfig,
    
    /// Effect system configuration
    pub effect_config: EffectConfig,
    
    /// Memory manager configuration
    pub memory_config: MemoryConfig,
    
    /// Fault manager configuration
    pub fault_config: FaultConfig,
    
    /// Enable debug tracing
    pub enable_debug_trace: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            scheduler_config: SchedulerConfig::default(),
            effect_config: EffectConfig::default(),
            memory_config: MemoryConfig::default(),
            fault_config: FaultConfig::default(),
            enable_debug_trace: true,
        }
    }
}

/// UART runtime profile presets
pub enum RuntimeProfile {
    /// General-purpose profile with balanced settings
    Standard,
    
    /// High-performance profile with optimized settings
    HighPerformance,
    
    /// Low-footprint profile for resource-constrained environments
    Embedded,
    
    /// Safety-focused profile with additional checks
    Safety,
    
    /// Debugging profile with extra instrumentation
    Debug,
}

impl RuntimeProfile {
    /// Get configuration for this profile
    pub fn config(&self) -> RuntimeConfig {
        match self {
            Self::Standard => RuntimeConfig::default(),
            
            Self::HighPerformance => {
                let mut config = RuntimeConfig::default();
                
                // Optimize for performance
                config.scheduler_config.worker_threads = num_cpus::get();
                config.scheduler_config.time_slice_ms = 20;
                config.memory_config.trace_allocations = false;
                config.memory_config.leak_detection = false;
                config.enable_debug_trace = false;
                
                config
            },
            
            Self::Embedded => {
                let mut config = RuntimeConfig::default();
                
                // Reduce footprint
                config.scheduler_config.worker_threads = 1;
                config.scheduler_config.adaptive_scheduling = false;
                config.memory_config.trace_allocations = false;
                config.memory_config.leak_detection = false;
                config.enable_debug_trace = false;
                
                config
            },
            
            Self::Safety => {
                let mut config = RuntimeConfig::default();
                
                // Add safety checks
                config.effect_config.strict_effects = true;
                config.memory_config.trace_allocations = true;
                config.memory_config.leak_detection = true;
                config.fault_config.recover_from_panics = true;
                config.fault_config.circuit_breakers = true;
                
                config
            },
            
            Self::Debug => {
                let mut config = RuntimeConfig::default();
                
                // Enable all debug features
                config.enable_debug_trace = true;
                config.memory_config.trace_allocations = true;
                config.memory_config.leak_detection = true;
                config.fault_config.fault_logging = true;
                
                config
            },
        }
    }
}

extern crate num_cpus;