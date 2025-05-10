//! Synapse Holographic Debugger public API
//! 
//! Provides the runtime instrumentation points and external API

use crate::{
    TraceStream, TraceEvent, EventCategory, SourceLocation,
    TraceQuery, Result, StateReconstructor, MemoryTraceStorage
};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicU64, Ordering}};
use serde_json::json;
use std::thread;

/// Global thread/actor ID counter
static THREAD_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Global trace manager
pub struct TraceManager {
    /// Current active trace
    active_trace: Arc<RwLock<Option<Arc<RwLock<TraceStream>>>>>,
    
    /// Thread-local trace contexts
    thread_contexts: Mutex<HashMap<u64, ThreadContext>>,
}

impl TraceManager {
    /// Get the global trace manager instance
    pub fn instance() -> &'static TraceManager {
        lazy_static::lazy_static! {
            static ref INSTANCE: TraceManager = TraceManager::new();
        }
        
        &INSTANCE
    }
    
    /// Create a new trace manager
    fn new() -> Self {
        Self {
            active_trace: Arc::new(RwLock::new(None)),
            thread_contexts: Mutex::new(HashMap::new()),
        }
    }
    
    /// Start a new trace session
    pub fn start_trace(&self) -> Result<Arc<RwLock<TraceStream>>> {
        let trace = Arc::new(RwLock::new(
            TraceStream::new(Some(Box::new(MemoryTraceStorage::new())))
        ));
        
        let mut active = self.active_trace.write().unwrap();
        *active = Some(Arc::clone(&trace));
        
        Ok(trace)
    }
    
    /// Stop the current trace session
    pub fn stop_trace(&self) -> Result<Arc<RwLock<TraceStream>>> {
        let mut active = self.active_trace.write().unwrap();
        
        if let Some(trace) = active.take() {
            // Clear all thread contexts
            let mut contexts = self.thread_contexts.lock().unwrap();
            contexts.clear();
            
            Ok(trace)
        } else {
            Err(crate::TracingError::RecordingFailed("No active trace session".to_string()))
        }
    }
    
    /// Get or create a context for the current thread
    pub fn current_thread_context(&self) -> Result<ThreadContext> {
        let thread_id = thread::current().id().as_u64_unique();
        
        let mut contexts = self.thread_contexts.lock().unwrap();
        
        if !contexts.contains_key(&thread_id) {
            // Create new context
            let next_id = THREAD_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
            let ctx = ThreadContext {
                thread_id,
                actor_id: next_id,
                trace_manager: self,
                call_stack: Vec::new(),
            };
            
            contexts.insert(thread_id, ctx.clone());
            return Ok(ctx);
        }
        
        Ok(contexts.get(&thread_id).unwrap().clone())
    }
    
    /// Record a trace event in the active trace
    pub fn record_event(&self,
                        category: EventCategory,
                        causal_parent_id: Option<u64>,
                        source_node_id: Option<u64>,
                        thread_id: u64,
                        data: serde_json::Value) -> Result<u64> {
        let active = self.active_trace.read().unwrap();
        
        if let Some(trace) = &*active {
            let mut trace = trace.write().unwrap();
            trace.record_event(category, causal_parent_id, source_node_id, thread_id, data)
        } else {
            Err(crate::TracingError::RecordingFailed("No active trace session".to_string()))
        }
    }
}

/// Thread-local tracing context
#[derive(Clone)]
pub struct ThreadContext {
    /// Native thread ID
    thread_id: u64,
    
    /// Logical actor/thread ID used in traces
    actor_id: u64,
    
    /// Reference to the global trace manager
    trace_manager: &'static TraceManager,
    
    /// Call stack for this thread
    call_stack: Vec<u64>, // Stack of event IDs
}

impl ThreadContext {
    /// Record a function call
    pub fn record_function_call(&mut self, 
                            function_name: &str,
                            args: HashMap<String, serde_json::Value>,
                            source_node_id: Option<u64>) -> Result<u64> {
        let data = json!({
            "function_name": function_name,
            "arguments": args,
        });
        
        let parent_id = self.call_stack.last().copied();
        
        let event_id = self.trace_manager.record_event(
            EventCategory::FunctionCall,
            parent_id,
            source_node_id,
            self.actor_id,
            data
        )?;
        
        self.call_stack.push(event_id);
        Ok(event_id)
    }
    
    /// Record a function return
    pub fn record_function_return(&mut self,
                              result: Option<serde_json::Value>,
                              result_name: Option<&str>,
                              source_node_id: Option<u64>) -> Result<u64> {
        let call_event_id = self.call_stack.pop();
        
        let data = json!({
            "result_value": result.unwrap_or(json!(null)),
            "result_name": result_name,
        });
        
        self.trace_manager.record_event(
            EventCategory::FunctionReturn,
            call_event_id,
            source_node_id,
            self.actor_id,
            data
        )
    }
    
    /// Record a variable assignment
    pub fn record_variable_assignment(&self,
                                  name: &str,
                                  value: serde_json::Value,
                                  is_local: bool,
                                  source_node_id: Option<u64>) -> Result<u64> {
        let data = json!({
            "name": name,
            "value": value,
            "is_local": is_local,
        });
        
        let parent_id = self.call_stack.last().copied();
        
        self.trace_manager.record_event(
            EventCategory::VariableAssignment,
            parent_id,
            source_node_id,
            self.actor_id,
            data
        )
    }
    
    /// Record a memory allocation
    pub fn record_memory_allocation(&self,
                                address: u64,
                                size: usize,
                                initial_value: serde_json::Value,
                                source_node_id: Option<u64>) -> Result<u64> {
        let data = json!({
            "address": address,
            "size": size,
            "initial_value": initial_value,
        });
        
        let parent_id = self.call_stack.last().copied();
        
        self.trace_manager.record_event(
            EventCategory::MemoryAllocation,
            parent_id,
            source_node_id,
            self.actor_id,
            data
        )
    }
    
    /// Record a memory deallocation
    pub fn record_memory_deallocation(&self,
                                  address: u64,
                                  source_node_id: Option<u64>) -> Result<u64> {
        let data = json!({
            "address": address,
        });
        
        let parent_id = self.call_stack.last().copied();
        
        self.trace_manager.record_event(
            EventCategory::MemoryDeallocation,
            parent_id,
            source_node_id,
            self.actor_id,
            data
        )
    }
    
    /// Record an effect being performed
    pub fn record_effect(&self,
                     effect_name: &str,
                     effect_data: serde_json::Value,
                     source_node_id: Option<u64>) -> Result<u64> {
        let data = json!({
            "effect_name": effect_name,
            "effect_data": effect_data,
        });
        
        let parent_id = self.call_stack.last().copied();
        
        self.trace_manager.record_event(
            EventCategory::EffectPerformed,
            parent_id,
            source_node_id,
            self.actor_id,
            data
        )
    }
}

use std::collections::HashMap;

/// Extension for thread::ThreadId to get a u64 representation
trait ThreadIdExt {
    fn as_u64_unique(&self) -> u64;
}

impl ThreadIdExt for thread::ThreadId {
    fn as_u64_unique(&self) -> u64 {
        // ThreadId doesn't expose its inner value, so we hash it to get a stable u64
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        std::hash::Hash::hash(self, &mut hasher);
        std::hash::Hasher::finish(&hasher)
    }
}