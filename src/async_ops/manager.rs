use crate::async_ops::progress::ProgressUpdate;
use crate::async_ops::{AsyncOperation, OperationResult, OperationType};
use chrono::Utc;
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Central manager for async operations with scheduling, monitoring, and lifecycle management
pub struct AsyncManager {
    operations: Arc<Mutex<HashMap<String, AsyncOperation>>>,
    operation_results: Arc<Mutex<HashMap<String, OperationResult>>>,
    operation_queue: Arc<Mutex<VecDeque<String>>>,
    running_operations: Arc<Mutex<HashSet<String>>>,
    completed_operations: Arc<Mutex<Vec<String>>>,
    failed_operations: Arc<Mutex<Vec<String>>>,

    // Configuration
    max_concurrent_operations: usize,
    cleanup_interval_hours: u64,
    auto_retry_enabled: bool,
    operation_timeout_seconds: u64,

    // Statistics
    stats: Arc<Mutex<OperationStats>>,

    // Callbacks and monitoring
    status_callbacks: Arc<Mutex<Vec<StatusCallback>>>,
    last_cleanup: Arc<Mutex<Instant>>,
}

/// Handle for tracking and controlling an async operation
#[derive(Clone)]
pub struct OperationHandle {
    pub id: String,
    manager: Arc<AsyncManager>,
}

/// Current status of an operation
#[derive(Debug, Clone, PartialEq)]
pub enum OperationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Statistics about operation execution
#[derive(Debug, Clone)]
pub struct OperationStats {
    pub total_operations: u64,
    pub completed_operations: u64,
    pub failed_operations: u64,
    pub cancelled_operations: u64,
    pub average_execution_time_ms: f64,
    pub operations_per_hour: f64,
    pub success_rate: f64,
    pub current_queue_size: usize,
    pub peak_queue_size: usize,
    pub operation_type_counts: HashMap<OperationType, u64>,
}

/// Callback for operation status changes
pub type StatusCallback =
    Arc<dyn Fn(String, OperationStatus, Option<ProgressUpdate>) + Send + Sync>;

