//! Synapse Holographic Debugger - Core Tracing Engine (P4T1)
//!
//! This provides:
//! - Event trace format
//! - Trace capture/storage
//! - State reconstruction
//! - Causal relationship tracking

use asg_core::{AsgGraph, AsgNode};
use thiserror::Error;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Instant, Duration};

/// Errors that can occur during debugging/tracing
#[derive(Error, Debug)]
pub enum TracingError {
    #[error("Failed to record trace event: {0}")]
    RecordingFailed(String),
    
    #[error("Failed to reconstruct state: {0}")]
    ReconstructionFailed(String),
    
    #[error("Invalid event ID: {0}")]
    InvalidEventId(u64),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Result type for tracing operations
pub type Result<T> = std::result::Result<T, TracingError>;

/// Trace event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventCategory {
    FunctionCall,
    FunctionReturn,
    VariableAssignment,
    MemoryAllocation,
    MemoryDeallocation,
    EffectPerformed,
    MessageSend,
    MessageReceive,
    Error,
}

/// A single traced event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Unique ID for this event
    pub id: u64,
    
    /// Logical timestamp (order within execution)
    pub logical_time: u64,
    
    /// Wall clock timestamp (nanoseconds since trace start)
    pub timestamp_ns: u64,
    
    /// Event category
    pub category: EventCategory,
    
    /// ID of the event that caused this one (causal parent)
    pub causal_parent_id: Option<u64>,
    
    /// ASG node ID that generated this event
    pub source_node_id: Option<u64>,
    
    /// Location information for event
    pub location: Option<SourceLocation>,
    
    /// Thread or actor ID
    pub thread_id: u64,
    
    /// Event-specific data (serialized JSON)
    pub data: serde_json::Value,
}

/// Source code location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub file: String,
    pub line: u32,
    pub column: u32,
}

/// A stream of events, optionally written to storage 
pub struct TraceStream {
    /// All events in this trace
    events: Vec<TraceEvent>,
    
    /// Next event ID
    next_id: u64,
    
    /// Start time of the trace
    start_time: Instant,
    
    /// Current logical clock
    logical_clock: u64,
    
    /// Storage backend for events (optional)
    storage: Option<Box<dyn TraceStorage>>,
    
    /// Optional ASG for source mapping
    asg: Option<Arc<AsgGraph>>,
}

impl TraceStream {
    /// Create a new trace stream
    pub fn new(storage: Option<Box<dyn TraceStorage>>) -> Self {
        Self {
            events: Vec::new(),
            next_id: 1,
            start_time: Instant::now(),
            logical_clock: 0,
            storage,
            asg: None,
        }
    }
    
    /// Set the ASG for source mapping
    pub fn with_asg(mut self, asg: Arc<AsgGraph>) -> Self {
        self.asg = Some(asg);
        self
    }
    
    /// Record a new event in the trace
    pub fn record_event(&mut self, 
                        category: EventCategory,
                        causal_parent_id: Option<u64>,
                        source_node_id: Option<u64>,
                        thread_id: u64,
                        data: serde_json::Value) -> Result<u64> {
        // Increment logical clock
        self.logical_clock += 1;
        
        // Create the event
        let event = TraceEvent {
            id: self.next_id,
            logical_time: self.logical_clock,
            timestamp_ns: self.start_time.elapsed().as_nanos() as u64,
            category,
            causal_parent_id,
            source_node_id,
            location: self.source_location_for_node_id(source_node_id),
            thread_id,
            data,
        };
        
        // Store the event
        self.events.push(event.clone());
        self.next_id += 1;
        
        // Write to storage if available
        if let Some(storage) = &mut self.storage {
            storage.store_event(&event)?;
        }
        
        Ok(event.id)
    }
    
    /// Get a specific event by ID
    pub fn get_event(&self, event_id: u64) -> Option<&TraceEvent> {
        self.events.iter().find(|e| e.id == event_id)
    }
    
    /// Get all events in the trace
    pub fn all_events(&self) -> &[TraceEvent] {
        &self.events
    }
    
    /// Get events matching a filter function
    pub fn filter_events<F>(&self, filter: F) -> Vec<&TraceEvent>
    where F: Fn(&TraceEvent) -> bool {
        self.events.iter().filter(|e| filter(e)).collect()
    }
    
    /// Get all events within a time range
    pub fn events_in_time_range(&self, start_ns: u64, end_ns: u64) -> Vec<&TraceEvent> {
        self.events.iter()
            .filter(|e| e.timestamp_ns >= start_ns && e.timestamp_ns <= end_ns)
            .collect()
    }
    
