use crate::scanner::models::CameraConfig;
use crate::scanner::ScannerError;
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Camera manager for handling camera operations and frame capture
pub struct CameraManager {
    config: CameraConfig,
    is_running: Arc<Mutex<bool>>,
    current_frame: Arc<Mutex<Option<Vec<u8>>>>,
    last_capture_time: Arc<Mutex<Instant>>,
}

impl CameraManager {
    pub fn new() -> Self {
        Self {
            config: CameraConfig::default(),
            is_running: Arc::new(Mutex::new(false)),
            current_frame: Arc::new(Mutex::new(None)),
            last_capture_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn with_config(config: CameraConfig) -> Self {
        Self {
            config,
            is_running: Arc::new(Mutex::new(false)),
            current_frame: Arc::new(Mutex::new(None)),
            last_capture_time: Arc::new(Mutex::new(Instant::now())),
        }
    }

    /// Start the camera with the specified configuration
    pub fn start_camera(&self) -> Result<(), ScannerError> {
        let mut is_running = self
            .is_running
            .lock()
            .map_err(|e| ScannerError::CameraAccess(format!("Failed to acquire lock: {}", e)))?;

        if *is_running {
            return Err(ScannerError::CameraAccess(
                "Camera is already running".to_string(),
            ));
        }

        log::info!(
            "Starting camera with config: {}x{} at {} fps",
            self.config.width,
            self.config.height,
            self.config.fps
        );

        // In a real implementation, this would initialize the camera hardware
        // For now, we'll simulate camera operation
        *is_running = true;

        // Start the camera capture thread (mock implementation)
        self.start_capture_thread()?;

        Ok(())
    }

    /// Stop the camera
    pub fn stop_camera(&self) -> Result<(), ScannerError> {
        let mut is_running = self
            .is_running
            .lock()
            .map_err(|e| ScannerError::CameraAccess(format!("Failed to acquire lock: {}", e)))?;

        if !*is_running {
            return Ok(()); // Already stopped
        }

        log::info!("Stopping camera");
        *is_running = false;

        Ok(())
    }

    /// Check if camera is currently running
    pub fn is_running(&self) -> bool {
        match self.is_running.lock() {
            Ok(guard) => *guard,
            Err(_) => false, // If the mutex is poisoned, assume camera is not running
        }
    }

    /// Capture a single frame from the camera
    pub fn capture_frame(&self) -> Result<Vec<u8>, ScannerError> {
        if !self.is_running() {
            return Err(ScannerError::CameraAccess(
                "Camera is not running".to_string(),
            ));
        }

        // Update last capture time
        if let Ok(mut last_time) = self.last_capture_time.lock() {
            *last_time = Instant::now();
        }

        // Get current frame
        let current_frame = self.current_frame.lock().map_err(|e| {
            ScannerError::CameraAccess(format!("Failed to acquire frame lock: {}", e))
        })?;

        match current_frame.as_ref() {
            Some(frame) => Ok(frame.clone()),
            None => {
                // Generate a mock frame if no real frame is available
                self.generate_mock_frame()
            }
        }
    }

    /// Get the current camera configuration
    pub fn get_config(&self) -> &CameraConfig {
        &self.config
    }

    /// Update camera configuration (requires restart)
    pub fn update_config(&mut self, config: CameraConfig) -> Result<(), ScannerError> {
        if self.is_running() {
            return Err(ScannerError::CameraAccess(
                "Cannot update config while camera is running. Stop camera first.".to_string(),
            ));
        }

        self.config = config;
        Ok(())
    }

    /// List available cameras (mock implementation)
    pub fn list_cameras() -> Vec<CameraInfo> {
        // In a real implementation, this would enumerate actual camera devices
        vec![
            CameraInfo {
                id: 0,
                name: "Default Camera".to_string(),
                description: "Built-in camera or default video device".to_string(),
                supported_resolutions: vec![(640, 480), (1280, 720), (1920, 1080)],
            },
            CameraInfo {
                id: 1,
                name: "External Camera".to_string(),
                description: "USB or external camera device".to_string(),
                supported_resolutions: vec![(640, 480), (1280, 720)],
            },
        ]
    }

    /// Start the camera capture thread (mock implementation)
    fn start_capture_thread(&self) -> Result<(), ScannerError> {
        let is_running = Arc::clone(&self.is_running);
        let current_frame = Arc::clone(&self.current_frame);
        let fps = self.config.fps;

        thread::spawn(move || {
            let frame_duration = Duration::from_millis(1000 / fps as u64);

            loop {
                let should_continue = match is_running.lock() {
                    Ok(guard) => *guard,
                    Err(_) => false, // If poisoned, stop the thread
                };

                if !should_continue {
                    break;
                }

                // Simulate frame capture
                if let Ok(mut frame) = current_frame.lock() {
                    *frame = Some(Self::generate_mock_frame_static());
                }

                thread::sleep(frame_duration);
            }
        });

        Ok(())
    }

    /// Generate a mock frame for testing purposes
    fn generate_mock_frame(&self) -> Result<Vec<u8>, ScannerError> {
        Ok(Self::generate_mock_frame_static())
    }

    /// Generate a static mock frame
    fn generate_mock_frame_static() -> Vec<u8> {
        // Generate a simple mock frame (grayscale image)
        let width = 640;
        let height = 480;
        let mut frame = vec![128u8; width * height]; // Gray image

        // Add some pattern to make it look like a real frame
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                if (x / 50 + y / 50) % 2 == 0 {
                    frame[idx] = 200; // Lighter gray
                } else {
                    frame[idx] = 100; // Darker gray
                }
            }
        }

        frame
    }
}

impl Default for CameraManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about available cameras
#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub supported_resolutions: Vec<(u32, u32)>,
}
