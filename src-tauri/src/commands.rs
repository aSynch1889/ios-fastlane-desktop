use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct ProjectConfig {
    pub project_path: String,
    pub workspace: String,
    pub xcodeproj: String,
    pub scheme_dev: String,
    pub scheme_dis: String,
    pub bundle_id_dev: String,
    pub bundle_id_dis: String,
    pub team_id: String,
    pub profile_dev: String,
    pub profile_dis: String,
    pub signing_style: String,
    pub match_git_url: String,
    pub match_git_branch: String,
    pub pgyer_api_key: String,
    pub app_store_connect_api_key_path: String,
    pub enable_quality_gate: bool,
    pub enable_tests: bool,
    pub enable_swiftlint: bool,
    pub enable_slack_notify: bool,
    pub enable_wechat_notify: bool,
    pub enable_snapshot: bool,
    pub snapshot_scheme: String,
    pub snapshot_devices: String,
    pub snapshot_languages: String,
    pub metadata_path: String,
    pub enable_metadata_upload: bool,
    pub enable_screenshot_upload: bool,
    pub gym_skip_clean: bool,
    pub derived_data_path: String,
    pub ci_bundle_install: bool,
    pub ci_cocoapods_deployment: bool,
    pub bootstrap_mode: String,
    pub bootstrap_config_path: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_path: String::new(),
            workspace: String::new(),
            xcodeproj: String::new(),
            scheme_dev: String::new(),
            scheme_dis: String::new(),
            bundle_id_dev: String::new(),
            bundle_id_dis: String::new(),
            team_id: String::new(),
            profile_dev: String::new(),
            profile_dis: String::new(),
            signing_style: "automatic".to_string(),
            match_git_url: String::new(),
            match_git_branch: "main".to_string(),
            pgyer_api_key: String::new(),
            app_store_connect_api_key_path: String::new(),
            enable_quality_gate: true,
            enable_tests: true,
            enable_swiftlint: false,
            enable_slack_notify: false,
            enable_wechat_notify: false,
            enable_snapshot: false,
            snapshot_scheme: String::new(),
            snapshot_devices: String::new(),
            snapshot_languages: String::new(),
            metadata_path: "fastlane/metadata".to_string(),
            enable_metadata_upload: false,
            enable_screenshot_upload: false,
            gym_skip_clean: false,
            derived_data_path: String::new(),
            ci_bundle_install: true,
            ci_cocoapods_deployment: true,
            bootstrap_mode: "standard".to_string(),
            bootstrap_config_path: String::new(),
        }
    }
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
pub struct GeneratedFileStatus {
    pub path: String,
    pub exists: bool,
    pub generated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateResult {
    pub status: String,
    pub mode: String,
    pub runtime_env_path: Option<String>,
    pub runtime_env_written: bool,
    pub files: Vec<GeneratedFileStatus>,
    pub stdout: String,
    pub stderr: String,
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
            "Ruby Compatibility",
            "/bin/zsh",
            &[
                "-lc",
                "ruby -e 'v=RUBY_VERSION.split(\".\").map(&:to_i); abort(\"Ruby >= 4.0 is not supported by this fastlane setup\") if v[0] >= 4; puts \"Ruby #{RUBY_VERSION} compatible\"'",
            ],
            Some("Use Ruby 3.1~3.3 via rbenv/rvm/asdf, then run with `bundle exec fastlane ...`."),
        ),
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
    let input_path = PathBuf::from(project_path.clone());
    if !input_path.exists() {
        return Err(format!("Project path not found: {}", project_path));
    }
    let root = normalize_project_root(&input_path);
    let (container_workspace, container_xcodeproj) = container_hint_from_input_path(&input_path);

    let workspace = container_workspace.or_else(|| find_first_with_ext(&root, "xcworkspace"));
    let xcodeproj = container_xcodeproj.or_else(|| find_first_with_ext(&root, "xcodeproj"));

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
    let input_path = PathBuf::from(&project_path);
    if !input_path.exists() {
        return Err(format!("Project path not found: {}", project_path));
    }
    let root = normalize_project_root(&input_path);
    let (container_workspace, container_xcodeproj) = container_hint_from_input_path(&input_path);

