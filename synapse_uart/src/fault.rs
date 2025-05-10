//! UART Fault Tolerance System
//!
//! Provides failure isolation, recovery, and adaptation mechanisms
//! for resilient Synapse applications.

use crate::{RuntimeError, Result};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicBool, AtomicUsize, Ordering}};
use std::time::{Duration, Instant};
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::panic;

/// Fault manager configuration
#[derive(Debug, Clone)]
pub struct FaultConfig {
    /// Enable fault tolerance
    pub enabled: bool,
    
    /// Enable panic recovery
    pub recover_from_panics: bool,
    
    /// Max retry count
    pub max_retries: usize,
    
    /// Enable circuit breakers
    pub circuit_breakers: bool,
    
    /// Enable logging
    pub fault_logging: bool,
}

impl Default for FaultConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            recover_from_panics: true,
            max_retries: 3,
            circuit_breakers: true,
            fault_logging: true,
        }
    }
}

/// Fault type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FaultType {
    /// Panic (unhandled crash)
    Panic,
    
    /// Assertion failure
    Assertion,
    
    /// Effect failure (missing capability)
    EffectFailure,
    
    /// Memory failure (allocation, region, etc)
    MemoryFailure,
    
    /// Task failure (scheduler, timeout, etc)
    TaskFailure,
    
    /// External resource failure (IO, network, etc)
    ExternalResourceFailure,
    
    /// Custom failure
    Custom,
}

impl fmt::Display for FaultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Panic => write!(f, "Panic"),
            Self::Assertion => write!(f, "Assertion"),
            Self::EffectFailure => write!(f, "EffectFailure"),
            Self::MemoryFailure => write!(f, "MemoryFailure"),
            Self::TaskFailure => write!(f, "TaskFailure"),
            Self::ExternalResourceFailure => write!(f, "ExternalResourceFailure"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Fault record
#[derive(Debug, Clone)]
pub struct FaultRecord {
    /// Fault ID
    id: usize,
    
    /// Fault type
    fault_type: FaultType,
    
    /// Fault message
    message: String,
    
    /// Fault timestamp
    timestamp: Instant,
    
    /// Source location
    source_location: Option<SourceLocation>,
    
    /// Component name
    component: Option<String>,
}

/// Source location for fault
#[derive(Debug, Clone)]
pub struct SourceLocation {
    /// File name
    file: String,
    
    /// Line number
    line: u32,
    
    /// Column number
    column: u32,
}

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed (allowing operations)
    Closed,
    
    /// Circuit is open (blocking operations)
    Open,
    
    /// Circuit is half-open (testing recovery)
    HalfOpen,
}

/// Circuit breaker for fault isolation
pub struct CircuitBreaker {
    /// Circuit breaker name
    name: String,
    
    /// Current state
    state: Mutex<CircuitState>,
    
    /// Failure count
    failure_count: AtomicUsize,
    
    /// Success count (for half-open state)
    success_count: AtomicUsize,
    
    /// Last state change time
    last_state_change: Mutex<Instant>,
    
    /// Failure threshold before opening
    failure_threshold: usize,
    
    /// Success threshold before closing
    success_threshold: usize,
    
