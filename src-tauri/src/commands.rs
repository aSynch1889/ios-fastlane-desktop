use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectConfig {
    pub project_path: String,
    pub workspace: String,
    pub xcodeproj: String,
    pub scheme_dev: String,
    pub scheme_dis: String,
    pub bundle_id_dev: String,
    pub bundle_id_dis: String,
    pub team_id: String,
    pub signing_style: String,
    pub match_git_url: String,
    pub match_git_branch: String,
    pub pgyer_api_key: String,
    pub app_store_connect_api_key_path: String,
    pub enable_quality_gate: bool,
    pub enable_tests: bool,
    pub enable_swiftlint: bool,
    pub enable_snapshot: bool,
    pub metadata_path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub project_name: String,
    pub workspace: Option<String>,
    pub xcodeproj: Option<String>,
    pub schemes: Vec<String>,
    pub bundle_id_dev: Option<String>,
    pub bundle_id_dis: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneRunResult {
    pub status: String,
    pub exit_code: i32,
    pub output: String,
    pub lane: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityResult {
    pub bundle_id_dev: Option<String>,
    pub bundle_id_dis: Option<String>,
    pub team_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorCheck {
    pub name: String,
    pub status: String,
    pub detail: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorReport {
    pub checks: Vec<DoctorCheck>,
}

#[tauri::command]
pub fn doctor_check(project_path: Option<String>) -> Result<DoctorReport, String> {
    let root = project_path
        .filter(|p| !p.trim().is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

    let checks = vec![
        check_cmd("Xcode CLI", "/bin/zsh", &["-lc", "xcode-select -p"], None),
        check_cmd("Xcode Build", "/bin/zsh", &["-lc", "xcodebuild -version"], None),
        check_cmd("Ruby", "/bin/zsh", &["-lc", "ruby -v"], Some("Install Ruby and ensure it is in PATH.")),
        check_cmd(
            "Bundler",
            "/bin/zsh",
            &["-lc", "bundle -v"],
            Some("Run `gem install bundler` or ensure Bundler is available."),
        ),
        check_cmd(
            "Fastlane",
            "/bin/zsh",
            &["-lc", "fastlane --version"],
            Some("Run `bundle install` or install fastlane."),
        ),
        check_cmd(
            "CocoaPods",
            "/bin/zsh",
            &["-lc", "pod --version"],
            Some("Install CocoaPods if your project depends on Pods."),
        ),
        check_cmd(
            "Gemfile",
            "/bin/zsh",
            &["-lc", &format!("cd '{}' && test -f Gemfile && echo ok", escape_single_quote(&root.to_string_lossy()))],
            Some("Create Gemfile to manage fastlane gems consistently."),
        ),
    ];

    Ok(DoctorReport { checks })
}

#[tauri::command]
pub fn scan_project(project_path: String) -> Result<ScanResult, String> {
    let root = PathBuf::from(project_path.clone());
    if !root.exists() {
        return Err(format!("Project path not found: {}", project_path));
    }

    let workspace = find_first_with_ext(&root, "xcworkspace");
    let xcodeproj = find_first_with_ext(&root, "xcodeproj");

    let schemes = match parse_schemes_from_xcodebuild(&root, workspace.as_deref(), xcodeproj.as_deref()) {
        Ok(list) if !list.is_empty() => list,
        _ => vec![],
    };
    let (scheme_dev, scheme_dis) = pick_dev_dis_schemes(&schemes);
    let identity = resolve_identity_internal(
        &root,
        workspace.as_deref(),
        xcodeproj.as_deref(),
        scheme_dev,
        scheme_dis,
    );
    let project_name = root
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("iOSProject")
        .to_string();

    Ok(ScanResult {
        project_name,
        workspace,
        xcodeproj,
        schemes,
        bundle_id_dev: identity.bundle_id_dev,
        bundle_id_dis: identity.bundle_id_dis,
        team_id: identity.team_id,
    })
}

#[tauri::command]
pub fn resolve_identity(
    project_path: String,
    workspace: Option<String>,
    xcodeproj: Option<String>,
    scheme_dev: String,
    scheme_dis: String,
) -> Result<IdentityResult, String> {
    let root = PathBuf::from(&project_path);
    if !root.exists() {
        return Err(format!("Project path not found: {}", project_path));
    }

    let resolved_workspace = workspace
        .filter(|v| !v.trim().is_empty())
        .or_else(|| find_first_with_ext(&root, "xcworkspace"));
    let resolved_xcodeproj = xcodeproj
        .filter(|v| !v.trim().is_empty())
        .or_else(|| find_first_with_ext(&root, "xcodeproj"));

    Ok(resolve_identity_internal(
        &root,
        resolved_workspace.as_deref(),
        resolved_xcodeproj.as_deref(),
        Some(scheme_dev),
        Some(scheme_dis),
    ))
}

#[tauri::command]
pub fn save_profile(config: ProjectConfig) -> Result<String, String> {
    let project_root = PathBuf::from(&config.project_path);
    if !project_root.exists() {
        return Err(format!("projectPath does not exist: {}", config.project_path));
    }

    let profile_dir = project_root.join(".fastlane-desktop");
    fs::create_dir_all(&profile_dir).map_err(|e| format!("Create profile dir failed: {}", e))?;
    let profile_path = profile_dir.join("profile.json");
    let payload = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("Serialize profile failed: {}", e))?;
    fs::write(&profile_path, payload).map_err(|e| format!("Write profile failed: {}", e))?;

    Ok(format!("Profile saved: {}", profile_path.display()))
}

#[tauri::command]
pub fn load_profile(project_path: String) -> Result<ProjectConfig, String> {
    let root = PathBuf::from(&project_path);
    let profile_path = root.join(".fastlane-desktop").join("profile.json");
    if !profile_path.exists() {
        return Err(format!("Profile not found: {}", profile_path.display()));
    }

    let content = fs::read_to_string(&profile_path)
        .map_err(|e| format!("Read profile failed: {}", e))?;
    serde_json::from_str::<ProjectConfig>(&content)
        .map_err(|e| format!("Parse profile failed: {}", e))
}

#[tauri::command]
pub fn generate_fastlane_files(config: ProjectConfig) -> Result<String, String> {
    let project_root = PathBuf::from(&config.project_path);
    if !project_root.exists() {
        return Err(format!("projectPath does not exist: {}", config.project_path));
    }

    let fastlane_dir = project_root.join("fastlane");
    fs::create_dir_all(&fastlane_dir).map_err(|e| format!("Create fastlane dir failed: {}", e))?;

    let env_file = fastlane_dir.join(".env.fastlane");
    let env_content = render_env(&config);
    fs::write(&env_file, env_content).map_err(|e| format!("Write env failed: {}", e))?;

    let readme = fastlane_dir.join("DESKTOP_GENERATED_NOTE.md");
    let note = format!(
        "# Generated by iOS Fastlane Desktop\\n\\n- signing_style: {}\\n- scheme_dev: {}\\n- scheme_dis: {}\\n",
        config.signing_style, config.scheme_dev, config.scheme_dis
    );
    fs::write(&readme, note).map_err(|e| format!("Write note failed: {}", e))?;

    Ok(format!(
        "Generated files:\\n- {}\\n- {}",
        env_file.display(),
        readme.display()
    ))
}

#[tauri::command]
pub fn run_lane(project_path: String, lane: String) -> Result<LaneRunResult, String> {
    let output = Command::new("/bin/zsh")
        .arg("-lc")
        .arg(format!(
            "cd '{}' && bundle exec fastlane ios {}",
            escape_single_quote(&project_path),
            lane
        ))
        .output()
        .map_err(|e| format!("Failed to run lane: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);
    let status = if output.status.success() { "success" } else { "failed" };

    Ok(LaneRunResult {
        status: status.to_string(),
        exit_code,
        output: format!("{}\\n{}", stdout, stderr),
        lane,
    })
}

fn find_first_with_ext(root: &Path, ext: &str) -> Option<String> {
    for entry in WalkDir::new(root)
        .max_depth(4)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_dir() && path.extension().and_then(OsStr::to_str) == Some(ext) {
            return path
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().to_string());
        }
    }
    None
}

fn render_env(config: &ProjectConfig) -> String {
    let team_id = if config.team_id.trim().is_empty() {
        "TODO_TEAM_ID"
    } else {
        &config.team_id
    };

    [
        format!("SCHEME_DEV={}", config.scheme_dev),
        format!("SCHEME_DIS={}", config.scheme_dis),
        format!("BUNDLE_ID_DEV={}", config.bundle_id_dev),
        format!("BUNDLE_ID_DIS={}", config.bundle_id_dis),
        format!("TEAM_ID={}", team_id),
        format!("SIGNING_STYLE={}", config.signing_style),
        format!("MATCH_GIT_URL={}", config.match_git_url),
        format!("MATCH_GIT_BRANCH={}", config.match_git_branch),
        format!("PGYER_API_KEY={}", config.pgyer_api_key),
        format!(
            "APP_STORE_CONNECT_API_KEY_PATH={}",
            config.app_store_connect_api_key_path
        ),
        format!("ENABLE_QUALITY_GATE={}", config.enable_quality_gate),
        format!("ENABLE_TESTS={}", config.enable_tests),
        format!("ENABLE_SWIFTLINT={}", config.enable_swiftlint),
        format!("ENABLE_SNAPSHOT={}", config.enable_snapshot),
        format!("METADATA_PATH={}", config.metadata_path),
    ]
    .join("\n")
}

fn escape_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn parse_schemes_from_xcodebuild(
    root: &Path,
    workspace: Option<&str>,
    xcodeproj: Option<&str>,
) -> Result<Vec<String>, String> {
    let target_arg = if let Some(ws) = workspace {
        format!("-workspace '{}'", escape_single_quote(ws))
    } else if let Some(proj) = xcodeproj {
        format!("-project '{}'", escape_single_quote(proj))
    } else {
        return Ok(vec![]);
    };

    let cmd = format!(
        "cd '{}' && xcodebuild -list {}",
        escape_single_quote(&root.to_string_lossy()),
        target_arg
    );

    let output = Command::new("/bin/zsh")
        .arg("-lc")
        .arg(cmd)
        .output()
        .map_err(|e| format!("xcodebuild -list failed: {}", e))?;

    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(extract_schemes(&text))
}

fn extract_schemes(xcodebuild_output: &str) -> Vec<String> {
    let mut in_schemes = false;
    let mut schemes = vec![];

    for line in xcodebuild_output.lines() {
        let trimmed = line.trim();
        if trimmed.eq_ignore_ascii_case("Schemes:") {
            in_schemes = true;
            continue;
        }
        if !in_schemes {
            continue;
        }

        if trimmed.is_empty() {
            if !schemes.is_empty() {
                break;
            }
            continue;
        }

        // Stop when entering another top-level section.
        if !line.starts_with(' ') && !line.starts_with('\t') {
            break;
        }
        schemes.push(trimmed.to_string());
    }

    schemes
}

fn pick_dev_dis_schemes(schemes: &[String]) -> (Option<String>, Option<String>) {
    if schemes.is_empty() {
        return (None, None);
    }

    let dev = schemes
        .iter()
        .find(|s| {
            let lower = s.to_lowercase();
            lower.contains("dev") || lower.contains("debug") || lower.contains("staging")
        })
        .cloned()
        .or_else(|| schemes.first().cloned());

    let dis = schemes
        .iter()
        .find(|s| {
            let lower = s.to_lowercase();
            lower.contains("prod") || lower.contains("release") || lower.contains("appstore")
        })
        .cloned()
        .or_else(|| schemes.iter().find(|s| Some((*s).clone()) != dev).cloned())
        .or_else(|| schemes.first().cloned());

    (dev, dis)
}

fn resolve_build_setting(
    root: &Path,
    workspace: Option<&str>,
    xcodeproj: Option<&str>,
    scheme: &str,
    key: &str,
) -> Option<String> {
    let target_arg = if let Some(ws) = workspace {
        format!("-workspace '{}'", escape_single_quote(ws))
    } else if let Some(proj) = xcodeproj {
        format!("-project '{}'", escape_single_quote(proj))
    } else {
        return None;
    };

    let cmd = format!(
        "cd '{}' && xcodebuild -showBuildSettings {} -scheme '{}'",
        escape_single_quote(&root.to_string_lossy()),
        target_arg,
        escape_single_quote(scheme)
    );

    let output = Command::new("/bin/zsh").arg("-lc").arg(cmd).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    extract_build_setting(&text, key)
}

fn extract_build_setting(output: &str, key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            let parsed = value.trim();
            if !parsed.is_empty() {
                return Some(parsed.to_string());
            }
        }
    }
    None
}

fn check_cmd(
    name: &str,
    program: &str,
    args: &[&str],
    suggestion: Option<&str>,
) -> DoctorCheck {
    let out = Command::new(program).args(args).output();
    match out {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            if output.status.success() {
                let detail = if stdout.is_empty() { "ok".to_string() } else { stdout };
                DoctorCheck {
                    name: name.to_string(),
                    status: "pass".to_string(),
                    detail,
                    suggestion: None,
                }
            } else {
                let detail = if !stderr.is_empty() { stderr } else { stdout };
                DoctorCheck {
                    name: name.to_string(),
                    status: "warn".to_string(),
                    detail: if detail.is_empty() { "command failed".to_string() } else { detail },
                    suggestion: suggestion.map(|s| s.to_string()),
                }
            }
        }
        Err(e) => DoctorCheck {
            name: name.to_string(),
            status: "warn".to_string(),
            detail: format!("failed to execute: {}", e),
            suggestion: suggestion.map(|s| s.to_string()),
        },
    }
}

fn resolve_identity_internal(
    root: &Path,
    workspace: Option<&str>,
    xcodeproj: Option<&str>,
    scheme_dev: Option<String>,
    scheme_dis: Option<String>,
) -> IdentityResult {
    let bundle_id_dev = scheme_dev
        .as_deref()
        .and_then(|scheme| resolve_build_setting(root, workspace, xcodeproj, scheme, "PRODUCT_BUNDLE_IDENTIFIER"));
    let bundle_id_dis = scheme_dis
        .as_deref()
        .and_then(|scheme| resolve_build_setting(root, workspace, xcodeproj, scheme, "PRODUCT_BUNDLE_IDENTIFIER"));
    let team_id_dev = scheme_dev
        .as_deref()
        .and_then(|scheme| resolve_build_setting(root, workspace, xcodeproj, scheme, "DEVELOPMENT_TEAM"));
    let team_id_dis = scheme_dis
        .as_deref()
        .and_then(|scheme| resolve_build_setting(root, workspace, xcodeproj, scheme, "DEVELOPMENT_TEAM"));

    IdentityResult {
        bundle_id_dev,
        bundle_id_dis,
        team_id: team_id_dis.or(team_id_dev),
    }
}
