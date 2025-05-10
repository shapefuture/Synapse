//! UART Task Scheduler with adaptive prioritization
//! 
//! Provides a work-stealing scheduler with priority queues,
//! adaptive scheduling policies, and integration with the effect system.

use crate::{RuntimeError, Result};
use crossbeam_deque::{Worker, Stealer, Injector};
use std::sync::{Arc, atomic::{AtomicBool, AtomicU64, Ordering}};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::{HashMap, BinaryHeap};
use std::cmp::Reverse;
use crossbeam_channel::{Sender, Receiver, bounded};
use parking_lot::{Mutex, RwLock};
use std::fmt;

/// Task identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TaskId(u64);

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Task-{}", self.0)
    }
}

/// Task priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl Priority {
    /// Convert from u8 to Priority
    pub fn from_u8(value: u8) -> Self {
        match value {
            0 => Self::Critical,
            1 => Self::High,
            2 => Self::Normal,
            3 => Self::Low,
            _ => Self::Background,
        }
    }
    
    /// Convert to u8
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// Task state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskState {
    Ready,
    Running,
    Blocked,
    Completed,
    Failed,
}

/// Task metadata
#[derive(Debug)]
pub struct TaskMetadata {
    /// Task identifier
    id: TaskId,
    
    /// Task name (optional)
    name: Option<String>,
    
    /// Current priority
    priority: Priority,
    
    /// Current state
    state: TaskState,
    
    /// Creation time
    created_at: Instant,
    
    /// Last execution time
    last_scheduled: Option<Instant>,
    
    /// Execution statistics
    stats: TaskStats,
}

/// Task execution statistics
#[derive(Debug, Default, Clone)]
pub struct TaskStats {
    /// Total run time
    total_run_time: Duration,
    
    /// Number of times scheduled
    schedule_count: u64,
    
    /// Number of preemptions
    preemption_count: u64,
    
    /// Number of yields
    yield_count: u64,
}

/// Task handle for client code
#[derive(Debug, Clone)]
pub struct TaskHandle {
    /// Task ID
    id: TaskId,
    
    /// Completion channel
    completion: Arc<CompletionSignal>,
    
    /// Task cancellation flag
    cancel: Arc<AtomicBool>,
    
    /// Scheduler control (for operations that need scheduler)
    scheduler_control: Arc<SchedulerControl>,
}

impl TaskHandle {
    /// Wait for task completion
    pub fn join(&self) -> Result<()> {
        self.completion.wait()
    }
    
    /// Try to join without blocking
    pub fn try_join(&self) -> Option<Result<()>> {
        self.completion.try_wait()
    }
    
    /// Cancel the task
    pub fn cancel(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
    
    /// Check if task is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }
    
    /// Get the task ID
    pub fn id(&self) -> TaskId {
        self.id
    }
    
    /// Change task priority
    pub fn set_priority(&self, priority: Priority) -> Result<()> {
        self.scheduler_control.update_priority(self.id, priority)
    }
}

/// Task completion signaling
#[derive(Debug)]
struct CompletionSignal {
    /// Sender for completion result
    sender: Mutex<Option<Sender<Result<()>>>>,
    
    /// Receiver for completion result
    receiver: Mutex<Option<Receiver<Result<()>>>>,
}

impl CompletionSignal {
    /// Create a new completion signal
    fn new() -> Self {
        let (sender, receiver) = bounded(1);
        Self {
            sender: Mutex::new(Some(sender)),
            receiver: Mutex::new(Some(receiver)),
        }
    }
    
    /// Signal completion with result
    fn signal(&self, result: Result<()>) {
        if let Some(sender) = self.sender.lock().take() {
            let _ = sender.send(result);
        }
    }
    
    /// Wait for completion
    fn wait(&self) -> Result<()> {
        if let Some(receiver) = self.receiver.lock().take() {
            match receiver.recv() {
                Ok(result) => result,
                Err(_) => Err(RuntimeError::SchedulerError("Task completion channel closed".to_string())),
            }
        } else {
            Err(RuntimeError::SchedulerError("Task already joined".to_string()))
        }
    }
    
