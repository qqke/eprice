use crate::scanner::ScannerError;
use crate::scanner::models::CameraConfig;
use anyhow::Result;
use nokhwa::Camera;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
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

        // Try to initialize the real camera
        match self.initialize_real_camera() {
            Ok(_) => {
                log::info!("Real camera initialized successfully");
            }
            Err(e) => {
                log::warn!(
                    "Failed to initialize real camera: {}. Using mock camera.",
                    e
                );
                // Fall back to mock implementation
                self.start_mock_capture_thread()?;
            }
        }

        *is_running = true;
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

        // Try to capture from real camera first
        match self.capture_real_frame() {
            Ok(frame) => Ok(frame),
            Err(_) => {
                // Fall back to mock frame
                self.generate_mock_frame()
            }
        }
    }

    /// Capture frame from real camera
    fn capture_real_frame(&self) -> Result<Vec<u8>, ScannerError> {
        let camera_index = CameraIndex::Index(self.config.camera_index);
        let requested_format =
            RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestResolution);

        match Camera::new(camera_index, requested_format) {
            Ok(mut camera) => {
                camera.open_stream().map_err(|e| {
                    ScannerError::CameraAccess(format!("Failed to open camera stream: {}", e))
                })?;

                let frame = camera.frame().map_err(|e| {
                    ScannerError::CameraAccess(format!("Failed to capture frame: {}", e))
                })?;

                Ok(frame.buffer().to_vec())
            }
            Err(e) => Err(ScannerError::CameraAccess(format!(
                "Failed to initialize camera for capture: {}",
                e
            ))),
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

    /// List available cameras using nokhwa
    pub fn list_cameras() -> Vec<CameraInfo> {
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(camera_list) => {
                camera_list
                    .into_iter()
                    .map(|info| CameraInfo {
                        id: match info.index() {
                            CameraIndex::Index(i) => *i,
                            CameraIndex::String(_) => 0, // Fallback for string-based indices
                        },
                        name: info.human_name().to_string(),
                        description: info.description().to_string(),
                        supported_resolutions: vec![(640, 480), (1280, 720), (1920, 1080)], // Default resolutions
                    })
                    .collect()
            }
            Err(e) => {
                log::warn!("Failed to query cameras: {}. Using mock cameras.", e);
                // Fallback to mock cameras
                vec![CameraInfo {
                    id: 0,
                    name: "Mock Camera 1".to_string(),
                    description: "Simulated camera device".to_string(),
                    supported_resolutions: vec![(640, 480), (1280, 720), (1920, 1080)],
                }]
            }
        }
    }

    /// Initialize real camera using nokhwa
    fn initialize_real_camera(&self) -> Result<(), ScannerError> {
        // For now, we'll just verify that nokhwa can detect cameras
        // The actual camera will be created on-demand for each frame capture
        match nokhwa::query(nokhwa::utils::ApiBackend::Auto) {
            Ok(cameras) => {
                if cameras.is_empty() {
                    return Err(ScannerError::CameraAccess(
                        "No cameras detected".to_string(),
                    ));
                }

                log::info!("Found {} camera(s)", cameras.len());

                // Start mock capture thread as fallback
                self.start_mock_capture_thread()?;
                Ok(())
            }
            Err(e) => Err(ScannerError::CameraAccess(format!(
                "Failed to query cameras: {}",
                e
            ))),
        }
    }

    /// Start the camera capture thread (mock implementation)
    fn start_mock_capture_thread(&self) -> Result<(), ScannerError> {
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