impl AsyncManager {
    /// Create a new async operation manager
    pub fn new() -> Self {
        Self {
            operations: Arc::new(Mutex::new(HashMap::new())),
            operation_results: Arc::new(Mutex::new(HashMap::new())),
            operation_queue: Arc::new(Mutex::new(VecDeque::new())),
            running_operations: Arc::new(Mutex::new(HashSet::new())),
            completed_operations: Arc::new(Mutex::new(Vec::new())),
            failed_operations: Arc::new(Mutex::new(Vec::new())),
            max_concurrent_operations: 5,
            cleanup_interval_hours: 24,
            auto_retry_enabled: true,
            operation_timeout_seconds: 300,
            stats: Arc::new(Mutex::new(OperationStats::new())),
            status_callbacks: Arc::new(Mutex::new(Vec::new())),
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Configure the manager
    pub fn configure(
        mut self,
        max_concurrent: usize,
        cleanup_interval_hours: u64,
        auto_retry: bool,
        timeout_seconds: u64,
    ) -> Self {
        self.max_concurrent_operations = max_concurrent;
        self.cleanup_interval_hours = cleanup_interval_hours;
        self.auto_retry_enabled = auto_retry;
        self.operation_timeout_seconds = timeout_seconds;
        self
    }

    /// Submit an operation for execution
    pub fn submit_operation(&self, operation: AsyncOperation) -> OperationHandle {
        let handle = OperationHandle {
            id: operation.id.clone(),
            manager: Arc::new(self.clone()),
        };

        // Add to operations map
        self.operations
            .lock()
            .unwrap()
            .insert(operation.id.clone(), operation.clone());

        // Add to queue if dependencies are satisfied
        if self.are_dependencies_satisfied(&operation.id) {
            self.operation_queue
                .lock()
                .unwrap()
                .push_back(operation.id.clone());
        }

        // Update statistics
        self.update_stats_for_submission(&operation.operation_type);

        // Notify status change
        self.notify_status_change(&operation.id, OperationStatus::Pending, None);

        // Try to start operations
        self.process_queue();

        handle
    }

    /// Submit multiple operations as a batch
    pub fn submit_batch(&self, operations: Vec<AsyncOperation>) -> Vec<OperationHandle> {
        operations
            .into_iter()
            .map(|op| self.submit_operation(op))
            .collect()
    }

    /// Cancel an operation
    pub fn cancel_operation(&self, operation_id: &str) -> Result<(), String> {
        let mut operations = self.operations.lock().unwrap();

        if let Some(operation) = operations.get_mut(operation_id) {
            if !operation.cancellable {
                return Err("Operation is not cancellable".to_string());
            }

            // Remove from queue if pending
            let mut queue = self.operation_queue.lock().unwrap();
            if let Some(pos) = queue.iter().position(|id| id == operation_id) {
                queue.remove(pos);
                drop(queue);

                // Mark as cancelled
                self.operation_results
                    .lock()
                    .unwrap()
                    .insert(operation_id.to_string(), OperationResult::Cancelled);

                self.notify_status_change(operation_id, OperationStatus::Cancelled, None);
                return Ok(());
            }

            // If running, mark for cancellation (actual cancellation depends on implementation)
            let running = self.running_operations.lock().unwrap();
            if running.contains(operation_id) {
                // In a real implementation, this would signal the running task to stop
                drop(running);

                self.operation_results
                    .lock()
                    .unwrap()
                    .insert(operation_id.to_string(), OperationResult::Cancelled);

                self.notify_status_change(operation_id, OperationStatus::Cancelled, None);
                return Ok(());
            }

            Err("Operation not found in queue or running operations".to_string())
        } else {
            Err("Operation not found".to_string())
        }
    }

    /// Get the status of an operation
    pub fn get_operation_status(&self, operation_id: &str) -> Option<OperationStatus> {
        // Check if completed
        if self
            .completed_operations
            .lock()
            .unwrap()
            .contains(&operation_id.to_string())
        {
            return Some(OperationStatus::Completed);
        }

        // Check if failed
        if self
            .failed_operations
            .lock()
            .unwrap()
            .contains(&operation_id.to_string())
        {
            return Some(OperationStatus::Failed);
        }

        // Check if running
        if self
            .running_operations
            .lock()
            .unwrap()
            .contains(operation_id)
        {
            return Some(OperationStatus::Running);
        }

        // Check if in queue
        let queue = self.operation_queue.lock().unwrap();
        if queue.iter().any(|id| id == operation_id) {
            return Some(OperationStatus::Pending);
        }

        // Check if cancelled
        if let Some(result) = self.operation_results.lock().unwrap().get(operation_id) {
            match result {
                OperationResult::Cancelled => Some(OperationStatus::Cancelled),
                OperationResult::Success(_) => Some(OperationStatus::Completed),
                OperationResult::Failure(_) => Some(OperationStatus::Failed),
            }
        } else {
            None
        }
    }

    /// Get operation result if completed
    pub fn get_operation_result(&self, operation_id: &str) -> Option<OperationResult> {
        self.operation_results
            .lock()
            .unwrap()
            .get(operation_id)
            .cloned()
    }

    /// Get current operation statistics
    pub fn get_statistics(&self) -> OperationStats {
        let mut stats = self.stats.lock().unwrap().clone();

        // Update current queue size
        stats.current_queue_size = self.operation_queue.lock().unwrap().len();
        stats.peak_queue_size = stats.peak_queue_size.max(stats.current_queue_size);

        // Calculate success rate
        if stats.total_operations > 0 {
            stats.success_rate =
                (stats.completed_operations as f64 / stats.total_operations as f64) * 100.0;
        }

        stats
    }

    /// Get all operations with their status
    pub fn get_all_operations(&self) -> Vec<(AsyncOperation, OperationStatus)> {
        let operations = self.operations.lock().unwrap();
        operations
            .iter()
            .map(|(id, op)| {
                let status = self
                    .get_operation_status(id)
                    .unwrap_or(OperationStatus::Pending);
                (op.clone(), status)
            })
            .collect()
    }

    /// Get operations by type
    pub fn get_operations_by_type(
        &self,
        operation_type: &OperationType,
    ) -> Vec<(AsyncOperation, OperationStatus)> {
        self.get_all_operations()
            .into_iter()
            .filter(|(op, _)| &op.operation_type == operation_type)
            .collect()
    }

    /// Get running operations
    pub fn get_running_operations(&self) -> Vec<AsyncOperation> {
        let operations = self.operations.lock().unwrap();
        let running = self.running_operations.lock().unwrap();

        running
            .iter()
            .filter_map(|id| operations.get(id).cloned())
            .collect()
    }

    /// Add a status change callback
    pub fn add_status_callback(&self, callback: StatusCallback) {
        self.status_callbacks.lock().unwrap().push(callback);
    }

    /// Clear all status callbacks
    pub fn clear_status_callbacks(&self) {
        self.status_callbacks.lock().unwrap().clear();
    }

    /// Pause operation processing
    pub fn pause(&self) {
        // Implementation would set a flag to pause queue processing
    }

    /// Resume operation processing
    pub fn resume(&self) {
        // Implementation would clear pause flag and restart queue processing
        self.process_queue();
    }

    /// Clean up old completed operations
    pub fn cleanup_old_operations(&self) {
        let now = Instant::now();
        let mut last_cleanup = self.last_cleanup.lock().unwrap();

        if now.duration_since(*last_cleanup)
            < Duration::from_secs(self.cleanup_interval_hours * 3600)
        {
            return;
        }

        let cutoff_time = Utc::now() - chrono::Duration::hours(self.cleanup_interval_hours as i64);
        let mut operations = self.operations.lock().unwrap();
        let mut results = self.operation_results.lock().unwrap();
        let mut completed = self.completed_operations.lock().unwrap();
        let mut failed = self.failed_operations.lock().unwrap();

        // Remove old completed operations
        operations.retain(|_, op| {
            op.completed_at
                .map_or(true, |completed_at| completed_at > cutoff_time)
        });

        // Clean up results
        let operation_ids: HashSet<String> = operations.keys().cloned().collect();
        results.retain(|id, _| operation_ids.contains(id));
        completed.retain(|id| operation_ids.contains(id));
        failed.retain(|id| operation_ids.contains(id));

        *last_cleanup = now;
    }

    // Private helper methods

    fn process_queue(&self) {
        let max_concurrent = self.max_concurrent_operations;
        let mut running = self.running_operations.lock().unwrap();

        if running.len() >= max_concurrent {
            return;
        }

        let mut queue = self.operation_queue.lock().unwrap();

        while running.len() < max_concurrent && !queue.is_empty() {
            if let Some(operation_id) = queue.pop_front() {
                let operations = self.operations.lock().unwrap();
                if let Some(_operation) = operations.get(&operation_id) {
                    if self.are_dependencies_satisfied(&operation_id) {
                        running.insert(operation_id.clone());
                        drop(running);
                        drop(queue);
                        drop(operations);

                        // Start the operation (in a real implementation, this would spawn a task)
                        self.start_operation(&operation_id);

                        // Re-acquire locks for next iteration
                        running = self.running_operations.lock().unwrap();
                        queue = self.operation_queue.lock().unwrap();
                    } else {
                        // Dependencies not satisfied, put back in queue
                        queue.push_back(operation_id);
                        break;
                    }
                }
            }
        }
    }

    fn start_operation(&self, operation_id: &str) {
        self.notify_status_change(operation_id, OperationStatus::Running, None);

        // In a real implementation, this would spawn an async task
        // For now, we'll simulate completion after a short delay
        let manager = self.clone();
        let id = operation_id.to_string();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100)); // Simulate work
            manager.complete_operation(
                &id,
                OperationResult::Success(crate::async_ops::operations::OperationData::None),
            );
        });
    }

    fn complete_operation(&self, operation_id: &str, result: OperationResult) {
        // Remove from running
        self.running_operations.lock().unwrap().remove(operation_id);

        // Store result
        self.operation_results
            .lock()
            .unwrap()
            .insert(operation_id.to_string(), result.clone());

        // Update lists
        match result {
            OperationResult::Success(_) => {
                self.completed_operations
                    .lock()
                    .unwrap()
                    .push(operation_id.to_string());
                self.notify_status_change(operation_id, OperationStatus::Completed, None);
            }
            OperationResult::Failure(_) => {
                self.failed_operations
                    .lock()
                    .unwrap()
                    .push(operation_id.to_string());

                // Handle retry if enabled
                if self.auto_retry_enabled {
                    if let Some(operation) = self.operations.lock().unwrap().get_mut(operation_id) {
                        if operation.can_retry() {
                            operation.increment_retry();
                            self.operation_queue
                                .lock()
                                .unwrap()
                                .push_back(operation_id.to_string());
                            self.notify_status_change(operation_id, OperationStatus::Pending, None);
                            self.process_queue();
                            return;
                        }
                    }
                }

                self.notify_status_change(operation_id, OperationStatus::Failed, None);
            }
            OperationResult::Cancelled => {
                self.notify_status_change(operation_id, OperationStatus::Cancelled, None);
            }
        }

        // Update statistics
        self.update_stats_for_completion(operation_id);

        // Process queue for next operations
        self.process_queue();

        // Check for dependent operations
        self.check_dependent_operations(operation_id);
    }

    fn are_dependencies_satisfied(&self, operation_id: &str) -> bool {
        let operations = self.operations.lock().unwrap();
        let completed = self.completed_operations.lock().unwrap();

        if let Some(operation) = operations.get(operation_id) {
            operation.dependencies_satisfied(&completed)
        } else {
            false
        }
    }

    fn check_dependent_operations(&self, completed_operation_id: &str) {
        let operations = self.operations.lock().unwrap();
        let mut queue = self.operation_queue.lock().unwrap();

        // Find operations that depend on the completed operation
        for (id, operation) in operations.iter() {
            if operation
                .dependencies
                .contains(&completed_operation_id.to_string())
            {
                if self.are_dependencies_satisfied(id) && !queue.contains(id) {
                    queue.push_back(id.clone());
                }
            }
        }

        drop(queue);
        drop(operations);
        self.process_queue();
    }

    fn notify_status_change(
        &self,
        operation_id: &str,
        status: OperationStatus,
        progress: Option<ProgressUpdate>,
    ) {
        let callbacks = self.status_callbacks.lock().unwrap();
        for callback in callbacks.iter() {
            callback(operation_id.to_string(), status.clone(), progress.clone());
        }
    }

    fn update_stats_for_submission(&self, operation_type: &OperationType) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_operations += 1;
        *stats
            .operation_type_counts
            .entry(operation_type.clone())
            .or_insert(0) += 1;
    }

    fn update_stats_for_completion(&self, operation_id: &str) {
        let operations = self.operations.lock().unwrap();
        if let Some(operation) = operations.get(operation_id) {
            if let Some(duration) = operation.duration_seconds() {
                let mut stats = self.stats.lock().unwrap();

                // Update average execution time
                let total_time =
                    stats.average_execution_time_ms * stats.completed_operations as f64;
                stats.completed_operations += 1;
                stats.average_execution_time_ms =
                    (total_time + duration as f64 * 1000.0) / stats.completed_operations as f64;
            }
        }
    }
}