    /// Reset timeout (how long to stay open)
    reset_timeout: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            state: Mutex::new(CircuitState::Closed),
            failure_count: AtomicUsize::new(0),
            success_count: AtomicUsize::new(0),
            last_state_change: Mutex::new(Instant::now()),
            failure_threshold: 5,
            success_threshold: 3,
            reset_timeout: Duration::from_secs(30),
        }
    }
    
    /// Get the circuit breaker name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the current state
    pub fn state(&self) -> CircuitState {
        let mut state = *self.state.lock().unwrap();
        
        // Check if we should transition from Open to HalfOpen
        if state == CircuitState::Open {
            let last_change = *self.last_state_change.lock().unwrap();
            if last_change.elapsed() >= self.reset_timeout {
                // Transition to half-open
                state = CircuitState::HalfOpen;
                *self.state.lock().unwrap() = state;
                *self.last_state_change.lock().unwrap() = Instant::now();
            }
        }
        
        state
    }
    
    /// Execute an operation protected by the circuit breaker
    pub fn execute<F, T>(&self, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check circuit state
        let state = self.state();
        
        match state {
            CircuitState::Open => {
                // Circuit is open, fail fast
                Err(RuntimeError::InternalError(format!(
                    "Circuit breaker '{}' is open",
                    self.name
                )))
            }
            CircuitState::HalfOpen | CircuitState::Closed => {
                // Try the operation
                match operation() {
                    Ok(result) => {
                        // Operation succeeded
                        self.record_success();
                        Ok(result)
                    }
                    Err(err) => {
                        // Operation failed
                        self.record_failure();
                        Err(err)
                    }
                }
            }
        }
    }
    
    /// Record a successful operation
    fn record_success(&self) {
        let state = self.state();
        
        match state {
            CircuitState::HalfOpen => {
                // Increment success count
                let success_count = self.success_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                // Check if we should transition back to closed
                if success_count >= self.success_threshold {
                    *self.state.lock().unwrap() = CircuitState::Closed;
                    *self.last_state_change.lock().unwrap() = Instant::now();
                    self.success_count.store(0, Ordering::SeqCst);
                    self.failure_count.store(0, Ordering::SeqCst);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    /// Record a failed operation
    fn record_failure(&self) {
        let state = self.state();
        
        match state {
            CircuitState::Closed => {
                // Increment failure count
                let failure_count = self.failure_count.fetch_add(1, Ordering::SeqCst) + 1;
                
                // Check if we should open the circuit
                if failure_count >= self.failure_threshold {
                    *self.state.lock().unwrap() = CircuitState::Open;
                    *self.last_state_change.lock().unwrap() = Instant::now();
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open state opens the circuit again
                *self.state.lock().unwrap() = CircuitState::Open;
                *self.last_state_change.lock().unwrap() = Instant::now();
                self.success_count.store(0, Ordering::SeqCst);
            }
            _ => {}
        }
    }
    
    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        *self.state.lock().unwrap() = CircuitState::Closed;
        *self.last_state_change.lock().unwrap() = Instant::now();
        self.failure_count.store(0, Ordering::SeqCst);
        self.success_count.store(0, Ordering::SeqCst);
    }
}

/// Retry policy
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum retry count
    max_retries: usize,
    
    /// Initial backoff duration
    initial_backoff: Duration,
    
    /// Backoff factor
    backoff_factor: f64,
    
    /// Maximum backoff duration
    max_backoff: Duration,
    
    /// Retry only specific fault types
    fault_types: Option<Vec<FaultType>>,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_retries: usize) -> Self {
        Self {
            max_retries,
            initial_backoff: Duration::from_millis(100),
            backoff_factor: 2.0,
            max_backoff: Duration::from_secs(60),
            fault_types: None,
        }
    }
    
    /// Set initial backoff
    pub fn with_initial_backoff(mut self, duration: Duration) -> Self {
        self.initial_backoff = duration;
        self
    }
    
    /// Set backoff factor
    pub fn with_backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor;
        self
    }
    
    /// Set maximum backoff
    pub fn with_max_backoff(mut self, duration: Duration) -> Self {
        self.max_backoff = duration;
        self
    }
    
    /// Set fault types to retry
    pub fn with_fault_types(mut self, fault_types: Vec<FaultType>) -> Self {
        self.fault_types = Some(fault_types);
        self
    }
    
    /// Check if this policy applies to a fault
    fn applies_to(&self, fault_type: FaultType) -> bool {
        if let Some(ref types) = self.fault_types {
            types.contains(&fault_type)
        } else {
            true
        }
    }
    
    /// Calculate backoff duration for a retry
    fn calculate_backoff(&self, retry_count: usize) -> Duration {
        let backoff = self.initial_backoff.as_millis() as f64 * self.backoff_factor.powi(retry_count as i32);
        let max_backoff = self.max_backoff.as_millis() as f64;
        
        Duration::from_millis(backoff.min(max_backoff) as u64)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Fault manager
pub struct FaultManager {
    /// Configuration
    config: FaultConfig,
    
    /// Fault history
    fault_history: Mutex<VecDeque<FaultRecord>>,
    
    /// Circuit breakers
    circuit_breakers: RwLock<HashMap<String, Arc<CircuitBreaker>>>,
    
    /// Next fault ID
    next_fault_id: AtomicUsize,
    
    /// Fault handlers
    fault_handlers: RwLock<HashMap<FaultType, Vec<Box<dyn Fn(&FaultRecord) + Send + Sync>>>>,
}

impl FaultManager {
    /// Create a new fault manager
    pub fn new(config: FaultConfig) -> Result<Self> {
        let manager = Self {
            config,
            fault_history: Mutex::new(VecDeque::with_capacity(100)),
            circuit_breakers: RwLock::new(HashMap::new()),
            next_fault_id: AtomicUsize::new(1),
            fault_handlers: RwLock::new(HashMap::new()),
        };
        
        Ok(manager)
    }
    
    /// Start the fault manager
    pub fn start(&self) -> Result<()> {
        // Set up panic handler if enabled
        if self.config.enabled && self.config.recover_from_panics {
            let original_hook = panic::take_hook();
            
            panic::set_hook(Box::new(move |panic_info| {
                // Call the original hook
                original_hook(panic_info);
                
                // TODO: Record the panic in the fault manager
                // This would require a thread-local or global fault manager reference
            }));
        }
        
        Ok(())
    }
    
    /// Shutdown the fault manager
    pub fn shutdown(&self) -> Result<()> {
        Ok(())
    }
    
    /// Record a fault
    pub fn record_fault(&self, fault_type: FaultType, message: &str) -> Result<usize> {
        if !self.config.enabled {
            return Ok(0);
        }
        
        let fault_id = self.next_fault_id.fetch_add(1, Ordering::SeqCst);
        
        let fault = FaultRecord {
            id: fault_id,
            fault_type,
            message: message.to_string(),
            timestamp: Instant::now(),
            source_location: None,
            component: None,
        };
        
        // Add to history
        {
            let mut history = self.fault_history.lock().unwrap();
            
            // Keep history bounded
            if history.len() >= 100 {
                history.pop_front();
            }
            
            history.push_back(fault.clone());
        }
        
        // Notify handlers
        self.notify_handlers(&fault);
        
        // Log the fault
        if self.config.fault_logging {
            println!("FAULT: [{}] {}: {}", fault_id, fault_type, message);
        }
        
        Ok(fault_id)
    }
    
    /// Register a circuit breaker
    pub fn register_circuit_breaker<S: Into<String>>(&self, name: S) -> Result<Arc<CircuitBreaker>> {
        if !self.config.enabled || !self.config.circuit_breakers {
            return Err(RuntimeError::InternalError("Circuit breakers are disabled".to_string()));
        }
        
        let name = name.into();
        
        // Check if already exists
        {
            let breakers = self.circuit_breakers.read().unwrap();
            if let Some(breaker) = breakers.get(&name) {
                return Ok(Arc::clone(breaker));
            }
        }
        
        // Create new circuit breaker
        let breaker = Arc::new(CircuitBreaker::new(name.clone()));
        
        // Register it
        self.circuit_breakers.write().unwrap().insert(name, Arc::clone(&breaker));
        
        Ok(breaker)
    }
    
    /// Get a circuit breaker
    pub fn get_circuit_breaker<S: Into<String>>(&self, name: S) -> Option<Arc<CircuitBreaker>> {
        let name = name.into();
        let breakers = self.circuit_breakers.read().unwrap();
        breakers.get(&name).cloned()
    }
    
    /// Register a fault handler
    pub fn register_fault_handler<F>(&self, fault_type: FaultType, handler: F) -> Result<()>
    where
        F: Fn(&FaultRecord) + Send + Sync + 'static,
    {
        let mut handlers = self.fault_handlers.write().unwrap();
        let type_handlers = handlers.entry(fault_type).or_insert_with(Vec::new);
        type_handlers.push(Box::new(handler));
        
        Ok(())
    }
    
    /// Notify handlers of a fault
    fn notify_handlers(&self, fault: &FaultRecord) {
        let handlers = self.fault_handlers.read().unwrap();
        
        if let Some(type_handlers) = handlers.get(&fault.fault_type) {
            for handler in type_handlers {
                handler(fault);
            }
        }
    }
    
    /// Execute with retry
    pub fn with_retry<F, T>(&self, operation: F, policy: RetryPolicy) -> Result<T>
    where
        F: Fn() -> Result<T>,
    {
        let mut retry_count = 0;
        
        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    // Determine fault type from error
                    let fault_type = match &err {
                        RuntimeError::EffectError(_) => FaultType::EffectFailure,
                        RuntimeError::MemoryError(_) => FaultType::MemoryFailure,
                        RuntimeError::SchedulerError(_) => FaultType::TaskFailure,
                        _ => FaultType::Custom,
                    };
                    
                    // Check if policy applies to this fault
                    if !policy.applies_to(fault_type) {
                        return Err(err);
                    }
                    
                    // Record the fault
                    let _ = self.record_fault(fault_type, &format!("{}", err));
                    
                    // Check retry count
                    retry_count += 1;
                    if retry_count >= policy.max_retries {
                        return Err(err);
                    }
                    
                    // Calculate backoff
                    let backoff = policy.calculate_backoff(retry_count);
                    
                    // Log retry
                    if self.config.fault_logging {
                        println!("RETRY: {}/{} (backoff {:?})", retry_count, policy.max_retries, backoff);
                    }
                    
                    // Wait for backoff
                    std::thread::sleep(backoff);
                }
            }
        }
    }
    
    /// Execute with circuit breaker
    pub fn with_circuit_breaker<F, T>(&self, breaker_name: &str, operation: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        if !self.config.enabled || !self.config.circuit_breakers {
            return operation();
        }
        
        // Get or create circuit breaker
        let breaker = self.register_circuit_breaker(breaker_name)?;
        
        // Execute with circuit breaker
        breaker.execute(operation)
    }
    
    /// Get recent faults
    pub fn recent_faults(&self, count: usize) -> Vec<FaultRecord> {
        let history = self.fault_history.lock().unwrap();
        
        history.iter()
            .rev()
            .take(count)
            .cloned()
            .collect()
    }
}