    /// Get events causally related to a specific event
    pub fn causal_history(&self, event_id: u64) -> Result<Vec<&TraceEvent>> {
        let mut history = Vec::new();
        let mut current_id = Some(event_id);
        
        while let Some(id) = current_id {
            if let Some(event) = self.get_event(id) {
                history.push(event);
                current_id = event.causal_parent_id;
            } else {
                return Err(TracingError::InvalidEventId(id));
            }
        }
        
        // Reverse to get chronological order
        history.reverse();
        Ok(history)
    }
    
    /// Convert a node ID to a source location
    fn source_location_for_node_id(&self, node_id: Option<u64>) -> Option<SourceLocation> {
        if let Some(node_id) = node_id {
            if let Some(asg) = &self.asg {
                if let Some(node) = asg.nodes.get(&node_id) {
                    if let Some(metadata) = &node.metadata {
                        if let Some(loc) = &metadata.source_location {
                            return Some(SourceLocation {
                                file: loc.filename.clone(),
                                line: loc.start_line,
                                column: loc.start_col,
                            });
                        }
                    }
                }
            }
        }
        None
    }
}

/// Storage backend for trace events
pub trait TraceStorage: Send {
    /// Store a single event
    fn store_event(&mut self, event: &TraceEvent) -> Result<()>;
    
    /// Flush any buffered events to storage
    fn flush(&mut self) -> Result<()>;
    
    /// Query events matching a filter
    fn query_events(&self, query: &TraceQuery) -> Result<Vec<TraceEvent>>;
}

/// Simple in-memory trace storage
pub struct MemoryTraceStorage {
    events: Vec<TraceEvent>,
}

impl MemoryTraceStorage {
    /// Create a new in-memory trace storage
    pub fn new() -> Self {
        Self {
            events: Vec::new(),
        }
    }
}

impl TraceStorage for MemoryTraceStorage {
    fn store_event(&mut self, event: &TraceEvent) -> Result<()> {
        self.events.push(event.clone());
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
    
    fn query_events(&self, query: &TraceQuery) -> Result<Vec<TraceEvent>> {
        let mut results = Vec::new();
        
        for event in &self.events {
            if query.matches(event) {
                results.push(event.clone());
            }
        }
        
        Ok(results)
    }
}

/// File-based trace storage
pub struct FileTraceStorage {
    file: std::fs::File,
    buffer: Vec<TraceEvent>,
    buffer_size: usize,
}

impl FileTraceStorage {
    /// Create a new file-based trace storage
    pub fn new(path: &std::path::Path, buffer_size: usize) -> Result<Self> {
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?;
        
        Ok(Self {
            file,
            buffer: Vec::new(),
            buffer_size,
        })
    }
}

impl TraceStorage for FileTraceStorage {
    fn store_event(&mut self, event: &TraceEvent) -> Result<()> {
        self.buffer.push(event.clone());
        
        if self.buffer.len() >= self.buffer_size {
            self.flush()?;
        }
        
        Ok(())
    }
    
    fn flush(&mut self) -> Result<()> {
        use std::io::Write;
        
        for event in &self.buffer {
            let json = serde_json::to_string(event)?;
            writeln!(self.file, "{}", json)?;
        }
        
        self.buffer.clear();
        self.file.flush()?;
        
        Ok(())
    }
    
    fn query_events(&self, _query: &TraceQuery) -> Result<Vec<TraceEvent>> {
        // For simplicity, file storage doesn't support querying
        Err(TracingError::RecordingFailed("Querying not supported for file storage".to_string()))
    }
}

/// Query for filtering trace events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceQuery {
    /// Filter events by category
    pub categories: Option<Vec<EventCategory>>,
    
    /// Filter events by time range
    pub time_range: Option<(u64, u64)>,
    
    /// Filter events by source node IDs
    pub source_node_ids: Option<Vec<u64>>,
    
    /// Filter events by thread IDs
    pub thread_ids: Option<Vec<u64>>,
}

impl TraceQuery {
    /// Create a new empty query (matches all events)
    pub fn new() -> Self {
        Self {
            categories: None,
            time_range: None,
            source_node_ids: None,
            thread_ids: None,
        }
    }
    
