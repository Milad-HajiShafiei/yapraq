use crate::features::packages::{PackageItem, PkgsMsg};
use tokio::process::Command;
use tokio::sync::mpsc::UnboundedSender;

pub async fn scan_packages(tx: UnboundedSender<PkgsMsg>) {
    let _ = tx.send(PkgsMsg::Start);
    let mut items = Vec::new();

    if cfg!(target_os = "macos") {
        items.extend(scan_brew().await);
        items.extend(scan_macports().await);
    } else if cfg!(target_os = "linux") {
        items.extend(scan_dpkg().await);
        items.extend(scan_pacman().await);
        items.extend(scan_rpm().await);
        items.extend(scan_brew().await);
        items.extend(scan_snap().await);
        items.extend(scan_flatpak().await);
        items.extend(scan_nix().await);
    } else if cfg!(target_os = "windows") {
        items.extend(scan_winget().await);
        items.extend(scan_scoop().await);
        items.extend(scan_choco().await);
    }

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

async fn run_command(command: &str, args: &[&str]) -> Option<Vec<u8>> {
    if !command_exists(command).await {
        return None;
    }
    let output = Command::new(command).args(args).output().await.ok()?;
    output.status.success().then_some(output.stdout)
}

fn lines(bytes: &[u8]) -> impl Iterator<Item = &str> {
    std::str::from_utf8(bytes).unwrap_or_default().lines()
}

fn parse_whitespace_packages(bytes: &[u8], manager: &str, skip: usize) -> Vec<PackageItem> {
    lines(bytes)
        .skip(skip)
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let name = parts.next()?.to_string();
            let version = parts.last()?.to_string();
            (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                name,
                version,
                manager: manager.to_string(),
            })
        })
        .collect()
}

async fn scan_brew() -> Vec<PackageItem> {
    run_command("brew", &["list", "--versions"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "brew", 0)
        })
}

async fn scan_macports() -> Vec<PackageItem> {
    run_command("port", &["-qv", "installed"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "macports", 0)
        })
}

async fn scan_dpkg() -> Vec<PackageItem> {
    run_command("dpkg-query", &["-W", "-f=${binary:Package}\t${Version}\n"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(|line| {
                    let (name, version) = line.split_once('\t')?;
                    (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        manager: "apt".to_string(),
                    })
                })
                .collect()
        })
}

async fn scan_pacman() -> Vec<PackageItem> {
    run_command("pacman", &["-Q"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "pacman", 0)
        })
}

async fn scan_rpm() -> Vec<PackageItem> {
    run_command(
        "rpm",
        &["-qa", "--queryformat", "%{NAME}\t%{VERSION}-%{RELEASE}\n"],
    )
    .await
    .map_or_else(Vec::new, |output| {
        lines(&output)
            .filter_map(|line| {
                let (name, version) = line.split_once('\t')?;
                (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                    name: name.to_string(),
                    version: version.to_string(),
                    manager: "rpm".to_string(),
                })
            })
            .collect()
    })
}

async fn scan_snap() -> Vec<PackageItem> {
    run_command("snap", &["list"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "snap", 1)
        })
}

async fn scan_flatpak() -> Vec<PackageItem> {
    run_command("flatpak", &["list", "--columns=application,version"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "flatpak", 0)
        })
}

async fn scan_nix() -> Vec<PackageItem> {
    run_command("nix-env", &["-q", "--json"])
        .await
        .map_or_else(Vec::new, |output| parse_json_packages(&output, "nix"))
}

async fn scan_winget() -> Vec<PackageItem> {
    run_command("winget", &["list", "--disable-interactivity"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "winget", 2)
        })
}

async fn scan_scoop() -> Vec<PackageItem> {
    run_command("scoop", &["list"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "scoop", 1)
        })
}

async fn scan_choco() -> Vec<PackageItem> {
    run_command("choco", &["list", "--limit-output"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(|line| {
                    let (name, version) = line.split_once('|')?;
                    (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        manager: "choco".to_string(),
                    })
                })
                .collect()
        })
}

async fn scan_pip() -> Vec<PackageItem> {
    let command = if command_exists("pip3").await {
        "pip3"
    } else {
        "pip"
    };
    run_command(command, &["list", "--format=columns"])
        .await
        .map_or_else(Vec::new, |output| {
            parse_whitespace_packages(&output, "pip", 2)
        })
}

async fn scan_uv() -> Vec<PackageItem> {
    run_command("uv", &["tool", "list"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(parse_at_version)
                .map(|(name, version)| PackageItem {
                    name,
                    version,
                    manager: "uv".to_string(),
                })
                .collect()
        })
}