impl Clone for AsyncManager {
    fn clone(&self) -> Self {
        Self {
            operations: Arc::clone(&self.operations),
            operation_results: Arc::clone(&self.operation_results),
            operation_queue: Arc::clone(&self.operation_queue),
            running_operations: Arc::clone(&self.running_operations),
            completed_operations: Arc::clone(&self.completed_operations),
            failed_operations: Arc::clone(&self.failed_operations),
            max_concurrent_operations: self.max_concurrent_operations,
            cleanup_interval_hours: self.cleanup_interval_hours,
            auto_retry_enabled: self.auto_retry_enabled,
            operation_timeout_seconds: self.operation_timeout_seconds,
            stats: Arc::clone(&self.stats),
            status_callbacks: Arc::clone(&self.status_callbacks),
            last_cleanup: Arc::clone(&self.last_cleanup),
        }
    }
}

impl OperationHandle {
    /// Get the current status of this operation
    pub fn status(&self) -> Option<OperationStatus> {
        self.manager.get_operation_status(&self.id)
    }

    /// Get the result if the operation is completed
    pub fn result(&self) -> Option<OperationResult> {
        self.manager.get_operation_result(&self.id)
    }

    /// Cancel this operation
    pub fn cancel(&self) -> Result<(), String> {
        self.manager.cancel_operation(&self.id)
    }