    /// Check if an event matches this query
    pub fn matches(&self, event: &TraceEvent) -> bool {
        // Check category
        if let Some(categories) = &self.categories {
            if !categories.contains(&event.category) {
                return false;
            }
        }
        
        // Check time range
        if let Some((start, end)) = self.time_range {
            if event.timestamp_ns < start || event.timestamp_ns > end {
                return false;
            }
        }
        
        // Check source node ID
        if let Some(source_node_ids) = &self.source_node_ids {
            if let Some(node_id) = event.source_node_id {
                if !source_node_ids.contains(&node_id) {
                    return false;
                }
            } else {
                return false;
            }
        }
        
        // Check thread ID
        if let Some(thread_ids) = &self.thread_ids {
            if !thread_ids.contains(&event.thread_id) {
                return false;
            }
        }
        
        true
    }
}

/// State snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    /// Logical time of the snapshot
    pub logical_time: u64,
    
    /// Wall clock timestamp
    pub timestamp_ns: u64,
    
    /// Map of variable names to values
    pub variables: HashMap<String, serde_json::Value>,
    
    /// Map of memory addresses to values
    pub memory: HashMap<u64, serde_json::Value>,
    
    /// Call stack (most recent call last)
    pub call_stack: Vec<CallStackFrame>,
    
    /// Thread/actor states
    pub threads: HashMap<u64, ThreadState>,
}

/// Function call stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallStackFrame {
    /// Function name
    pub function_name: String,
    
    /// Source location
    pub location: Option<SourceLocation>,
    
    /// ASG node ID
    pub node_id: Option<u64>,
    
    /// Local variables
    pub locals: HashMap<String, serde_json::Value>,
}

/// Thread or actor state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadState {
    /// Thread ID
    pub id: u64,
    
    /// Thread name
    pub name: Option<String>,
    
    /// Current status
    pub status: ThreadStatus,
    
    /// Call stack (most recent call last)
    pub call_stack: Vec<CallStackFrame>,
}

/// Thread status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreadStatus {
    Running,
    Blocked,
    Waiting,
    Terminated,
}

/// State reconstruction engine
pub struct StateReconstructor {
    /// Trace to reconstruct from
    trace: Arc<RwLock<TraceStream>>,
    
    /// Cache of reconstructed states
    state_cache: Mutex<BTreeMap<u64, Arc<StateSnapshot>>>,
}

impl StateReconstructor {
    /// Create a new state reconstructor
    pub fn new(trace: Arc<RwLock<TraceStream>>) -> Self {
        Self {
            trace,
            state_cache: Mutex::new(BTreeMap::new()),
        }
    }
    
    /// Reconstruct state at a specific logical time
    pub fn reconstruct_at(&self, logical_time: u64) -> Result<Arc<StateSnapshot>> {
        // Check cache first
        {
            let cache = self.state_cache.lock().unwrap();
            if let Some(snapshot) = cache.get(&logical_time) {
                return Ok(Arc::clone(snapshot));
            }
        }
        
        // Not in cache, reconstruct from events
        let trace = self.trace.read().unwrap();
        
        // Get all events up to logical_time
        let events: Vec<_> = trace.all_events()
            .iter()
            .filter(|e| e.logical_time <= logical_time)
            .collect();
        
        if events.is_empty() {
            return Err(TracingError::ReconstructionFailed(
                format!("No events before logical time {}", logical_time)));
        }
        
        // Start with an empty state
        let mut state = StateSnapshot {
            logical_time,
            timestamp_ns: events.last().unwrap().timestamp_ns,
            variables: HashMap::new(),
            memory: HashMap::new(),
            call_stack: Vec::new(),
            threads: HashMap::new(),
        };
        
        // Apply each event to build up the state
        for event in &events {
            self.apply_event_to_state(&mut state, event)?;
        }
        
        // Cache the result
        let snapshot = Arc::new(state);
        {
            let mut cache = self.state_cache.lock().unwrap();
            cache.insert(logical_time, Arc::clone(&snapshot));
        }
        
        Ok(snapshot)
    }
    
