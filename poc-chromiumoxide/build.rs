use std::process::Command;

fn main() {
    // Build timestamp
    // Use system time in seconds since epoch to avoid extra deps
    let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => format!("{}", d.as_secs()),
        Err(_) => "0".to_string(),
    };
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", ts);

    // Git short hash if available
    let git = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    if let Ok(out) = git {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                println!("cargo:rustc-env=BUILD_GIT_HASH={}", s);
            }
        }
    }
}