    let resolved_workspace = workspace
        .filter(|v| !v.trim().is_empty())
        .or(container_workspace)
        .or_else(|| find_first_with_ext(&root, "xcworkspace"));
    let resolved_xcodeproj = xcodeproj
        .filter(|v| !v.trim().is_empty())
        .or(container_xcodeproj)
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
    let project_root = normalize_project_root(&PathBuf::from(&config.project_path));
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
    let root = normalize_project_root(&PathBuf::from(&project_path));
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
pub fn generate_fastlane_files(config: ProjectConfig) -> Result<GenerateResult, String> {
    let input_path = PathBuf::from(&config.project_path);
    let project_root = normalize_project_root(&input_path);
    if !project_root.exists() {
        return Err(format!("projectPath does not exist: {}", config.project_path));
    }
    let (container_workspace, container_xcodeproj) = container_hint_from_input_path(&input_path);

    let skill_script = PathBuf::from("/Users/newdroid/.codex/skills/ios-fastlane-skill/scripts/bootstrap_fastlane.sh");
    if !skill_script.exists() {
        return Err(format!("Skill bootstrap script not found: {}", skill_script.display()));
    }

    let project_name = project_root
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("iOSProject")
        .to_string();
    let resolved_workspace = if !config.workspace.trim().is_empty() {
        to_absolute_from_project(&project_root, config.workspace.trim())
    } else if let Some(ws) = container_workspace {
        to_absolute_from_project(&project_root, &ws)
    } else {
        to_absolute_from_project(
            &project_root,
            &find_first_with_ext(&project_root, "xcworkspace").unwrap_or_default(),
        )
    };
    let resolved_xcodeproj = if !config.xcodeproj.trim().is_empty() {
        to_absolute_from_project(&project_root, config.xcodeproj.trim())
    } else if let Some(proj) = container_xcodeproj {
        to_absolute_from_project(&project_root, &proj)
    } else {
        to_absolute_from_project(
            &project_root,
            &find_first_with_ext(&project_root, "xcodeproj").unwrap_or_default(),
        )
    };

    if resolved_workspace.is_empty() && resolved_xcodeproj.is_empty() {
        return Err(
            "No .xcworkspace/.xcodeproj found under projectPath. Please set Workspace or Xcodeproj manually."
                .to_string(),
        );
    }

    let mut cmd = Command::new("bash");
    cmd.arg(skill_script);
    cmd.current_dir(&project_root);
    cmd.arg("--project-name").arg(project_name);
    let mode = normalize_bootstrap_mode(&config.bootstrap_mode)?;
    if mode == "configFile" {
        if config.bootstrap_config_path.trim().is_empty() {
            return Err("bootstrapConfigPath is required when bootstrapMode=configFile".to_string());
        }
        cmd.arg("--config").arg(config.bootstrap_config_path.trim());
    }
    push_optional_arg(&mut cmd, "--workspace", &resolved_workspace);
    push_optional_arg(&mut cmd, "--xcodeproj", &resolved_xcodeproj);
    push_optional_arg(&mut cmd, "--scheme-dev", &config.scheme_dev);
    push_optional_arg(&mut cmd, "--scheme-dis", &config.scheme_dis);
    push_optional_arg(&mut cmd, "--bundle-id-dev", &config.bundle_id_dev);
    push_optional_arg(&mut cmd, "--bundle-id-dis", &config.bundle_id_dis);
    push_optional_arg(&mut cmd, "--team-id", &config.team_id);
    push_optional_arg(&mut cmd, "--profile-dev", &config.profile_dev);
    push_optional_arg(&mut cmd, "--profile-dis", &config.profile_dis);
    push_optional_arg(&mut cmd, "--signing-style", &config.signing_style);
    push_optional_arg(&mut cmd, "--match-git-url", &config.match_git_url);
    push_optional_arg(&mut cmd, "--match-git-branch", &config.match_git_branch);
    cmd.arg("--enable-quality-gate")
        .arg(bool_to_string(config.enable_quality_gate));
    cmd.arg("--enable-tests")
        .arg(bool_to_string(config.enable_tests));
    cmd.arg("--enable-swiftlint")
        .arg(bool_to_string(config.enable_swiftlint));
    cmd.arg("--enable-slack-notify")
        .arg(bool_to_string(config.enable_slack_notify));
    cmd.arg("--enable-wechat-notify")
        .arg(bool_to_string(config.enable_wechat_notify));
    cmd.arg("--enable-snapshot")
        .arg(bool_to_string(config.enable_snapshot));
    push_optional_arg(&mut cmd, "--snapshot-scheme", &config.snapshot_scheme);
    push_optional_arg(&mut cmd, "--snapshot-devices", &config.snapshot_devices);
    push_optional_arg(&mut cmd, "--snapshot-languages", &config.snapshot_languages);
    push_optional_arg(&mut cmd, "--metadata-path", &config.metadata_path);
    cmd.arg("--enable-metadata-upload")
        .arg(bool_to_string(config.enable_metadata_upload));
    cmd.arg("--enable-screenshot-upload")
        .arg(bool_to_string(config.enable_screenshot_upload));
    cmd.arg("--gym-skip-clean")
        .arg(bool_to_string(config.gym_skip_clean));
    push_optional_arg(&mut cmd, "--derived-data-path", &config.derived_data_path);
    cmd.arg("--ci-bundle-install")
        .arg(bool_to_string(config.ci_bundle_install));
    cmd.arg("--ci-cocoapods-deployment")
        .arg(bool_to_string(config.ci_cocoapods_deployment));
    if mode == "dryRun" {
        cmd.arg("--dry-run");
    }
    if mode == "interactive" {
        cmd.arg("--interactive");
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Run skill bootstrap failed: {}", e))?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        return Ok(GenerateResult {
            status: "failed".to_string(),
            mode: mode.to_string(),
            runtime_env_path: None,
            runtime_env_written: false,
            files: expected_generated_files(&project_root, &[]),
            stdout: stdout.to_string(),
            stderr: stderr.to_string(),
        });
    }

    let runtime_env_path = project_root.join("fastlane").join(".env.fastlane");
    let mut runtime_env_written = false;
    if mode != "dryRun" {
        fs::write(&runtime_env_path, render_runtime_env(&config))
            .map_err(|e| format!("Write runtime env failed: {}", e))?;
        ensure_fastlane_plugin_gemfile(&project_root)?;
        patch_generated_doctor_script_for_bash3(&project_root)?;
        runtime_env_written = true;
    }

    let generated_from_stdout = parse_generated_paths(&project_root, &stdout);
    let files = expected_generated_files(&project_root, &generated_from_stdout);

    Ok(GenerateResult {
        status: "success".to_string(),
        mode: mode.to_string(),
        runtime_env_path: if mode == "dryRun" {
            None
        } else {
            Some(runtime_env_path.display().to_string())
        },
        runtime_env_written,
        files,
        stdout: stdout.to_string(),
        stderr: stderr.to_string(),
    })
}

