use crate::features::packages::{PackageItem, PkgsMsg};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

pub async fn scan_packages(tx: UnboundedSender<PkgsMsg>) {
    let _ = tx.send(PkgsMsg::Start);
    let mut items = Vec::new();

    // OS-specific system package managers
    if cfg!(target_os = "macos") {
        items.extend(scan_brew().await);
        items.extend(scan_macports().await);
    } else if cfg!(target_os = "linux") {
        items.extend(scan_dpkg().await);
        items.extend(scan_pacman().await);
        items.extend(scan_rpm().await);
        items.extend(scan_brew().await); // Linuxbrew
        items.extend(scan_snap().await);
        items.extend(scan_flatpak().await);
        items.extend(scan_nix().await);
    } else if cfg!(target_os = "windows") {
        items.extend(scan_winget().await);
        items.extend(scan_scoop().await);
        items.extend(scan_choco().await);
    }

    // Cross-platform language package managers
    items.extend(scan_pip().await);
    items.extend(scan_uv().await);
    items.extend(scan_conda().await);
    items.extend(scan_npm().await);
    items.extend(scan_yarn().await);
    items.extend(scan_pnpm().await);
    items.extend(scan_cargo().await);
    items.extend(scan_gem().await);
    items.extend(scan_go().await);

    let _ = tx.send(PkgsMsg::Done(items));
}

async fn command_exists(cmd: &str) -> bool {
    #[cfg(windows)]
    let which_cmd = "where";
    #[cfg(not(windows))]
    let which_cmd = "which";

    Command::new(which_cmd)
        .arg(cmd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false)
}

// ---------------------------------------------------------------------------
// OS-specific system package managers
// ---------------------------------------------------------------------------

async fn scan_brew() -> Vec<PackageItem> {
    if !command_exists("brew").await {
        return Vec::new();
    }
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

async fn scan_macports() -> Vec<PackageItem> {
    if !command_exists("port").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("port")
        .args(["-qv", "installed"])
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
            let name = parts.next()?.to_string();
            let version = parts.last()?.to_string();
            if name.is_empty() || name.starts_with('\t') {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "macports".to_string(),
            })
        })
        .collect()
}

async fn scan_dpkg() -> Vec<PackageItem> {
    if !command_exists("dpkg-query").await {
        return Vec::new();
    }
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

async fn scan_pacman() -> Vec<PackageItem> {
    if !command_exists("pacman").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("pacman")
        .args(["-Q"])
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
            let name = parts.next()?.to_string();
            let version = parts.last()?.to_string();
            if name.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "pacman".to_string(),
            })
        })
        .collect()
}

async fn scan_rpm() -> Vec<PackageItem> {
    if !command_exists("rpm").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("rpm")
        .args(["-qa", "--queryformat", "%{NAME}\t%{VERSION}-%{RELEASE}\n"])
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
                manager: "rpm".to_string(),
            })
        })
        .collect()
}

async fn scan_snap() -> Vec<PackageItem> {
    if !command_exists("snap").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("snap")
        .args(["list"])
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
        .skip(1) // header
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "snap".to_string(),
            })
        })
        .collect()
}

async fn scan_flatpak() -> Vec<PackageItem> {
    if !command_exists("flatpak").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("flatpak")
        .args(["list", "--columns=application,version"])
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
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "flatpak".to_string(),
            })
        })
        .collect()
}

