use crate::features::packages::{PackageItem, PkgsMsg};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

pub async fn scan_packages(tx: UnboundedSender<PkgsMsg>) {
    let _ = tx.send(PkgsMsg::Start);
    let items = if cfg!(target_os = "macos") {
        scan_brew().await
    } else if cfg!(target_os = "linux") {
        scan_dpkg().await
    } else if cfg!(target_os = "windows") {
        scan_winget().await
    } else {
        Vec::new()
    };

    let _ = tx.send(PkgsMsg::Done(items));
}

async fn scan_brew() -> Vec<PackageItem> {
    let Ok(output) = Command::new("brew")
        .args(["list", "--versions"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let mut parts = line.split_whitespace();
            Some(PackageItem {
                name: parts.next()?.to_string(),
                version: parts.last()?.to_string(),
                manager: "brew".to_string(),
            })
        })
        .collect()
}

async fn scan_dpkg() -> Vec<PackageItem> {
    let Ok(output) = Command::new("dpkg-query")
        .args(["-W", "-f=${binary:Package}\t${Version}\n"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    output
        .stdout
        .split(|byte| *byte == b'\n')
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let (name, version) = line.split_once('\t')?;
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name: name.to_string(),
                version: version.to_string(),
                manager: "apt".to_string(),
            })
        })
        .collect()
}

async fn scan_winget() -> Vec<PackageItem> {
    let Ok(output) = Command::new("winget")
        .args(["list", "--disable-interactivity"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    output
        .stdout
        .split(|byte| *byte == b'\n')
        .skip(2)
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name == "Name" || name.chars().all(|character| character == '-') {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "winget".to_string(),
            })
        })
        .collect()
}
