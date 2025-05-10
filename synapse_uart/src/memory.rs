//! UART Memory Manager
//!
//! Provides explicit memory management with quantitative type-like guarantees.
//! Implements lifetime tracking, memory regions, and safe reclamation patterns.

use crate::{RuntimeError, Result};
use std::sync::{Arc, RwLock, Mutex, atomic::{AtomicUsize, Ordering}};
use std::collections::{HashMap, HashSet};
use std::ptr::NonNull;
use std::alloc::{Layout, alloc, dealloc};
use std::marker::PhantomData;
use std::fmt;
use std::mem;
use std::ops::{Deref, DerefMut};

/// Memory manager configuration
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// Enable memory tracing
    pub trace_allocations: bool,
    
    /// Enable memory leak detection
    pub leak_detection: bool,
    
    /// Enable region-based allocation
    pub region_allocation: bool,
    
    /// Enable reference counting
    pub reference_counting: bool,
    
    /// Default allocation strategy
    pub default_strategy: AllocationStrategy,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            trace_allocations: true,
            leak_detection: true,
            region_allocation: true,
            reference_counting: true,
            default_strategy: AllocationStrategy::Global,
        }
    }
}

/// Allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// Global allocator
    Global,
    
    /// Region-based allocation
    Region,
    
    /// Arena allocation
    Arena,
    
    /// Thread-local allocation
    ThreadLocal,
}

impl fmt::Display for AllocationStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Global => write!(f, "Global"),
            Self::Region => write!(f, "Region"),
            Self::Arena => write!(f, "Arena"),
            Self::ThreadLocal => write!(f, "ThreadLocal"),
        }
    }
}

/// Memory resource with explicit ownership
pub struct QBox<T> {
    /// Pointer to the allocated object
    ptr: NonNull<T>,
    
    /// Phantom data for type
    _marker: PhantomData<T>,
    
    /// Memory manager reference
    manager: Arc<MemoryManager>,
    
    /// Memory block ID
    block_id: usize,
}

impl<T> QBox<T> {
    /// Create a new QBox
    fn new(ptr: NonNull<T>, manager: Arc<MemoryManager>, block_id: usize) -> Self {
        Self {
            ptr,
            _marker: PhantomData,
            manager,
            block_id,
        }
    }
    
    /// Get a reference to the manager
    pub fn manager(&self) -> &MemoryManager {
        &self.manager
    }
    
    /// Get the block ID
    pub fn block_id(&self) -> usize {
        self.block_id
    }
    
    /// Convert to a raw pointer (unsafe, gives up ownership without deallocating)
    pub fn into_raw(mut self) -> *mut T {
        let ptr = self.ptr.as_ptr();
        // Prevent drop from running
        mem::forget(self);
        ptr
    }
    
    /// Create from a raw pointer (takes ownership)
    pub unsafe fn from_raw(ptr: *mut T, manager: Arc<MemoryManager>, block_id: usize) -> Self {
        Self::new(NonNull::new_unchecked(ptr), manager, block_id)
    }
}