async fn scan_nix() -> Vec<PackageItem> {
    if !command_exists("nix").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("nix-env")
        .args(["-q", "--json"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    // Simple JSON parsing without serde: find "name" and "version" patterns
    let mut items = Vec::new();
    for chunk in json_str.split('{') {
        if let Some(name_start) = chunk.find("\"name\":\"") {
            let name_rest = &chunk[name_start + 8..];
            let name_end = name_rest.find('"').unwrap_or(0);
            let name = name_rest[..name_end].to_string();

            if let Some(ver_start) = chunk.find("\"version\":\"") {
                let ver_rest = &chunk[ver_start + 11..];
                let ver_end = ver_rest.find('"').unwrap_or(0);
                let version = ver_rest[..ver_end].to_string();
                if !name.is_empty() && !version.is_empty() {
                    items.push(PackageItem {
                        name,
                        version,
                        manager: "nix".to_string(),
                    });
                }
            }
        }
    }
    items
}

async fn scan_winget() -> Vec<PackageItem> {
    if !command_exists("winget").await {
        return Vec::new();
    }
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
            if name == "Name" || name.chars().all(|c| c == '-') {
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

async fn scan_scoop() -> Vec<PackageItem> {
    if !command_exists("scoop").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("scoop")
        .args(["list"])
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
        .skip(1) // header
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "scoop".to_string(),
            })
        })
        .collect()
}

async fn scan_choco() -> Vec<PackageItem> {
    if !command_exists("choco").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("choco")
        .args(["list", "--limit-output"])
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
            // choco --limit-output uses pipe-delimited: name|version|other
            let mut parts = line.split('|');
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "choco".to_string(),
            })
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Cross-platform language package managers
// ---------------------------------------------------------------------------

async fn scan_pip() -> Vec<PackageItem> {
    let pip_cmd = if command_exists("pip3").await {
        "pip3"
    } else if command_exists("pip").await {
        "pip"
    } else {
        return Vec::new();
    };

    let Ok(output) = Command::new(pip_cmd)
        .args(["list", "--format=columns"])
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
        .skip(2) // header and separator
        .filter_map(|line| {
            let line = String::from_utf8_lossy(line);
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "pip".to_string(),
            })
        })
        .collect()
}

async fn scan_uv() -> Vec<PackageItem> {
    if !command_exists("uv").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("uv")
        .args(["tool", "list"])
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
            // uv tool list: "name v1.2.3"
            let line = line.trim();
            if line.is_empty() || line.starts_with('-') {
                return None;
            }
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.next()?.trim_start_matches('v').to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "uv".to_string(),
            })
        })
        .collect()
}

async fn scan_conda() -> Vec<PackageItem> {
    let conda_cmd = if command_exists("conda").await {
        "conda"
    } else if command_exists("mamba").await {
        "mamba"
    } else {
        return Vec::new();
    };

    let Ok(output) = Command::new(conda_cmd)
        .args(["list", "--json"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    // conda list --json returns an array of objects with "name" and "version"
    for chunk in json_str.split('{') {
        if let Some(name_start) = chunk.find("\"name\":\"") {
            let name_rest = &chunk[name_start + 8..];
            let name_end = name_rest.find('"').unwrap_or(0);
            let name = name_rest[..name_end].to_string();

            if let Some(ver_start) = chunk.find("\"version\":\"") {
                let ver_rest = &chunk[ver_start + 11..];
                let ver_end = ver_rest.find('"').unwrap_or(0);
                let version = ver_rest[..ver_end].to_string();
                if !name.is_empty() && !version.is_empty() {
                    items.push(PackageItem {
                        name,
                        version,
                        manager: "conda".to_string(),
                    });
                }
            }
        }
    }
    items
}

async fn scan_npm() -> Vec<PackageItem> {
    if !command_exists("npm").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("npm")
        .args(["list", "-g", "--depth=0"])
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
            // Lines look like: "  package@1.2.3" or "  @scope/package@1.2.3"
            let line = line.trim_start();
            if line.is_empty() || line.starts_with('/') {
                return None;
            }
            let (name, version) = if let Some(at_pos) = line.rfind('@') {
                let (name, ver) = line.split_at(at_pos);
                (name.to_string(), ver[1..].to_string())
            } else {
                return None;
            };
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "npm".to_string(),
            })
        })
        .collect()
}

