use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(target_os = "windows")]
use std::process::Command;
use std::time::Duration;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

const GITHUB_LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Ba-koD/keyviewer/releases/latest";
const GITHUB_RELEASES_URL: &str =
    "https://api.github.com/repos/Ba-koD/keyviewer/releases?per_page=100";
const WINDOWS_ASSET_NAME: &str = "KBQV-windows-x64.exe";
const USER_AGENT: &str = "KeyQueueViewer";

#[derive(Clone, Debug, Serialize)]
pub struct ReleaseUpdate {
    pub current_version: String,
    pub latest_version: String,
    pub tag_name: String,
    pub release_url: String,
    pub asset_name: String,
    pub asset_size: Option<u64>,
    #[serde(skip_serializing)]
    download_url: String,
}

#[derive(Deserialize)]
struct GithubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GithubAsset>,
}

#[derive(Deserialize)]
struct GithubAsset {
    name: String,
    browser_download_url: String,
    size: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct VersionParts {
    major: u64,
    minor: u64,
    patch: u64,
}

pub fn check_latest_release() -> Result<Option<ReleaseUpdate>, String> {
    let client = http_client()?;
    let Some(release) = fetch_latest_versioned_release(&client)? else {
        return Ok(None);
    };

    let current_version = parse_version(CURRENT_VERSION)
        .ok_or_else(|| format!("Current app version is invalid: {}", CURRENT_VERSION))?;
    let latest_version = parse_version(&release.tag_name)
        .ok_or_else(|| format!("Latest release tag is not a version: {}", release.tag_name))?;

    if latest_version <= current_version {
        return Ok(None);
    }

    let asset = select_update_asset(&release.assets, latest_version).ok_or_else(|| {
        format!(
            "No Windows update asset was found in release {}",
            release.tag_name
        )
    })?;

    Ok(Some(ReleaseUpdate {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: latest_version.to_string(),
        tag_name: release.tag_name,
        release_url: release.html_url,
        asset_name: asset.name.clone(),
        asset_size: asset.size,
        download_url: asset.browser_download_url.clone(),
    }))
}

pub fn install_latest_update() -> Result<(), String> {
    let Some(update) = check_latest_release()? else {
        return Err("No newer release is available.".to_string());
    };

    install_and_restart(&update)
}

fn fetch_latest_versioned_release(client: &Client) -> Result<Option<GithubRelease>, String> {
    match client
        .get(GITHUB_LATEST_RELEASE_URL)
        .send()
        .map_err(|e| format!("Failed to request GitHub latest release: {}", e))
        .and_then(|response| {
            response
                .error_for_status()
                .map_err(|e| format!("GitHub latest release request failed: {}", e))
        })
        .and_then(|response| {
            response
                .json::<GithubRelease>()
                .map_err(|e| format!("Failed to decode GitHub latest release: {}", e))
        }) {
        Ok(release) if parse_version(&release.tag_name).is_some() => return Ok(Some(release)),
        Ok(_) | Err(_) => {}
    }

    let releases: Vec<GithubRelease> = client
        .get(GITHUB_RELEASES_URL)
        .send()
        .map_err(|e| format!("Failed to request GitHub releases: {}", e))?
        .error_for_status()
        .map_err(|e| format!("GitHub releases request failed: {}", e))?
        .json()
        .map_err(|e| format!("Failed to decode GitHub releases: {}", e))?;

    Ok(releases
        .into_iter()
        .filter_map(|release| parse_version(&release.tag_name).map(|version| (version, release)))
        .max_by(|(left, _), (right, _)| left.cmp(right))
        .map(|(_, release)| release))
}

fn select_update_asset(
    assets: &[GithubAsset],
    latest_version: VersionParts,
) -> Option<&GithubAsset> {
    let expected_windows_exe = format!("KBQV-{}-windows-x64.exe", latest_version);

    assets
        .iter()
        .find(|asset| asset.name.eq_ignore_ascii_case(WINDOWS_ASSET_NAME))
        .or_else(|| {
            assets
                .iter()
                .find(|asset| asset.name.eq_ignore_ascii_case("KBQV.exe"))
        })
        .or_else(|| {
            assets
                .iter()
                .find(|asset| asset.name.eq_ignore_ascii_case(&expected_windows_exe))
        })
        .or_else(|| {
            assets.iter().find(|asset| {
                let name = asset.name.to_ascii_lowercase();
                name.starts_with("kbqv-") && name.contains("windows") && name.ends_with(".exe")
            })
        })
}

fn install_and_restart(update: &ReleaseUpdate) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("Failed to resolve current executable: {}", e))?;
    let downloaded_exe = download_update_asset(update)?;
    schedule_replace_and_restart(&downloaded_exe, &current_exe)?;
    std::process::exit(0);
}

