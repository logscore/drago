use std::fs;
use std::io;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

const PID_FILE: &str = "/tmp/drago.pid";
const LOG_OUT: &str = "/tmp/drago.out";
const LOG_ERR: &str = "/tmp/drago.err";

pub fn is_daemon_running() -> io::Result<bool> {
    if !fs::metadata(PID_FILE).is_ok() {
        return Ok(false);
    }

    let pid_str = fs::read_to_string(PID_FILE)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid PID format"))?;

    // Check if process exists
    unsafe {
        if libc::kill(pid as i32, 0) == 0 {
            Ok(true)
        } else {
            // Process doesn't exist, remove stale PID file
            let _ = fs::remove_file(PID_FILE);
            Ok(false)
        }
    }
}

pub fn start_daemon() -> io::Result<()> {
    if is_daemon_running()? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Daemon already running",
        ));
    }

    let exe = std::env::current_exe()?;

    Command::new(exe)
        .arg("daemon")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Give the daemon a moment to start and write its PID
    thread::sleep(Duration::from_millis(500));

    if !is_daemon_running()? {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Failed to start daemon",
        ));
    }

    Ok(())
}

pub fn stop_daemon() -> io::Result<()> {
    if !is_daemon_running()? {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Daemon not running",
        ));
    }

    let pid_str = fs::read_to_string(PID_FILE)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid PID format"))?;

    // Send SIGTERM
    unsafe {
        if libc::kill(pid as i32, 15) != 0 {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to send SIGTERM",
            ));
        }
    }

    // Give the process a moment to exit
    thread::sleep(Duration::from_millis(1000));

    // Remove PID file
    fs::remove_file(PID_FILE)?;

    Ok(())
}

pub fn daemon_status() -> io::Result<String> {
    if is_daemon_running()? {
        let pid_str = fs::read_to_string(PID_FILE)?;
        let pid: u32 = pid_str
            .trim()
            .parse()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid PID format"))?;

        // Try to get the last sync timestamp
        let last_sync = get_last_sync_time().unwrap_or_else(|_| "Unknown".to_string());

        Ok(format!(
            "✅ Running (PID {}) - Last sync: {}",
            pid, last_sync
        ))
    } else {
        Ok("❌ Not running".to_string())
    }
}

pub fn restart_daemon() -> io::Result<()> {
    if is_daemon_running()? {
        stop_daemon()?;
    }
    start_daemon()?;
    Ok(())
}

pub fn get_last_sync_time() -> io::Result<String> {
    if !fs::metadata(LOG_OUT).is_ok() {
        return Ok("No logs available".to_string());
    }

    let log_content = fs::read_to_string(LOG_OUT)?;

    // Find the last line that contains "Sync completed"
    let last_sync_line = log_content
        .lines()
        .rev()
        .find(|line| line.contains("Sync completed"));

    if let Some(line) = last_sync_line {
        // Extract timestamp from line
        if let Some(start) = line.find('[') {
            if let Some(end) = line.find(']') {
                return Ok(line[start + 1..end].to_string());
            }
        }
        Ok("Unknown timestamp".to_string())
    } else {
        Ok("No sync completed yet".to_string())
    }
}
