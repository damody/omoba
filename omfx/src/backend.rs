//! Backend process management
//!
//! Handles automatic starting and stopping of the game backend (omb).

use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use log::{info, warn, error};

use crate::config::BackendConfig;

/// Backend process manager
pub struct BackendManager {
    /// Backend configuration
    config: BackendConfig,
    /// Child process handle
    process: Option<Child>,
    /// Time when backend was started
    start_time: Option<Instant>,
}

impl BackendManager {
    /// Create a new backend manager
    pub fn new(config: BackendConfig) -> Self {
        Self {
            config,
            process: None,
            start_time: None,
        }
    }

    /// Start the backend process
    pub fn start(&mut self) -> Result<(), String> {
        if self.is_running() {
            warn!("Backend is already running");
            return Ok(());
        }

        // Resolve executable path
        let exe_path = self.resolve_executable_path()?;

        info!("Starting backend: {:?}", exe_path);

        // Build command
        let mut cmd = Command::new(&exe_path);

        // Add arguments
        for arg in &self.config.args {
            cmd.arg(arg);
        }

        // Set working directory
        if let Some(ref work_dir) = self.config.working_directory {
            let work_path = PathBuf::from(work_dir);
            if work_path.exists() {
                cmd.current_dir(work_path);
            } else {
                warn!("Working directory does not exist: {}", work_dir);
            }
        }

        // Set environment variables
        for (key, value) in &self.config.env {
            cmd.env(key, value);
        }

        // Configure stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Spawn process
        match cmd.spawn() {
            Ok(child) => {
                info!("Backend started with PID: {}", child.id());
                self.process = Some(child);
                self.start_time = Some(Instant::now());
                Ok(())
            }
            Err(e) => {
                error!("Failed to start backend: {}", e);
                Err(format!("Failed to start backend: {}", e))
            }
        }
    }

    /// Stop the backend process
    pub fn stop(&mut self) -> Result<(), String> {
        if let Some(mut child) = self.process.take() {
            info!("Stopping backend (PID: {})", child.id());

            // Try graceful shutdown first
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                // Send SIGTERM
                unsafe {
                    libc::kill(child.id() as i32, libc::SIGTERM);
                }
            }

            #[cfg(windows)]
            {
                // On Windows, just kill the process
                let _ = child.kill();
            }

            // Wait for process to exit with timeout
            let timeout = Duration::from_millis(self.config.shutdown_timeout_ms);
            let start = Instant::now();

            loop {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        info!("Backend exited with status: {}", status);
                        break;
                    }
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            warn!("Backend did not exit gracefully, forcing kill");
                            let _ = child.kill();
                            let _ = child.wait();
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                    Err(e) => {
                        error!("Error waiting for backend: {}", e);
                        break;
                    }
                }
            }

            self.start_time = None;
            Ok(())
        } else {
            warn!("Backend is not running");
            Ok(())
        }
    }

    /// Check if backend is running
    pub fn is_running(&mut self) -> bool {
        if let Some(ref mut child) = self.process {
            match child.try_wait() {
                Ok(Some(_)) => {
                    // Process has exited
                    self.process = None;
                    self.start_time = None;
                    false
                }
                Ok(None) => true, // Still running
                Err(_) => {
                    self.process = None;
                    self.start_time = None;
                    false
                }
            }
        } else {
            false
        }
    }

    /// Get backend uptime
    pub fn uptime(&self) -> Option<Duration> {
        self.start_time.map(|t| t.elapsed())
    }

    /// Get backend process ID
    pub fn pid(&self) -> Option<u32> {
        self.process.as_ref().map(|c| c.id())
    }

    /// Resolve executable path (handle relative paths and Windows .exe extension)
    fn resolve_executable_path(&self) -> Result<PathBuf, String> {
        let path = PathBuf::from(&self.config.executable_path);

        // On Windows, try adding .exe if not already present
        let candidates = Self::get_path_candidates(&path);

        // If path is absolute, use it directly
        if path.is_absolute() {
            for candidate in &candidates {
                if candidate.exists() {
                    return Ok(candidate.clone());
                }
            }
            return Err(format!("Backend executable not found: {:?}", path));
        }

        // Try relative to current directory
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Cannot get current directory: {}", e))?;

        for candidate in &candidates {
            let abs_path = current_dir.join(candidate);
            if abs_path.exists() {
                return Ok(abs_path);
            }
        }

        // Try relative to executable directory
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                for candidate in &candidates {
                    let from_exe = exe_dir.join(candidate);
                    if from_exe.exists() {
                        return Ok(from_exe);
                    }
                }
            }
        }

        let abs_path = current_dir.join(&path);
        Err(format!("Backend executable not found: {} (tried {:?})",
                    self.config.executable_path, abs_path))
    }

    /// Get path candidates (original path + with .exe on Windows)
    fn get_path_candidates(path: &PathBuf) -> Vec<PathBuf> {
        let mut candidates = vec![path.clone()];

        // On Windows, also try with .exe extension if not already present
        #[cfg(windows)]
        {
            if path.extension().map_or(true, |ext| ext != "exe") {
                let mut with_exe = path.clone();
                let new_name = format!(
                    "{}.exe",
                    path.file_name().unwrap_or_default().to_string_lossy()
                );
                with_exe.set_file_name(new_name);
                candidates.push(with_exe);
            }
        }

        candidates
    }

    /// Restart the backend
    pub fn restart(&mut self) -> Result<(), String> {
        self.stop()?;
        std::thread::sleep(Duration::from_millis(500));
        self.start()
    }
}

impl Drop for BackendManager {
    fn drop(&mut self) {
        if self.is_running() {
            info!("Shutting down backend on drop");
            let _ = self.stop();
        }
    }
}