fn download_update_asset(update: &ReleaseUpdate) -> Result<PathBuf, String> {
    let client = http_client()?;
    let bytes = client
        .get(&update.download_url)
        .send()
        .map_err(|e| format!("Failed to download {}: {}", update.asset_name, e))?
        .error_for_status()
        .map_err(|e| format!("Update asset download failed: {}", e))?
        .bytes()
        .map_err(|e| format!("Failed to read update asset: {}", e))?;

    if bytes.is_empty() {
        return Err(format!(
            "Downloaded update asset is empty: {}",
            update.asset_name
        ));
    }

    let staging_dir = std::env::temp_dir().join(format!(
        "keyqueueviewer-update-{}-{}",
        update.latest_version,
        std::process::id()
    ));
    fs::create_dir_all(&staging_dir)
        .map_err(|e| format!("Failed to create {}: {}", staging_dir.display(), e))?;

    let output = staging_dir.join("KBQV.exe");
    fs::write(&output, &bytes)
        .map_err(|e| format!("Failed to write {}: {}", output.display(), e))?;
    Ok(output)
}

fn http_client() -> Result<Client, String> {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to create HTTP client: {}", e))
}

#[cfg(target_os = "windows")]
fn schedule_replace_and_restart(downloaded_exe: &Path, current_exe: &Path) -> Result<(), String> {
    const CREATE_NO_WINDOW: u32 = 0x08000000;

    let staging_dir = downloaded_exe
        .parent()
        .ok_or_else(|| "Downloaded update path has no parent directory".to_string())?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| "Current executable path has no parent directory".to_string())?;
    let target_temp = current_exe.with_file_name(format!(
        "{}.new",
        current_exe
            .file_name()
            .ok_or_else(|| "Current executable path has no file name".to_string())?
            .to_string_lossy()
    ));
    let log_file = staging_dir.join("update.log");
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         $log = {log}; \
         $targetTemp = {target_temp}; \
         function Write-UpdateLog([string]$message) {{ \
             Add-Content -LiteralPath $log -Value ((Get-Date -Format o) + ' ' + $message) -Encoding UTF8 -ErrorAction SilentlyContinue; \
         }} \
         try {{ \
             Write-UpdateLog 'Waiting for app process to exit.'; \
             Wait-Process -Id {pid} -ErrorAction SilentlyContinue; \
             Start-Sleep -Milliseconds 500; \
             $copied = $false; \
             for ($attempt = 1; $attempt -le 20; $attempt++) {{ \
                 try {{ \
                     Copy-Item -LiteralPath {downloaded} -Destination $targetTemp -Force -ErrorAction Stop; \
                     Move-Item -LiteralPath $targetTemp -Destination {current} -Force -ErrorAction Stop; \
                     $copied = $true; \
                     break; \
                 }} catch {{ \
                     Write-UpdateLog ('Copy attempt ' + $attempt + ' failed: ' + $_.Exception.Message); \
                     Start-Sleep -Milliseconds 500; \
                 }} \
             }} \
             if (-not $copied) {{ throw 'Failed to replace current executable.'; }} \
             Write-UpdateLog 'Starting updated app.'; \
             Start-Process -FilePath {current} -WorkingDirectory {current_dir}; \
             Start-Sleep -Seconds 2; \
             Remove-Item -LiteralPath $targetTemp -Force -ErrorAction SilentlyContinue; \
             Remove-Item -LiteralPath {downloaded} -Force -ErrorAction SilentlyContinue; \
             Remove-Item -LiteralPath {staging} -Recurse -Force -ErrorAction SilentlyContinue; \
         }} catch {{ \
             Remove-Item -LiteralPath $targetTemp -Force -ErrorAction SilentlyContinue; \
             Write-UpdateLog ('Update failed: ' + $_.Exception.Message); \
             exit 1; \
         }}",
        pid = std::process::id(),
        downloaded = powershell_literal(downloaded_exe),
        current = powershell_literal(current_exe),
        current_dir = powershell_literal(current_dir),
        target_temp = powershell_literal(&target_temp),
        staging = powershell_literal(staging_dir),
        log = powershell_literal(&log_file),
    );

    Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            &script,
        ])
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map_err(|e| format!("Failed to start updater process: {}", e))?;

    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn schedule_replace_and_restart(_downloaded_exe: &Path, _current_exe: &Path) -> Result<(), String> {
    Err("Automatic self update is only implemented for Windows builds.".to_string())
}