    /// Try to wait without blocking
    fn try_wait(&self) -> Option<Result<()>> {
        let guard = self.receiver.lock();
        if let Some(ref receiver) = *guard {
            match receiver.try_recv() {
                Ok(result) => Some(result),
                Err(crossbeam_channel::TryRecvError::Empty) => None,
                Err(_) => Some(Err(RuntimeError::SchedulerError("Task completion channel closed".to_string()))),
            }
        } else {
            Some(Err(RuntimeError::SchedulerError("Task already joined".to_string())))
        }
    }
}

impl Default for CompletionSignal {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal task representation
struct Task {
    /// Task function
    func: Option<Box<dyn FnOnce() -> Result<()> + Send + 'static>>,
    
    /// Task metadata
    metadata: TaskMetadata,
    
    /// Completion signal
    completion: Arc<CompletionSignal>,
    
    /// Cancellation flag
    cancel: Arc<AtomicBool>,
}

impl Task {
    /// Create a new task
    fn new<F>(id: TaskId, func: F, priority: Priority) -> Self
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        let completion = Arc::new(CompletionSignal::new());
        let cancel = Arc::new(AtomicBool::new(false));
        
        Self {
            func: Some(Box::new(func)),
            metadata: TaskMetadata {
                id,
                name: None,
                priority,
                state: TaskState::Ready,
                created_at: Instant::now(),
                last_scheduled: None,
                stats: TaskStats::default(),
            },
            completion,
            cancel,
        }
    }
    
    /// Execute the task
    fn execute(mut self) -> Result<()> {
        if self.cancel.load(Ordering::SeqCst) {
            return Ok(());
        }
        
        let start = Instant::now();
        self.metadata.state = TaskState::Running;
        self.metadata.last_scheduled = Some(start);
        self.metadata.stats.schedule_count += 1;
        
        // Take the function to avoid borrowing issues
        let func = self.func.take().ok_or_else(|| {
            RuntimeError::SchedulerError("Task function already executed".to_string())
        })?;
        
        // Execute the task function
        let result = func();
        
        // Update statistics
        let elapsed = start.elapsed();
        self.metadata.stats.total_run_time += elapsed;
        
        // Update state and signal completion
        self.metadata.state = if result.is_ok() {
            TaskState::Completed
        } else {
            TaskState::Failed
        };
        
        self.completion.signal(result);
        
        Ok(())
    }
}

/// Scheduler configuration
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Number of worker threads
    pub worker_threads: usize,
    
    /// Work stealing enabled
    pub work_stealing: bool,
    
    /// Task time slice in milliseconds
    pub time_slice_ms: u64,
    
    /// Adaptive scheduling enabled
    pub adaptive_scheduling: bool,
    
    /// Maximum queued tasks per priority
    pub max_queued_per_priority: usize,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            worker_threads: num_cpus::get(),
            work_stealing: true,
            time_slice_ms: 10,
            adaptive_scheduling: true,
            max_queued_per_priority: 1000,
        }
    }
}

/// Task scheduler control interface
#[derive(Debug)]
struct SchedulerControl {
    /// Task map
    tasks: RwLock<HashMap<TaskId, Arc<RwLock<TaskMetadata>>>>,
    
    /// Command sender to scheduler
    cmd_sender: Sender<SchedulerCommand>,
    
    /// Next task ID
    next_task_id: AtomicU64,
    
    /// Scheduler shutdown flag
    shutdown: AtomicBool,
}

/// Commands to the scheduler
enum SchedulerCommand {
    /// Add task to the scheduler
    SubmitTask(Task),
    
    /// Update task priority
    UpdatePriority(TaskId, Priority),
    
    /// Cancel task
    CancelTask(TaskId),
    
    /// Shutdown scheduler
    Shutdown,
}

/// Main scheduler
pub struct Scheduler {
    /// Scheduler control
    control: Arc<SchedulerControl>,
    
    /// Scheduler configuration
    config: SchedulerConfig,
    
    /// Worker threads
    workers: Vec<Worker<Task>>,
    
    /// Stealers for work-stealing
    stealers: Vec<Stealer<Task>>,
    
    /// Global task injector (for load balancing)
    injector: Injector<Task>,
    
