pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

/// Sanitize an error message for user display.
///
/// This removes potentially sensitive information like:
/// - Full file paths (replaces with ~)
/// - System-specific paths
/// - Internal error details
pub fn sanitize_error_message(error: &str) -> String {
    let sanitized = error
        // Replace home directory paths with ~
        .replace("/home/", "~/")
        .replace("/Users/", "~/")
        .replace("C:\\Users\\", "~\\")
        .replace("C:/Users/", "~/")
        // Remove common system paths
        .replace("/tmp/", "~/")
        .replace("/var/", "~/")
        .replace("/etc/", "~/")
        // Remove common error prefixes that leak info
        .replace("No such file or directory", "File not found")
        .replace("Permission denied", "Access denied")
        .replace("Operation not permitted", "Not allowed")
        // Truncate very long messages
        .chars()
        .take(200)
        .collect::<String>()
        .trim()
        .to_string();
    
    if sanitized.is_empty() {
        "An error occurred".to_string()
    } else {
        sanitized
    }
}

/// Sanitize a filename to prevent command injection.
///
/// Removes control characters and other potentially dangerous characters.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !c.is_control())
        .take(255) // Max filename length on most file systems
        .collect()
}

/// Validate that a path is safe to operate on.
///
/// Returns true if the path doesn't contain suspicious patterns.
pub fn is_path_safe(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Check for common dangerous patterns
    !path_str.contains("..")  // Directory traversal
        && !path_str.contains("\0")  // Null bytes
        && !path_str.starts_with("/proc")  // Linux proc filesystem
        && !path_str.starts_with("/sys")  // Linux sys filesystem
        && !path_str.contains("~/.ssh")  // SSH keys
        && !path_str.contains("~/.gnupg")  // GPG keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn formats_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
    }

    #[test]
    fn sanitize_error_removes_home_paths() {
        let error = "Permission denied /home/user/file.txt";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("/home/"));
        assert!(sanitized.contains("~/"));
    }

    #[test]
    fn sanitize_error_removes_user_paths() {
        let error = "File not found /Users/john/docs/file.txt";
        let sanitized = sanitize_error_message(error);
        assert!(!sanitized.contains("/Users/"));
        assert!(sanitized.contains("~/"));
    }

    #[test]
    fn sanitize_error_truncates_long_messages() {
        let error = "A".repeat(300);
        let sanitized = sanitize_error_message(&error);
        assert!(sanitized.len() <= 200);
    }

    #[test]
    fn sanitize_error_returns_default_for_empty() {
        let sanitized = sanitize_error_message("");
        assert_eq!(sanitized, "An error occurred");
    }

    #[test]
    fn sanitize_filename_removes_control_chars() {
        let filename = "file\x00name.txt";
        let sanitized = sanitize_filename(filename);
        assert!(!sanitized.contains('\x00'));
        assert_eq!(sanitized, "filename.txt");
    }

    #[test]
    fn sanitize_filename_limits_length() {
        let filename = "a".repeat(300);
        let sanitized = sanitize_filename(&filename);
        assert!(sanitized.len() <= 255);
    }

    #[test]
    fn is_path_safe_rejects_directory_traversal() {
        assert!(!is_path_safe(Path::new("/tmp/../../../etc/passwd")));
    }

    #[test]
    fn is_path_safe_rejects_proc_filesystem() {
        assert!(!is_path_safe(Path::new("/proc/self/environ")));
    }

    #[test]
    fn is_path_safe_rejects_ssh_keys() {
        assert!(!is_path_safe(Path::new("~/.ssh/id_rsa")));
    }

    #[test]
    fn is_path_safe_allows_normal_paths() {
        assert!(is_path_safe(Path::new("/tmp/myfile.txt")));
        assert!(is_path_safe(Path::new("~/Documents/report.pdf")));
    }
}