    /// Check if the operation is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status(), Some(OperationStatus::Completed))
    }

    /// Check if the operation is running
    pub fn is_running(&self) -> bool {
        matches!(self.status(), Some(OperationStatus::Running))
    }

    /// Check if the operation failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status(), Some(OperationStatus::Failed))
    }

    /// Wait for completion (blocking)
    pub fn wait_for_completion(
        &self,
        timeout_seconds: Option<u64>,
    ) -> Result<OperationResult, String> {
        let start = Instant::now();
        let timeout_duration = timeout_seconds.map(Duration::from_secs);

        loop {
            if let Some(result) = self.result() {
                return Ok(result);
            }

            if let Some(timeout) = timeout_duration {
                if start.elapsed() > timeout {
                    return Err("Operation timed out".to_string());
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

impl OperationStats {
    fn new() -> Self {
        Self {
            total_operations: 0,
            completed_operations: 0,
            failed_operations: 0,
            cancelled_operations: 0,
            average_execution_time_ms: 0.0,
            operations_per_hour: 0.0,
            success_rate: 0.0,
            current_queue_size: 0,
            peak_queue_size: 0,
            operation_type_counts: HashMap::new(),
        }
    }
}

impl Default for AsyncManager {
    fn default() -> Self {
        Self::new()
    }
}