    /// Apply a single event to a state
    fn apply_event_to_state(&self, state: &mut StateSnapshot, event: &TraceEvent) -> Result<()> {
        // Ensure thread exists
        if !state.threads.contains_key(&event.thread_id) {
            state.threads.insert(event.thread_id, ThreadState {
                id: event.thread_id,
                name: None,
                status: ThreadStatus::Running,
                call_stack: Vec::new(),
            });
        }
        
        match event.category {
            EventCategory::FunctionCall => {
                let data: FunctionCallData = serde_json::from_value(event.data.clone())
                    .map_err(|e| TracingError::SerializationError(e))?;
                
                // Add to call stack
                let thread = state.threads.get_mut(&event.thread_id).unwrap();
                thread.call_stack.push(CallStackFrame {
                    function_name: data.function_name,
                    location: event.location.clone(),
                    node_id: event.source_node_id,
                    locals: data.arguments,
                });
            },
            
            EventCategory::FunctionReturn => {
                // Pop from call stack
                let thread = state.threads.get_mut(&event.thread_id).unwrap();
                if !thread.call_stack.is_empty() {
                    thread.call_stack.pop();
                }
                
                // Store return value as a variable
                let data: FunctionReturnData = serde_json::from_value(event.data.clone())
                    .map_err(|e| TracingError::SerializationError(e))?;
                
                if let Some(name) = data.result_name {
                    state.variables.insert(name, data.result_value);
                }
            },
            
            EventCategory::VariableAssignment => {
                let data: VariableAssignmentData = serde_json::from_value(event.data.clone())
                    .map_err(|e| TracingError::SerializationError(e))?;
                
                if data.is_local && !state.threads.get(&event.thread_id).unwrap().call_stack.is_empty() {
                    // Local variable assignment
                    let thread = state.threads.get_mut(&event.thread_id).unwrap();
                    let frame = thread.call_stack.last_mut().unwrap();
                    frame.locals.insert(data.name, data.value);
                } else {
                    // Global variable assignment
                    state.variables.insert(data.name, data.value);
                }
            },
            
            EventCategory::MemoryAllocation => {
                let data: MemoryAllocationData = serde_json::from_value(event.data.clone())
                    .map_err(|e| TracingError::SerializationError(e))?;
                
                state.memory.insert(data.address, data.initial_value);
            },
            
            EventCategory::MemoryDeallocation => {
                let data: MemoryDeallocationData = serde_json::from_value(event.data.clone())
                    .map_err(|e| TracingError::SerializationError(e))?;
                
                state.memory.remove(&data.address);
            },
            
            // Other event types might not affect state directly
            _ => {}
        }
        
        // Update logical time
        state.logical_time = event.logical_time;
        state.timestamp_ns = event.timestamp_ns;
        
        Ok(())
    }
}

// Event-specific data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionCallData {
    function_name: String,
    arguments: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FunctionReturnData {
    result_name: Option<String>,
    result_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct VariableAssignmentData {
    name: String,
    value: serde_json::Value,
    is_local: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryAllocationData {
    address: u64,
    size: usize,
    initial_value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MemoryDeallocationData {
    address: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_trace_recording() {
        let mut trace = TraceStream::new(None);
        
        // Record a function call
        let call_data = serde_json::json!({
            "function_name": "test_function",
            "arguments": {
                "x": 42,
                "y": "hello"
            }
        });
        
        let call_id = trace.record_event(
            EventCategory::FunctionCall,
            None,
            Some(1),
            1,
            call_data
        ).unwrap();
        
        // Record a return
        let return_data = serde_json::json!({
            "result_name": "result",
            "result_value": 84
        });
        
        trace.record_event(
            EventCategory::FunctionReturn,
            Some(call_id),
            Some(2),
            1,
            return_data
        ).unwrap();
        
        // Verify events
        assert_eq!(trace.all_events().len(), 2);
        
        let event1 = trace.get_event(call_id).unwrap();
        assert_eq!(event1.category, EventCategory::FunctionCall);
        assert_eq!(event1.source_node_id, Some(1));
        
        let event2 = trace.all_events()[1];
        assert_eq!(event2.category, EventCategory::FunctionReturn);
        assert_eq!(event2.causal_parent_id, Some(call_id));
    }
    
    #[test]
    fn test_query() {
        let mut trace = TraceStream::new(None);
        
        // Record various events
        trace.record_event(
            EventCategory::FunctionCall,
            None,
            Some(1),
            1,
            serde_json::json!({})
        ).unwrap();
        
        trace.record_event(
            EventCategory::VariableAssignment,
            None,
            Some(2),
            1,
            serde_json::json!({})
        ).unwrap();
        
        trace.record_event(
            EventCategory::FunctionCall,
            None,
            Some(3),
            2,
            serde_json::json!({})
        ).unwrap();
        
        // Query by category
        let query = TraceQuery {
            categories: Some(vec![EventCategory::FunctionCall]),
            time_range: None,
            source_node_ids: None,
            thread_ids: None,
        };
        
        let results = trace.filter_events(|e| query.matches(e));
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].category, EventCategory::FunctionCall);
        assert_eq!(results[1].category, EventCategory::FunctionCall);
        
        // Query by thread
        let query = TraceQuery {
            categories: None,
            time_range: None,
            source_node_ids: None,
            thread_ids: Some(vec![2]),
        };
        
        let results = trace.filter_events(|e| query.matches(e));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].thread_id, 2);
    }
}