#[tauri::command]
pub fn bundle_install_and_validate(project_path: String) -> Result<LaneRunResult, String> {
    let normalized_project_root = normalize_project_root(&PathBuf::from(&project_path));
    ensure_fastlane_plugin_gemfile(&normalized_project_root)?;
    let normalized_project_path = normalized_project_root.to_string_lossy().to_string();
    let output = Command::new("/bin/zsh")
        .arg("-lc")
        .arg(ruby_aware_shell_command(
            &normalized_project_path,
            "FASTLANE_SKIP_UPDATE_CHECK=1 FASTLANE_DISABLE_COLORS=1 CI=1 bundle install && FASTLANE_SKIP_UPDATE_CHECK=1 FASTLANE_DISABLE_COLORS=1 CI=1 bundle exec fastlane ios validate_config",
        ))
        .output()
        .map_err(|e| format!("Failed to run bundle install + validate_config: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);
    let status = if output.status.success() { "success" } else { "failed" };

    Ok(LaneRunResult {
        status: status.to_string(),
        exit_code,
        output: format!("{}\n{}", stdout, stderr),
        lane: "bundle_install_and_validate".to_string(),
    })
}

#[tauri::command]
pub fn run_lane(project_path: String, lane: String) -> Result<LaneRunResult, String> {
    let normalized_project_path = normalize_project_root(&PathBuf::from(&project_path))
        .to_string_lossy()
        .to_string();

    if lane_requires_project_container(&lane) {
        // Force-sync WORKSPACE/XCODEPROJ in Fastfile to avoid stale or invalid
        // values from previous generations or incorrect UI states.
        sync_fastfile_container_config(&normalized_project_path)?;
        validate_fastfile_container_config(&normalized_project_path, &lane)?;
    }

    let (test_override, test_override_note) =
        detect_test_action_and_maybe_disable_tests(&normalized_project_path, &lane);
    let preflight = lane_preflight_report(
        &normalized_project_path,
        test_override_note.as_deref(),
    );
    let mut env_prefix = String::from("FASTLANE_SKIP_UPDATE_CHECK=1 FASTLANE_DISABLE_COLORS=1 CI=1");
    if test_override {
        env_prefix.push_str(" ENABLE_TESTS=false");
    }

    let mut output = run_lane_shell(&normalized_project_path, &lane, &env_prefix)?;
    let mut retry_note = String::new();

    if !output.status.success()
        && lane_runs_quality_gate(&lane)
        && !test_override
        && lane_failed_for_missing_test_action(&output)
    {
        let retry_env_prefix = format!("{env_prefix} ENABLE_TESTS=false");
        output = run_lane_shell(&normalized_project_path, &lane, &retry_env_prefix)?;
        retry_note = "[preflight] Retry with ENABLE_TESTS=false due to missing test action in scheme.\n".to_string();
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);
    let status = if output.status.success() { "success" } else { "failed" };
    let command_output = format!("{}\n{}", stdout, stderr);
    let full_output = if retry_note.is_empty() {
        format!("{}\n{}", preflight, command_output)
    } else {
        format!("{}\n{}\n{}", preflight, retry_note.trim_end(), command_output)
    };

    Ok(LaneRunResult {
        status: status.to_string(),
        exit_code,
        output: full_output,
        lane,
    })
}

fn run_lane_shell(project_path: &str, lane: &str, env_prefix: &str) -> Result<std::process::Output, String> {
    let lane_escaped = escape_single_quote(lane);
    let body = format!("{} bundle exec fastlane ios '{}'", env_prefix, lane_escaped);
    Command::new("/bin/zsh")
        .arg("-lc")
        .arg(ruby_aware_shell_command(project_path, &body))
        .output()
        .map_err(|e| format!("Failed to run lane: {}", e))
}

fn ruby_aware_shell_command(project_path: &str, body: &str) -> String {
    let project = escape_single_quote(project_path);
    let body_escaped = body.replace('\'', "'\\''");
    format!(
        "cd '{}' && RUBY_VER=$(tr -d '[:space:]' < .ruby-version 2>/dev/null || true) && \
if command -v rbenv >/dev/null 2>&1; then \
  eval \"$(rbenv init - zsh)\" >/dev/null 2>&1 || true; \
  if [ -n \"$RUBY_VER\" ]; then rbenv shell \"$RUBY_VER\" >/dev/null 2>&1 || true; fi; \
  /bin/zsh -lc '{}'; \
elif [ -x \"$HOME/.rvm/bin/rvm\" ] && [ -n \"$RUBY_VER\" ]; then \
  \"$HOME/.rvm/bin/rvm\" \"$RUBY_VER\" do /bin/zsh -lc '{}'; \
elif command -v asdf >/dev/null 2>&1 && [ -n \"$RUBY_VER\" ]; then \
  asdf shell ruby \"$RUBY_VER\" >/dev/null 2>&1 || true; \
  /bin/zsh -lc '{}'; \
else \
  /bin/zsh -lc '{}'; \
fi",
        project, body_escaped, body_escaped, body_escaped, body_escaped
    )
}

fn lane_failed_for_missing_test_action(output: &std::process::Output) -> bool {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let merged = format!("{stdout}\n{stderr}").to_lowercase();
    merged.contains("not currently configured for the test action")
        || merged.contains("error building/testing the application")
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
            // Ignore internal workspace metadata located inside *.xcodeproj bundles,
            // e.g. Foo.xcodeproj/project.xcworkspace.
            if ext == "xcworkspace" && is_inside_xcodeproj_bundle(root, path) {
                continue;
            }
            return path
                .strip_prefix(root)
                .ok()
                .map(|p| p.to_string_lossy().to_string());
        }
    }
    None
}