#[cfg(target_os = "windows")]
fn powershell_literal(path: &Path) -> String {
    let escaped = path.to_string_lossy().replace('\'', "''");
    format!("'{}'", escaped)
}

fn parse_version(value: &str) -> Option<VersionParts> {
    let trimmed = value.trim().trim_start_matches(['v', 'V']);
    let mut parts = trimmed.split(['.', '-']);
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next().unwrap_or("0").parse().ok()?;
    let patch = parts.next().unwrap_or("0").parse().ok()?;
    Some(VersionParts {
        major,
        minor,
        patch,
    })
}

impl Ord for VersionParts {
    fn cmp(&self, other: &Self) -> Ordering {
        self.major
            .cmp(&other.major)
            .then_with(|| self.minor.cmp(&other.minor))
            .then_with(|| self.patch.cmp(&other.patch))
    }
}

impl PartialOrd for VersionParts {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl std::fmt::Display for VersionParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tag_versions() {
        assert_eq!(parse_version("v1.2.3").unwrap().to_string(), "1.2.3");
        assert_eq!(parse_version("1.2").unwrap().to_string(), "1.2.0");
    }

    #[test]
    fn compares_versions_numerically() {
        assert!(parse_version("v1.10.0") > parse_version("v1.2.9"));
    }

    #[test]
    fn selects_versioned_windows_exe_asset() {
        let assets = vec![
            GithubAsset {
                name: "KBQV-1.2.3-windows-x64.zip".to_string(),
                browser_download_url: "https://example.invalid/zip".to_string(),
                size: Some(1),
            },
            GithubAsset {
                name: "KBQV-1.2.3-windows-x64.exe".to_string(),
                browser_download_url: "https://example.invalid/exe".to_string(),
                size: Some(2),
            },
        ];

        let selected = select_update_asset(&assets, parse_version("v1.2.3").unwrap()).unwrap();
        assert_eq!(selected.name, "KBQV-1.2.3-windows-x64.exe");
    }

    #[test]
    fn prefers_stable_windows_exe_asset_name() {
        let assets = vec![
            GithubAsset {
                name: "KBQV-1.2.3-windows-x64.exe".to_string(),
                browser_download_url: "https://example.invalid/versioned".to_string(),
                size: Some(1),
            },
            GithubAsset {
                name: "KBQV-windows-x64.exe".to_string(),
                browser_download_url: "https://example.invalid/stable".to_string(),
                size: Some(2),
            },
        ];

        let selected = select_update_asset(&assets, parse_version("v1.2.3").unwrap()).unwrap();
        assert_eq!(selected.name, "KBQV-windows-x64.exe");
    }
}