async fn scan_yarn() -> Vec<PackageItem> {
    if !command_exists("yarn").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("yarn")
        .args(["global", "list", "--json"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let mut items = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.starts_with('{') {
            continue;
        }
        // yarn --json outputs objects with "data" containing "body" with "name@version"
        if let Some(data_start) = line.find("\"body\":\"") {
            let body_rest = &line[data_start + 8..];
            let body_end = body_rest.find('"').unwrap_or(0);
            let body = &body_rest[..body_end];
            // body is like: "package@1.2.3" or "info \"package@1.2.3\" has binaries"
            if let Some(at_pos) = body.rfind('@') {
                let (name, ver) = body.split_at(at_pos);
                let name = name.trim().trim_start_matches('"').trim_start_matches("info \"");
                let version = &ver[1..];
                if !name.is_empty() && !version.is_empty() {
                    items.push(PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        manager: "yarn".to_string(),
                    });
                }
            }
        }
    }
    items
}

async fn scan_pnpm() -> Vec<PackageItem> {
    if !command_exists("pnpm").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("pnpm")
        .args(["list", "-g", "--depth=0"])
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
            let line = line.trim();
            // Lines like: "├── package@1.2.3" or "└── @scope/package@1.2.3"
            let line = line
                .trim_start_matches('├')
                .trim_start_matches('└')
                .trim_start_matches("──")
                .trim();
            if line.is_empty() {
                return None;
            }
            let (name, version) = if let Some(at_pos) = line.rfind('@') {
                let (name, ver) = line.split_at(at_pos);
                (name.to_string(), ver[1..].to_string())
            } else {
                return None;
            };
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "pnpm".to_string(),
            })
        })
        .collect()
}

async fn scan_cargo() -> Vec<PackageItem> {
    if !command_exists("cargo").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("cargo")
        .args(["install", "--list"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let mut items = Vec::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lines = stdout.lines().peekable();

    while let Some(line) = lines.next() {
        let line = line.trim();
        // Lines like: "package_name v1.2.3:" (with colon)
        if line.ends_with(':') && !line.starts_with('/') {
            let name_version = &line[..line.len() - 1]; // strip trailing ':'
            let mut parts = name_version.split_whitespace();
            let name = parts.next().unwrap_or("").to_string();
            let version = parts
                .next()
                .unwrap_or("")
                .trim_start_matches('v')
                .to_string();
            if !name.is_empty() && !version.is_empty() {
                items.push(PackageItem {
                    name,
                    version,
                    manager: "cargo".to_string(),
                });
            }
        }
    }
    items
}

async fn scan_gem() -> Vec<PackageItem> {
    if !command_exists("gem").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("gem")
        .args(["list", "--local"])
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
            // Lines like: "rails (7.1.2)"
            let line = line.trim();
            let name_end = line.find(' ')?;
            let name = line[..name_end].to_string();
            let version = line[name_end..]
                .trim()
                .trim_start_matches('(')
                .trim_end_matches(')')
                .to_string();
            if name.is_empty() || version.is_empty() {
                return None;
            }
            Some(PackageItem {
                name,
                version,
                manager: "gem".to_string(),
            })
        })
        .collect()
}

async fn scan_go() -> Vec<PackageItem> {
    if !command_exists("go").await {
        return Vec::new();
    }
    let Ok(output) = Command::new("go")
        .args(["list", "-m", "-json", "all"])
        .output()
        .await
    else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let mut items = Vec::new();
    for chunk in json_str.split('{') {
        if let Some(path_start) = chunk.find("\"Path\":\"") {
            let path_rest = &chunk[path_start + 8..];
            let path_end = path_rest.find('"').unwrap_or(0);
            let path = path_rest[..path_end].to_string();

            if let Some(ver_start) = chunk.find("\"Version\":\"") {
                let ver_rest = &chunk[ver_start + 11..];
                let ver_end = ver_rest.find('"').unwrap_or(0);
                let version = ver_rest[..ver_end].to_string();
                if !path.is_empty() && !version.is_empty() {
                    // Use the last segment of the path as the name
                    let name = path.rsplit('/').next().unwrap_or(&path).to_string();
                    items.push(PackageItem {
                        name,
                        version,
                        manager: "go".to_string(),
                    });
                }
            }
        }
    }
    items
}