async fn scan_conda() -> Vec<PackageItem> {
    let command = if command_exists("conda").await {
        "conda"
    } else {
        "mamba"
    };
    run_command(command, &["list", "--json"])
        .await
        .map_or_else(Vec::new, |output| parse_json_packages(&output, "conda"))
}

async fn scan_npm() -> Vec<PackageItem> {
    run_command("npm", &["list", "-g", "--depth=0"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(parse_at_version)
                .map(|(name, version)| PackageItem {
                    name,
                    version,
                    manager: "npm".to_string(),
                })
                .collect()
        })
}

async fn scan_yarn() -> Vec<PackageItem> {
    run_command("yarn", &["global", "list", "--depth=0"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(|line| {
                    let package = line
                        .split_once('"')
                        .and_then(|(_, rest)| rest.split_once('"').map(|(p, _)| p))
                        .unwrap_or(line);
                    parse_at_version(package)
                })
                .map(|(name, version)| PackageItem {
                    name,
                    version,
                    manager: "yarn".to_string(),
                })
                .collect()
        })
}

async fn scan_pnpm() -> Vec<PackageItem> {
    run_command("pnpm", &["list", "-g", "--depth=0"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(parse_at_version)
                .map(|(name, version)| PackageItem {
                    name,
                    version,
                    manager: "pnpm".to_string(),
                })
                .collect()
        })
}

async fn scan_cargo() -> Vec<PackageItem> {
    run_command("cargo", &["install", "--list"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(|line| {
                    let line = line.trim();
                    let name_version = line.strip_suffix(':')?;
                    let (name, version) = name_version.split_once(' ')?;
                    let version = version.trim().strip_prefix('v')?;
                    (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        manager: "cargo".to_string(),
                    })
                })
                .collect()
        })
}

async fn scan_gem() -> Vec<PackageItem> {
    run_command("gem", &["list", "--local"])
        .await
        .map_or_else(Vec::new, |output| {
            lines(&output)
                .filter_map(|line| {
                    let line = line.trim();
                    let name_end = line.find(' ')?;
                    let name = &line[..name_end];
                    let version = line[name_end..]
                        .trim()
                        .strip_prefix('(')?
                        .strip_suffix(')')?;
                    (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        manager: "gem".to_string(),
                    })
                })
                .collect()
        })
}

async fn scan_go() -> Vec<PackageItem> {
    run_command("go", &["list", "-m", "-json", "all"])
        .await
        .map_or_else(Vec::new, |output| parse_json_packages(&output, "go"))
}

fn parse_json_packages(bytes: &[u8], manager: &str) -> Vec<PackageItem> {
    let text = String::from_utf8_lossy(bytes);
    text.split('{')
        .skip(1)
        .filter_map(|object| {
            let name =
                json_string_field(object, "name").or_else(|| json_string_field(object, "Path"))?;
            let version = json_string_field(object, "version")
                .or_else(|| json_string_field(object, "Version"))?;
            (!name.is_empty() && !version.is_empty()).then_some(PackageItem {
                name: name.rsplit('/').next().unwrap_or(&name).to_string(),
                version,
                manager: manager.to_string(),
            })
        })
        .collect()
}

fn json_string_field(object: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\"");
    let key_end = object.find(&marker)? + marker.len();
    let value = object[key_end..]
        .trim_start()
        .strip_prefix(':')?
        .trim_start();
    let value = value.strip_prefix('"')?;
    let mut escaped = false;
    for (index, character) in value.char_indices() {
        if escaped {
            escaped = false;
        } else if character == '\\' {
            escaped = true;
        } else if character == '"' {
            return Some(value[..index].to_string());
        }
    }
    None
}

fn parse_at_version(line: &str) -> Option<(String, String)> {
    let line = line
        .trim()
        .trim_start_matches(|character| matches!(character, '├' | '└' | '─' | '│'))
        .trim();
    let (name, version) = line.rsplit_once('@')?;
    let version = version.split_whitespace().next()?;
    if name.is_empty() || name.chars().any(char::is_whitespace) || version.is_empty() {
        return None;
    }
    Some((
        name.to_string(),
        version.trim_start_matches('v').to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::{parse_at_version, parse_json_packages};

    #[test]
    fn parses_scoped_and_tree_prefixed_javascript_packages() {
        assert_eq!(
            parse_at_version("├── @scope/tool@1.2.3 deduped"),
            Some(("@scope/tool".to_string(), "1.2.3".to_string()))
        );
        assert_eq!(parse_at_version("npm v10.0.0"), None);
    }

    #[test]
    fn parses_pretty_printed_manager_json() {
        let json = br#"{
            "name": "demo",
            "version": "1.2.3"
        }"#;
        let packages = parse_json_packages(json, "test");
        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "demo");
        assert_eq!(packages[0].version, "1.2.3");
        assert_eq!(packages[0].manager, "test");
    }
}