fn is_inside_xcodeproj_bundle(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .ok()
        .map(|rel| {
            rel.components().any(|component| {
                component
                    .as_os_str()
                    .to_str()
                    .map(|s| s.ends_with(".xcodeproj"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

fn normalize_project_root(path: &Path) -> PathBuf {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        if ext == "xcodeproj" || ext == "xcworkspace" {
            return path
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| path.to_path_buf());
        }
    }
    path.to_path_buf()
}

fn container_hint_from_input_path(path: &Path) -> (Option<String>, Option<String>) {
    let file_name = path.file_name().and_then(OsStr::to_str).map(|s| s.to_string());
    match path.extension().and_then(OsStr::to_str) {
        Some("xcworkspace") => (file_name, None),
        Some("xcodeproj") => (None, file_name),
        _ => (None, None),
    }
}

fn escape_single_quote(value: &str) -> String {
    value.replace('\'', "'\\''")
}

fn push_optional_arg(cmd: &mut Command, name: &str, value: &str) {
    if !value.trim().is_empty() {
        cmd.arg(name).arg(value.trim());
    }
}

fn bool_to_string(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn ensure_fastlane_plugin_gemfile(project_root: &Path) -> Result<(), String> {
    let pluginfile = project_root.join("fastlane").join("Pluginfile");
    if !pluginfile.exists() {
        return Ok(());
    }

    let gemfile = project_root.join("Gemfile");
    let mut content = if gemfile.exists() {
        fs::read_to_string(&gemfile).map_err(|e| format!("Read Gemfile failed: {}", e))?
    } else {
        "source \"https://rubygems.org\"\n\ngem \"fastlane\"\n".to_string()
    };

    if content.contains("eval_gemfile(plugins_path)") {
        return Ok(());
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("\nplugins_path = File.join(File.dirname(__FILE__), \"fastlane\", \"Pluginfile\")\n");
    content.push_str("eval_gemfile(plugins_path) if File.exist?(plugins_path)\n");

    fs::write(&gemfile, content).map_err(|e| format!("Write Gemfile failed: {}", e))?;
    Ok(())
}

fn patch_generated_doctor_script_for_bash3(project_root: &Path) -> Result<(), String> {
    let doctor_script = project_root
        .join("scripts")
        .join("doctor_fastlane_env.sh");
    if !doctor_script.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&doctor_script)
        .map_err(|e| format!("Read doctor script failed: {}", e))?;
    if !content.contains("${IS_CI,,}") {
        return Ok(());
    }

    let patched = content.replace(
        "${IS_CI,,}",
        "$(printf '%s' \"$IS_CI\" | tr '[:upper:]' '[:lower:]')",
    );
    fs::write(&doctor_script, patched)
        .map_err(|e| format!("Patch doctor script compatibility failed: {}", e))?;
    Ok(())
}

fn normalize_bootstrap_mode(raw: &str) -> Result<&'static str, String> {
    match raw.trim() {
        "" | "standard" => Ok("standard"),
        "dryRun" => Ok("dryRun"),
        "configFile" => Ok("configFile"),
        "interactive" => Ok("interactive"),
        value => Err(format!("Unsupported bootstrapMode: {}", value)),
    }
}

fn render_runtime_env(config: &ProjectConfig) -> String {
    [
        format!("PGYER_API_KEY={}", config.pgyer_api_key),
        format!("MATCH_GIT_URL={}", config.match_git_url),
        format!("MATCH_GIT_BRANCH={}", config.match_git_branch),
        format!(
            "APP_STORE_CONNECT_API_KEY_PATH={}",
            config.app_store_connect_api_key_path
        ),
        format!("ENABLE_QUALITY_GATE={}", bool_to_string(config.enable_quality_gate)),
        format!("ENABLE_TESTS={}", bool_to_string(config.enable_tests)),
        format!("ENABLE_SWIFTLINT={}", bool_to_string(config.enable_swiftlint)),
        format!(
            "ENABLE_SLACK_NOTIFY={}",
            bool_to_string(config.enable_slack_notify)
        ),
        format!(
            "ENABLE_WECHAT_NOTIFY={}",
            bool_to_string(config.enable_wechat_notify)
        ),
        format!("ENABLE_SNAPSHOT={}", bool_to_string(config.enable_snapshot)),
        format!("SNAPSHOT_SCHEME={}", config.snapshot_scheme),
        format!("SNAPSHOT_DEVICES={}", config.snapshot_devices),
        format!("SNAPSHOT_LANGUAGES={}", config.snapshot_languages),
        format!("METADATA_PATH={}", config.metadata_path),
        format!(
            "ENABLE_METADATA_UPLOAD={}",
            bool_to_string(config.enable_metadata_upload)
        ),
        format!(
            "ENABLE_SCREENSHOT_UPLOAD={}",
            bool_to_string(config.enable_screenshot_upload)
        ),
        format!("GYM_SKIP_CLEAN={}", bool_to_string(config.gym_skip_clean)),
        format!("DERIVED_DATA_PATH={}", config.derived_data_path),
        format!("CI_BUNDLE_INSTALL={}", bool_to_string(config.ci_bundle_install)),
        format!(
            "CI_COCOAPODS_DEPLOYMENT={}",
            bool_to_string(config.ci_cocoapods_deployment)
        ),
    ]
    .join("\n")
}

fn parse_generated_paths(project_root: &Path, stdout: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if let Some(path) = trimmed.strip_prefix("Generated: ") {
            let path_value = path.trim().to_string();
            if !path_value.is_empty() {
                paths.push(path_value);
            }
        }
    }

    // If script output format changes, fall back to expected output paths.
    if paths.is_empty() {
        let fastlane_dir = project_root.join("fastlane");
        paths.push(fastlane_dir.join("Fastfile").display().to_string());
        paths.push(fastlane_dir.join("Appfile").display().to_string());
        paths.push(fastlane_dir.join("Pluginfile").display().to_string());
        paths.push(fastlane_dir.join(".env.fastlane.example").display().to_string());
        paths.push(fastlane_dir.join(".env.fastlane.staging.example").display().to_string());
        paths.push(fastlane_dir.join(".env.fastlane.prod.example").display().to_string());
        paths.push(project_root.join("Gemfile").display().to_string());
        paths.push(
            project_root
                .join("scripts")
                .join("doctor_fastlane_env.sh")
                .display()
                .to_string(),
        );
        paths.push(
            project_root
                .join("scripts")
                .join("fastlane_run.sh")
                .display()
                .to_string(),
        );
    }

    paths
}

fn expected_generated_files(project_root: &Path, generated_paths: &[String]) -> Vec<GeneratedFileStatus> {
    let fastlane_dir = project_root.join("fastlane");
    let expected = vec![
        fastlane_dir.join("Fastfile").display().to_string(),
        fastlane_dir.join("Appfile").display().to_string(),
        fastlane_dir.join("Pluginfile").display().to_string(),
        fastlane_dir.join(".env.fastlane.example").display().to_string(),
        fastlane_dir.join(".env.fastlane.staging.example").display().to_string(),
        fastlane_dir.join(".env.fastlane.prod.example").display().to_string(),
        project_root.join("Gemfile").display().to_string(),
        project_root
            .join("scripts")
            .join("doctor_fastlane_env.sh")
            .display()
            .to_string(),
        project_root
            .join("scripts")
            .join("fastlane_run.sh")
            .display()
            .to_string(),
    ];

    expected
        .into_iter()
        .map(|path| GeneratedFileStatus {
            exists: PathBuf::from(&path).exists(),
            generated: generated_paths.iter().any(|p| p == &path),
            path,
        })
        .collect()
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

fn validate_fastfile_container_config(project_path: &str, lane: &str) -> Result<(), String> {
    if !lane_requires_project_container(lane) {
        return Ok(());
    }

    let fastfile = PathBuf::from(project_path).join("fastlane").join("Fastfile");
    if !fastfile.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&fastfile)
        .map_err(|e| format!("Read Fastfile failed: {}", e))?;

    let workspace = extract_fastfile_value(&content, "WORKSPACE").unwrap_or_default();
    let xcodeproj = extract_fastfile_value(&content, "XCODEPROJ").unwrap_or_default();
    let workspace_ok = path_exists_from_project(project_path, &workspace);
    let xcodeproj_ok = path_exists_from_project(project_path, &xcodeproj);

    if !workspace_ok && !xcodeproj_ok {
        return Err(format!(
            "Fastfile container paths are invalid. WORKSPACE='{}' exists={} | XCODEPROJ='{}' exists={}. Please click Scan, confirm workspace/xcodeproj, then Generate Files again.",
            workspace,
            workspace_ok,
            xcodeproj,
            xcodeproj_ok
        ));
    }
    Ok(())
}

fn lane_requires_project_container(lane: &str) -> bool {
    matches!(
        lane,
        "dev"
            | "dis"
            | "staging"
            | "prod"
            | "release_testflight"
            | "release_appstore"
            | "ci_build_dev"
            | "ci_build_dis"
            | "snapshot_capture"
    )
}

fn extract_fastfile_value(content: &str, key: &str) -> Option<String> {
    let prefix = format!("{} = \"", key);
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix(&prefix) {
            if let Some(end_idx) = rest.find('"') {
                return Some(rest[..end_idx].to_string());
            }
        }
    }
    None
}

fn path_exists_from_project(project_path: &str, path_value: &str) -> bool {
    let trimmed = path_value.trim();
    if trimmed.is_empty() {
        return false;
    }

    let candidate = PathBuf::from(trimmed);
    if candidate.is_absolute() {
        return candidate.exists();
    }

    PathBuf::from(project_path).join(candidate).exists()
}

fn sync_fastfile_container_config(project_path: &str) -> Result<(), String> {
    let project_root = PathBuf::from(project_path);
    let fastfile = project_root.join("fastlane").join("Fastfile");
    if !fastfile.exists() {
        return Ok(());
    }

    let detected_workspace = to_absolute_from_project(
        &project_root,
        &find_first_with_ext(&project_root, "xcworkspace").unwrap_or_default(),
    );
    let detected_xcodeproj = to_absolute_from_project(
        &project_root,
        &find_first_with_ext(&project_root, "xcodeproj").unwrap_or_default(),
    );

    let content = fs::read_to_string(&fastfile)
        .map_err(|e| format!("Read Fastfile failed: {}", e))?;

    let mut changed = false;
    let mut new_lines = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let indent_len = line.len() - trimmed.len();
        let indent = &line[..indent_len];
        if trimmed.starts_with("WORKSPACE = ") {
            new_lines.push(format!("{indent}WORKSPACE = \"{}\"", detected_workspace));
            changed = true;
            continue;
        }
        if trimmed.starts_with("XCODEPROJ = ") {
            new_lines.push(format!("{indent}XCODEPROJ = \"{}\"", detected_xcodeproj));
            changed = true;
            continue;
        }
        new_lines.push(line.to_string());
    }

    if changed {
        let mut new_content = new_lines.join("\n");
        if content.ends_with('\n') {
            new_content.push('\n');
        }
        fs::write(&fastfile, new_content)
            .map_err(|e| format!("Write Fastfile sync failed: {}", e))?;
    }
    Ok(())
}

fn lane_preflight_report(project_path: &str, extra_note: Option<&str>) -> String {
    let mut lines = Vec::new();
    lines.push(format!("[preflight] cwd={}", project_path));

    let fastfile = PathBuf::from(project_path).join("fastlane").join("Fastfile");
    if !fastfile.exists() {
        lines.push("[preflight] fastfile_exists=false".to_string());
        if let Some(note) = extra_note {
            lines.push(format!("[preflight] {}", note));
        }
        return lines.join("\n");
    }

    match fs::read_to_string(&fastfile) {
        Ok(content) => {
            let ws = extract_fastfile_value(&content, "WORKSPACE").unwrap_or_default();
            let xp = extract_fastfile_value(&content, "XCODEPROJ").unwrap_or_default();
            let ws_exists = path_exists_from_project(project_path, &ws);
            let xp_exists = path_exists_from_project(project_path, &xp);
            lines.push(format!("[preflight] WORKSPACE='{}' exists={}", ws, ws_exists));
            lines.push(format!("[preflight] XCODEPROJ='{}' exists={}", xp, xp_exists));
        }
        Err(err) => lines.push(format!("[preflight] fastfile_read_error={}", err)),
    }

    if let Some(note) = extra_note {
        lines.push(format!("[preflight] {}", note));
    }
    lines.join("\n")
}

fn detect_test_action_and_maybe_disable_tests(project_path: &str, lane: &str) -> (bool, Option<String>) {
    if !lane_runs_quality_gate(lane) {
        return (false, None);
    }

    let fastfile = PathBuf::from(project_path).join("fastlane").join("Fastfile");
    let content = match fs::read_to_string(&fastfile) {
        Ok(v) => v,
        Err(_) => return (false, None),
    };
    let workspace = extract_fastfile_value(&content, "WORKSPACE").unwrap_or_default();
    let xcodeproj = extract_fastfile_value(&content, "XCODEPROJ").unwrap_or_default();
    let scheme_dev = extract_fastfile_value(&content, "SCHEME_DEV").unwrap_or_default();
    let scheme_dis = extract_fastfile_value(&content, "SCHEME_DIS").unwrap_or_default();

    let scheme = match lane {
        "dev" | "ci_build_dev" => scheme_dev,
        "dis" | "staging" | "prod" | "release_testflight" | "release_appstore" | "ci_build_dis" => scheme_dis,
        _ => scheme_dev,
    };
    if scheme.trim().is_empty() {
        return (false, None);
    }

    let container_arg = if path_exists_from_project(project_path, &workspace) {
        format!("-workspace '{}'", escape_single_quote(&workspace))
    } else if path_exists_from_project(project_path, &xcodeproj) {
        format!("-project '{}'", escape_single_quote(&xcodeproj))
    } else {
        return (false, None);
    };

    let cmd = format!(
        "cd '{}' && xcodebuild -showTestPlans {} -scheme '{}' >/dev/null 2>&1",
        escape_single_quote(project_path),
        container_arg,
        escape_single_quote(&scheme)
    );
    let ok = Command::new("/bin/zsh")
        .arg("-lc")
        .arg(cmd)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if ok {
        (false, None)
    } else {
        (
            true,
            Some(format!(
                "Detected scheme '{}' without test action. Auto override ENABLE_TESTS=false for this run.",
                scheme
            )),
        )
    }
}

fn lane_runs_quality_gate(lane: &str) -> bool {
    matches!(
        lane,
        "dev"
            | "dis"
            | "staging"
            | "prod"
            | "release_testflight"
            | "release_appstore"
            | "ci_build_dev"
            | "ci_build_dis"
    )
}

fn to_absolute_from_project(project_root: &Path, value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let path = PathBuf::from(trimmed);
    if path.is_absolute() {
        return path.to_string_lossy().to_string();
    }

    project_root.join(path).to_string_lossy().to_string()
}