    /// Worker thread handles
    worker_threads: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl Scheduler {
    /// Create a new scheduler
    pub fn new(config: SchedulerConfig) -> Result<Self> {
        let (cmd_sender, cmd_receiver) = bounded(1000);
        
        let control = Arc::new(SchedulerControl {
            tasks: RwLock::new(HashMap::new()),
            cmd_sender,
            next_task_id: AtomicU64::new(1),
            shutdown: AtomicBool::new(false),
        });
        
        let workers = (0..config.worker_threads)
            .map(|_| Worker::new_fifo())
            .collect::<Vec<_>>();
        
        let stealers = workers.iter()
            .map(|w| w.stealer())
            .collect::<Vec<_>>();
        
        let injector = Injector::new();
        
        Ok(Self {
            control,
            config,
            workers,
            stealers,
            injector,
            worker_threads: Mutex::new(Vec::new()),
        })
    }
    
    /// Start the scheduler
    pub fn start(&self) -> Result<()> {
        let (cmd_sender, cmd_receiver) = bounded::<SchedulerCommand>(1000);
        
        let mut worker_threads = self.worker_threads.lock();
        
        for (i, worker) in self.workers.iter().enumerate() {
            let worker_clone = worker.clone();
            let stealers_clone = self.stealers.clone();
            let injector_clone = self.injector.clone();
            let control_clone = Arc::clone(&self.control);
            let id = i;
            
            let thread = thread::Builder::new()
                .name(format!("uart-worker-{}", i))
                .spawn(move || {
                    Self::worker_loop(
                        id,
                        worker_clone,
                        stealers_clone,
                        injector_clone,
                        control_clone,
                    );
                })
                .map_err(|e| RuntimeError::SchedulerError(format!("Failed to spawn worker thread: {}", e)))?;
            
            worker_threads.push(thread);
        }
        
        // Start command processing thread
        let control_clone = Arc::clone(&self.control);
        let injector_clone = self.injector.clone();
        let cmd_thread = thread::Builder::new()
            .name("uart-scheduler-cmd".to_string())
            .spawn(move || {
                Self::command_loop(cmd_receiver, control_clone, injector_clone);
            })
            .map_err(|e| RuntimeError::SchedulerError(format!("Failed to spawn command thread: {}", e)))?;
        
        worker_threads.push(cmd_thread);
        
        Ok(())
    }
    
    /// Command processing loop
    fn command_loop(
        cmd_receiver: Receiver<SchedulerCommand>,
        control: Arc<SchedulerControl>,
        injector: Injector<Task>,
    ) {
        while !control.shutdown.load(Ordering::SeqCst) {
            match cmd_receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(cmd) => match cmd {
                    SchedulerCommand::SubmitTask(task) => {
                        // Add task to metadata
                        let metadata = task.metadata.clone();
                        let id = metadata.id;
                        
                        control.tasks.write().insert(id, Arc::new(RwLock::new(metadata)));
                        
                        // Submit to the global injector
                        injector.push(task);
                    },
                    SchedulerCommand::UpdatePriority(task_id, priority) => {
                        if let Some(metadata) = control.tasks.read().get(&task_id) {
                            metadata.write().priority = priority;
                        }
                    },
                    SchedulerCommand::CancelTask(task_id) => {
                        if let Some(metadata) = control.tasks.read().get(&task_id) {
                            metadata.write().state = TaskState::Failed;
                        }
                    },
                    SchedulerCommand::Shutdown => {
                        control.shutdown.store(true, Ordering::SeqCst);
                        break;
                    },
                },
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Just continue and check shutdown again
                },
                Err(_) => {
                    // Channel closed
                    break;
                },
            }
        }
    }
    
    /// Worker thread loop
    fn worker_loop(
        id: usize,
        local: Worker<Task>,
        stealers: Vec<Stealer<Task>>,
        injector: Injector<Task>,
        control: Arc<SchedulerControl>,
    ) {
        // Set up thread-local task queue
        let mut backoff = crossbeam_utils::Backoff::new();
        
        while !control.shutdown.load(Ordering::SeqCst) {
            // Try to get a task from the local queue
            if let Some(task) = local.pop() {
                backoff.reset();
                let _ = task.execute();
                continue;
            }
            
            // Try to steal from the global injector
            if let Some(task) = injector.steal_batch_and_pop(&local) {
                backoff.reset();
                let _ = task.execute();
                continue;
            }
            
            // Try work stealing
            let mut found = false;
            for (i, stealer) in stealers.iter().enumerate() {
                if i == id {
                    continue; // Don't steal from ourselves
                }
                
                match stealer.steal_batch_and_pop(&local) {
                    crossbeam_deque::Steal::Success(task) => {
                        backoff.reset();
                        let _ = task.execute();
                        found = true;
                        break;
                    },
                    crossbeam_deque::Steal::Retry => {},
                    crossbeam_deque::Steal::Empty => {},
                }
            }
            
            if found {
                continue;
            }
            
            // Nothing to do, back off
            backoff.snooze();
        }
    }
    
    /// Spawn a new task
    pub fn spawn<F>(&self, task: F) -> Result<TaskHandle>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        self.spawn_with_priority(task, Priority::Normal)
    }
    
    /// Spawn a task with specific priority
    pub fn spawn_with_priority<F>(&self, task: F, priority: Priority) -> Result<TaskHandle>
    where
        F: FnOnce() -> Result<()> + Send + 'static,
    {
        // Generate task ID
        let id = TaskId(self.control.next_task_id.fetch_add(1, Ordering::SeqCst));
        
        // Create task
        let task_obj = Task::new(id, task, priority);
        let completion = Arc::clone(&task_obj.completion);
        let cancel = Arc::clone(&task_obj.cancel);
        
        // Submit to scheduler
        self.control.cmd_sender.send(SchedulerCommand::SubmitTask(task_obj))
            .map_err(|_| RuntimeError::SchedulerError("Failed to submit task".to_string()))?;
        
        // Create handle
        Ok(TaskHandle {
            id,
            completion,
            cancel,
            scheduler_control: Arc::clone(&self.control),
        })
    }
    
    /// Run until all tasks are complete
    pub fn run_until_idle(&self) -> Result<()> {
        while !self.is_idle() {
            thread::sleep(Duration::from_millis(10));
            
            if self.control.shutdown.load(Ordering::SeqCst) {
                return Err(RuntimeError::SchedulerError("Scheduler shutting down".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// Check if scheduler is idle (no tasks)
    pub fn is_idle(&self) -> bool {
        let tasks = self.control.tasks.read();
        
        tasks.values().all(|metadata| {
            let guard = metadata.read();
            matches!(guard.state, TaskState::Completed | TaskState::Failed)
        })
    }
    
    /// Shutdown the scheduler
    pub fn shutdown(&self) -> Result<()> {
        // Set shutdown flag
        self.control.shutdown.store(true, Ordering::SeqCst);
        
        // Send shutdown command
        let _ = self.control.cmd_sender.send(SchedulerCommand::Shutdown);
        
        // Wait for worker threads to finish
        let mut worker_threads = self.worker_threads.lock();
        while let Some(thread) = worker_threads.pop() {
            let _ = thread.join();
        }
        
        Ok(())
    }
}

impl SchedulerControl {
    /// Update task priority
    fn update_priority(&self, task_id: TaskId, priority: Priority) -> Result<()> {
        // Check if task exists
        if !self.tasks.read().contains_key(&task_id) {
            return Err(RuntimeError::SchedulerError(format!("Task {} not found", task_id)));
        }
        
        // Send command to update priority
        self.cmd_sender
            .send(SchedulerCommand::UpdatePriority(task_id, priority))
            .map_err(|_| RuntimeError::SchedulerError("Failed to update task priority".to_string()))
    }
}

/// Helper functions for tasks
pub mod helpers {
    use super::*;
    use std::cell::RefCell;
    
    thread_local! {
        static CURRENT_TASK: RefCell<Option<TaskId>> = RefCell::new(None);
    }
    
    /// Set current task ID
    pub fn set_current_task(id: TaskId) {
        CURRENT_TASK.with(|current| {
            *current.borrow_mut() = Some(id);
        });
    }
    
    /// Get current task ID
    pub fn current_task() -> Option<TaskId> {
        CURRENT_TASK.with(|current| {
            *current.borrow()
        })
    }
    
    /// Yield execution to the scheduler
    pub fn yield_now() {
        // Simply yields the CPU to other tasks
        std::thread::yield_now();
    }
}

extern crate num_cpus;
use std::collections::HashMap;