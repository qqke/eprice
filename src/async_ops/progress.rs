use chrono::{DateTime, Utc};
use std::sync::{Arc, Mutex};

/// Progress tracking for async operations with callbacks and notifications
#[derive(Clone)]
pub struct ProgressTracker {
    current_progress: Arc<Mutex<f32>>, // 0.0 to 1.0
    total_steps: Arc<Mutex<Option<u32>>>,
    current_step: Arc<Mutex<u32>>,
    status_message: Arc<Mutex<String>>,
    error_message: Arc<Mutex<Option<String>>>,
    started_at: Arc<Mutex<Option<DateTime<Utc>>>>,
    last_update: Arc<Mutex<Option<DateTime<Utc>>>>,
    callbacks: Arc<Mutex<Vec<ProgressCallback>>>,
    sub_trackers: Arc<Mutex<Vec<ProgressTracker>>>,
}

/// Progress update information
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    pub progress: f32,
    pub step: u32,
    pub total_steps: Option<u32>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Callback function type for progress updates
pub type ProgressCallback = Arc<dyn Fn(ProgressUpdate) + Send + Sync>;

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new() -> Self {
        Self {
            current_progress: Arc::new(Mutex::new(0.0)),
            total_steps: Arc::new(Mutex::new(None)),
            current_step: Arc::new(Mutex::new(0)),
            status_message: Arc::new(Mutex::new("Initializing...".to_string())),
            error_message: Arc::new(Mutex::new(None)),
            started_at: Arc::new(Mutex::new(None)),
            last_update: Arc::new(Mutex::new(None)),
            callbacks: Arc::new(Mutex::new(Vec::new())),
            sub_trackers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a progress tracker with known total steps
    pub fn with_steps(total_steps: u32) -> Self {
        let tracker = Self::new();
        *tracker.total_steps.lock().unwrap() = Some(total_steps);
        tracker
    }

    /// Start tracking progress
    pub fn start(&self, message: &str) {
        *self.started_at.lock().unwrap() = Some(Utc::now());
        *self.status_message.lock().unwrap() = message.to_string();
        self.notify_progress();
    }

    /// Update progress with a value between 0.0 and 1.0
    pub fn update_progress(&self, progress: f32, message: &str) {
        let clamped_progress = progress.clamp(0.0, 1.0);
        *self.current_progress.lock().unwrap() = clamped_progress;
        *self.status_message.lock().unwrap() = message.to_string();
        *self.last_update.lock().unwrap() = Some(Utc::now());

        // Update current step if we have total steps
        if let Some(total) = *self.total_steps.lock().unwrap() {
            *self.current_step.lock().unwrap() = (clamped_progress * total as f32) as u32;
        }

        self.notify_progress();
    }

    /// Update progress by step
    pub fn update_step(&self, step: u32, message: &str) {
        *self.current_step.lock().unwrap() = step;
        *self.status_message.lock().unwrap() = message.to_string();
        *self.last_update.lock().unwrap() = Some(Utc::now());

        // Calculate progress if we have total steps
        if let Some(total) = *self.total_steps.lock().unwrap() {
            let progress = if total > 0 {
                (step as f32 / total as f32).clamp(0.0, 1.0)
            } else {
                0.0
            };
            *self.current_progress.lock().unwrap() = progress;
        }

        self.notify_progress();
    }

    /// Increment progress by one step
    pub fn increment_step(&self, message: &str) {
        let current = *self.current_step.lock().unwrap();
        self.update_step(current + 1, message);
    }

    /// Set an error message
    pub fn set_error(&self, error: &str) {
        *self.error_message.lock().unwrap() = Some(error.to_string());
        self.notify_progress();
    }

    /// Clear any error message
    pub fn clear_error(&self) {
        *self.error_message.lock().unwrap() = None;
        self.notify_progress();
    }

    /// Mark as completed
    pub fn complete(&self) {
        *self.current_progress.lock().unwrap() = 1.0;
        *self.status_message.lock().unwrap() = "Completed".to_string();
        *self.last_update.lock().unwrap() = Some(Utc::now());

        if let Some(total) = *self.total_steps.lock().unwrap() {
            *self.current_step.lock().unwrap() = total;
        }

        self.notify_progress();
    }

    /// Reset progress to initial state
    pub fn reset(&self) {
        *self.current_progress.lock().unwrap() = 0.0;
        *self.current_step.lock().unwrap() = 0;
        *self.status_message.lock().unwrap() = "Ready".to_string();
        *self.error_message.lock().unwrap() = None;
        *self.started_at.lock().unwrap() = None;
        *self.last_update.lock().unwrap() = None;
        self.notify_progress();
    }

    /// Get current progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        *self.current_progress.lock().unwrap()
    }

    /// Get current step
    pub fn current_step(&self) -> u32 {
        *self.current_step.lock().unwrap()
    }

    /// Get total steps
    pub fn total_steps(&self) -> Option<u32> {
        *self.total_steps.lock().unwrap()
    }

    /// Get current status message
    pub fn status_message(&self) -> String {
        self.status_message.lock().unwrap().clone()
    }

    /// Get error message if any
    pub fn error_message(&self) -> Option<String> {
        self.error_message.lock().unwrap().clone()
    }

    /// Check if there's an error
    pub fn has_error(&self) -> bool {
        self.error_message.lock().unwrap().is_some()
    }

    /// Check if completed
    pub fn is_completed(&self) -> bool {
        *self.current_progress.lock().unwrap() >= 1.0
    }

    /// Check if started
    pub fn is_started(&self) -> bool {
        self.started_at.lock().unwrap().is_some()
    }

    /// Get elapsed time since start
    pub fn elapsed_seconds(&self) -> Option<i64> {
        self.started_at
            .lock()
            .unwrap()
            .map(|start| Utc::now().signed_duration_since(start).num_seconds())
    }

    /// Estimate remaining time based on current progress
    pub fn estimated_remaining_seconds(&self) -> Option<i64> {
        if let Some(elapsed) = self.elapsed_seconds() {
            let progress = self.progress();
            if progress > 0.0 && progress < 1.0 {
                let total_estimated = elapsed as f32 / progress;
                let remaining = total_estimated - elapsed as f32;
                Some(remaining.max(0.0) as i64)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Add a progress callback
    pub fn add_callback(&self, callback: ProgressCallback) {
        self.callbacks.lock().unwrap().push(callback);
    }

    /// Remove all callbacks
    pub fn clear_callbacks(&self) {
        self.callbacks.lock().unwrap().clear();
    }

    /// Add a sub-tracker for hierarchical progress
    pub fn add_sub_tracker(&self, tracker: ProgressTracker) {
        self.sub_trackers.lock().unwrap().push(tracker);
    }

    /// Get combined progress including sub-trackers
    pub fn combined_progress(&self) -> f32 {
        let main_progress = self.progress();
        let sub_trackers = self.sub_trackers.lock().unwrap();

        if sub_trackers.is_empty() {
            return main_progress;
        }

        let sub_progress: f32 = sub_trackers
            .iter()
            .map(|tracker| tracker.progress())
            .sum::<f32>()
            / sub_trackers.len() as f32;

        (main_progress + sub_progress) / 2.0
    }

    /// Get detailed status including sub-trackers
    pub fn detailed_status(&self) -> DetailedStatus {
        let sub_trackers = self.sub_trackers.lock().unwrap();
        let sub_statuses: Vec<DetailedStatus> = sub_trackers
            .iter()
            .map(|tracker| tracker.detailed_status())
            .collect();

        DetailedStatus {
            progress: self.progress(),
            step: self.current_step(),
            total_steps: self.total_steps(),
            message: self.status_message(),
            error: self.error_message(),
            started_at: *self.started_at.lock().unwrap(),
            last_update: *self.last_update.lock().unwrap(),
            elapsed_seconds: self.elapsed_seconds(),
            estimated_remaining: self.estimated_remaining_seconds(),
            sub_operations: sub_statuses,
        }
    }

    /// Create progress update structure
    fn create_update(&self) -> ProgressUpdate {
        ProgressUpdate {
            progress: self.progress(),
            step: self.current_step(),
            total_steps: self.total_steps(),
            message: self.status_message(),
            timestamp: Utc::now(),
            estimated_completion: self
                .estimated_remaining_seconds()
                .map(|remaining| Utc::now() + chrono::Duration::seconds(remaining)),
        }
    }

    /// Notify all callbacks of progress update
    fn notify_progress(&self) {
        let update = self.create_update();
        let callbacks = self.callbacks.lock().unwrap();

        for callback in callbacks.iter() {
            callback(update.clone());
        }
    }
}

/// Detailed status information including sub-operations
#[derive(Debug, Clone)]
pub struct DetailedStatus {
    pub progress: f32,
    pub step: u32,
    pub total_steps: Option<u32>,
    pub message: String,
    pub error: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_update: Option<DateTime<Utc>>,
    pub elapsed_seconds: Option<i64>,
    pub estimated_remaining: Option<i64>,
    pub sub_operations: Vec<DetailedStatus>,
}

impl Default for ProgressTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Progress tracker builder for easy configuration
pub struct ProgressTrackerBuilder {
    total_steps: Option<u32>,
    callbacks: Vec<ProgressCallback>,
}

impl ProgressTrackerBuilder {
    pub fn new() -> Self {
        Self {
            total_steps: None,
            callbacks: Vec::new(),
        }
    }

    pub fn with_steps(mut self, steps: u32) -> Self {
        self.total_steps = Some(steps);
        self
    }

    pub fn with_callback(mut self, callback: ProgressCallback) -> Self {
        self.callbacks.push(callback);
        self
    }

    pub fn build(self) -> ProgressTracker {
        let tracker = if let Some(steps) = self.total_steps {
            ProgressTracker::with_steps(steps)
        } else {
            ProgressTracker::new()
        };

        for callback in self.callbacks {
            tracker.add_callback(callback);
        }

        tracker
    }
}

impl Default for ProgressTrackerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for creating common progress patterns
pub struct ProgressUtils;

impl ProgressUtils {
    /// Create a progress tracker with console logging
    pub fn with_console_logging(steps: Option<u32>) -> ProgressTracker {
        let tracker = if let Some(s) = steps {
            ProgressTracker::with_steps(s)
        } else {
            ProgressTracker::new()
        };

        let callback: ProgressCallback = Arc::new(move |update| {
            if let Some(_error) = &update.estimated_completion {
                println!("[ERROR] {}", update.message);
            } else {
                println!(
                    "[{}%] {} (Step {}/{})",
                    (update.progress * 100.0) as u32,
                    update.message,
                    update.step,
                    update.total_steps.unwrap_or(0)
                );
            }
        });

        tracker.add_callback(callback);
        tracker
    }

    /// Create a progress tracker that updates a shared status
    pub fn with_shared_status(status: Arc<Mutex<String>>) -> ProgressTracker {
        let tracker = ProgressTracker::new();

        let callback: ProgressCallback = Arc::new(move |update| {
            if let Ok(mut status_guard) = status.lock() {
                *status_guard =
                    format!("{} ({}%)", update.message, (update.progress * 100.0) as u32);
            }
        });

        tracker.add_callback(callback);
        tracker
    }

    /// Create a hierarchical progress tracker for batch operations
    pub fn create_batch_tracker(operation_count: u32) -> (ProgressTracker, Vec<ProgressTracker>) {
        let main_tracker = ProgressTracker::with_steps(operation_count);
        let mut sub_trackers = Vec::new();

        for i in 0..operation_count {
            let sub_tracker = ProgressTracker::new();
            let main_tracker_clone = main_tracker.clone();
            let step = i + 1;

            let callback: ProgressCallback = Arc::new(move |update| {
                if update.progress >= 1.0 {
                    main_tracker_clone.update_step(step, &format!("Completed operation {}", step));
                }
            });

            sub_tracker.add_callback(callback);
            main_tracker.add_sub_tracker(sub_tracker.clone());
            sub_trackers.push(sub_tracker);
        }

        (main_tracker, sub_trackers)
    }
}