impl<T> Deref for QBox<T> {
    type Target = T;
    
    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for QBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for QBox<T> {
    fn drop(&mut self) {
        // Mark block as deallocated
        let _ = self.manager.deallocate_block(self.block_id);
        
        // Drop and deallocate
        unsafe {
            ptr::drop_in_place(self.ptr.as_ptr());
            let layout = Layout::new::<T>();
            dealloc(self.ptr.as_ptr() as *mut u8, layout);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for QBox<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("QBox")
            .field("value", &**self)
            .field("block_id", &self.block_id)
            .finish()
    }
}

/// Memory region for grouped allocations
pub struct MemoryRegion {
    /// Region ID
    id: usize,
    
    /// Region name
    name: String,
    
    /// Memory manager reference
    manager: Arc<MemoryManager>,
    
    /// Allocated blocks in this region
    blocks: Mutex<HashSet<usize>>,
}

impl MemoryRegion {
    /// Create a new memory region
    fn new(id: usize, name: String, manager: Arc<MemoryManager>) -> Self {
        Self {
            id,
            name,
            manager,
            blocks: Mutex::new(HashSet::new()),
        }
    }
    
    /// Get region ID
    pub fn id(&self) -> usize {
        self.id
    }
    
    /// Get region name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Add a block to this region
    fn add_block(&self, block_id: usize) {
        let mut blocks = self.blocks.lock().unwrap();
        blocks.insert(block_id);
    }
    
    /// Remove a block from this region
    fn remove_block(&self, block_id: usize) {
        let mut blocks = self.blocks.lock().unwrap();
        blocks.remove(&block_id);
    }
    
    /// Get block count
    pub fn block_count(&self) -> usize {
        let blocks = self.blocks.lock().unwrap();
        blocks.len()
    }
    
    /// Allocate a new object in this region
    pub fn allocate<T>(&self, value: T) -> Result<QBox<T>> {
        let qbox = self.manager.allocate_in_region(value, self.id)?;
        Ok(qbox)
    }
}

impl Drop for MemoryRegion {
    fn drop(&mut self) {
        // Deallocate all blocks in the region
        let blocks = std::mem::take(&mut *self.blocks.lock().unwrap());
        for block_id in blocks {
            let _ = self.manager.deallocate_block(block_id);
        }
    }
}

/// Memory block tracking information
struct MemoryBlock {
    /// Block ID
    id: usize,
    
    /// Pointer address
    address: usize,
    
    /// Block size
    size: usize,
    
    /// Allocation time
    alloc_time: std::time::Instant,
    
    /// Region ID (0 = no region)
    region_id: usize,
    
    /// Allocation strategy used
    strategy: AllocationStrategy,
    
    /// Stack trace (if tracing enabled)
    stack_trace: Option<String>,
    
    /// Is block allocated
    allocated: bool,
}

/// Memory manager
pub struct MemoryManager {
    /// Allocated blocks
    blocks: RwLock<HashMap<usize, MemoryBlock>>,
    
    /// Memory regions
    regions: RwLock<HashMap<usize, Arc<MemoryRegion>>>,
    
    /// Configuration
    config: MemoryConfig,
    
    /// Next block ID
    next_block_id: AtomicUsize,
    
    /// Next region ID
    next_region_id: AtomicUsize,
    
    /// Total allocated memory
    total_allocated: AtomicUsize,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new(config: MemoryConfig) -> Result<Self> {
        Ok(Self {
            blocks: RwLock::new(HashMap::new()),
            regions: RwLock::new(HashMap::new()),
            config,
            next_block_id: AtomicUsize::new(1),
            next_region_id: AtomicUsize::new(1),
            total_allocated: AtomicUsize::new(0),
        })
    }
    
    /// Start the memory manager
    pub fn start(&self) -> Result<()> {
        // Initialize global region (ID 0) if region allocation is enabled
        if self.config.region_allocation {
            let global_region = MemoryRegion::new(
                0,
                "global".to_string(),
                Arc::new(self.clone()),
            );
            
            self.regions.write().unwrap().insert(0, Arc::new(global_region));
        }
        
        Ok(())
    }
    
    /// Shutdown the memory manager
    pub fn shutdown(&self) -> Result<()> {
        // Check for leaks if leak detection is enabled
        if self.config.leak_detection {
            self.check_leaks()?;
        }
        
        Ok(())
    }
    
    /// Create a new memory region
    pub fn create_region<S: Into<String>>(&self, name: S) -> Result<Arc<MemoryRegion>> {
        if !self.config.region_allocation {
            return Err(RuntimeError::MemoryError("Region allocation is disabled".to_string()));
        }
        
        let id = self.next_region_id.fetch_add(1, Ordering::SeqCst);
        let name = name.into();
        
        let region = Arc::new(MemoryRegion::new(
            id,
            name,
            Arc::new(self.clone()),
        ));
        
        self.regions.write().unwrap().insert(id, Arc::clone(&region));
        
        Ok(region)
    }
    
    /// Allocate a new object
    pub fn allocate<T>(&self, value: T) -> Result<QBox<T>> {
        self.allocate_with_strategy(value, self.config.default_strategy)
    }
    
    /// Allocate with a specific strategy
    pub fn allocate_with_strategy<T>(&self, value: T, strategy: AllocationStrategy) -> Result<QBox<T>> {
        // Allocate memory for the object
        let layout = Layout::new::<T>();
        let ptr = unsafe { alloc(layout) as *mut T };
        
        if ptr.is_null() {
            return Err(RuntimeError::MemoryError("Memory allocation failed".to_string()));
        }
        
        // Write the value
        unsafe {
            ptr::write(ptr, value);
        }
        
        // Generate block ID
        let block_id = self.next_block_id.fetch_add(1, Ordering::SeqCst);
        
        // Create memory block
        let block = MemoryBlock {
            id: block_id,
            address: ptr as usize,
            size: layout.size(),
            alloc_time: std::time::Instant::now(),
            region_id: 0, // No region
            strategy,
            stack_trace: if self.config.trace_allocations {
                // In a real implementation, we would capture stack trace here
                Some("[stack trace placeholder]".to_string())
            } else {
                None
            },
            allocated: true,
        };
        
        // Register the block
        self.blocks.write().unwrap().insert(block_id, block);
        
        // Update total allocated memory
        self.total_allocated.fetch_add(layout.size(), Ordering::SeqCst);
        
        // Create QBox
        Ok(QBox::new(
            NonNull::new(ptr).unwrap(),
            Arc::new(self.clone()),
            block_id,
        ))
    }
    
    /// Allocate in a specific region
    pub fn allocate_in_region<T>(&self, value: T, region_id: usize) -> Result<QBox<T>> {
        // Check if region exists
        if !self.region_exists(region_id) {
            return Err(RuntimeError::MemoryError(format!("Region {} does not exist", region_id)));
        }
        
        // Allocate with region strategy
        let qbox = self.allocate_with_strategy(value, AllocationStrategy::Region)?;
        
        // Update block region ID
        if let Some(block) = self.blocks.write().unwrap().get_mut(&qbox.block_id) {
            block.region_id = region_id;
        }
        
        // Add block to region
        if let Some(region) = self.regions.read().unwrap().get(&region_id) {
            region.add_block(qbox.block_id);
        }
        
        Ok(qbox)
    }
    
    /// Deallocate a block
    fn deallocate_block(&self, block_id: usize) -> Result<()> {
        let mut blocks = self.blocks.write().unwrap();
        
        if let Some(block) = blocks.get_mut(&block_id) {
            if !block.allocated {
                return Err(RuntimeError::MemoryError(format!("Block {} already deallocated", block_id)));
            }
            
            // Mark as deallocated
            block.allocated = false;
            
            // Update total allocated memory
            self.total_allocated.fetch_sub(block.size, Ordering::SeqCst);
            
            // Remove from region
            if block.region_id != 0 {
                if let Some(region) = self.regions.read().unwrap().get(&block.region_id) {
                    region.remove_block(block_id);
                }
            }
            
            // Remove from blocks map
            blocks.remove(&block_id);
            
            Ok(())
        } else {
            Err(RuntimeError::MemoryError(format!("Block {} not found", block_id)))
        }
    }
    
    /// Check if region exists
    fn region_exists(&self, region_id: usize) -> bool {
        self.regions.read().unwrap().contains_key(&region_id)
    }
    
    /// Check for memory leaks
    pub fn check_leaks(&self) -> Result<()> {
        let blocks = self.blocks.read().unwrap();
        
        let leak_count = blocks.values().filter(|b| b.allocated).count();
        
        if leak_count > 0 {
            let leak_size: usize = blocks.values()
                .filter(|b| b.allocated)
                .map(|b| b.size)
                .sum();
            
            return Err(RuntimeError::MemoryError(format!(
                "Memory leak detected: {} blocks, {} bytes",
                leak_count,
                leak_size
            )));
        }
        
        Ok(())
    }
    
    /// Get total allocated memory
    pub fn total_allocated(&self) -> usize {
        self.total_allocated.load(Ordering::SeqCst)
    }
    
    /// Get block count
    pub fn block_count(&self) -> usize {
        self.blocks.read().unwrap().len()
    }
}

impl Clone for MemoryManager {
    fn clone(&self) -> Self {
        Self {
            blocks: RwLock::new(HashMap::new()),
            regions: RwLock::new(HashMap::new()),
            config: self.config.clone(),
            next_block_id: AtomicUsize::new(self.next_block_id.load(Ordering::SeqCst)),
            next_region_id: AtomicUsize::new(self.next_region_id.load(Ordering::SeqCst)),
            total_allocated: AtomicUsize::new(self.total_allocated.load(Ordering::SeqCst)),
        }
    }
}

use std::ptr;