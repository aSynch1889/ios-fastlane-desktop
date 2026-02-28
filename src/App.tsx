import { useMemo, useState } from "react";
import { defaultConfig } from "./lib/defaultConfig";
import {
  doctorCheck,
  generateFastlaneFiles,
  loadProfile,
  resolveIdentity,
  runLane,
  saveProfile,
  selectProjectPath,
  scanProject
} from "./lib/tauri";
import type { DoctorReport, ProjectConfig, ScanResult } from "./types";

const laneButtons = [
  "validate_config",
  "dev",
  "dis",
  "staging",
  "prod",
  "release_testflight",
  "release_appstore",
  "snapshot_capture",
  "metadata_sync",
  "ci_setup",
  "ci_build_dev",
  "ci_build_dis"
];

function App() {
  const [config, setConfig] = useState<ProjectConfig>(defaultConfig);
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [log, setLog] = useState("Ready. Fill project path and click Scan.");
  const [busy, setBusy] = useState(false);
  const [hideThirdPartySchemes, setHideThirdPartySchemes] = useState(true);
  const [mainScheme, setMainScheme] = useState("");
  const [identityDiff, setIdentityDiff] = useState<string[]>([]);
  const [doctorReport, setDoctorReport] = useState<DoctorReport | null>(null);

  const generatedPreview = useMemo(() => {
    return [
      "# Preview (config snapshot)",
      `projectPath=${config.projectPath}`,
      `workspace=${config.workspace}`,
      `xcodeproj=${config.xcodeproj}`,
      `schemeDev=${config.schemeDev}`,
      `schemeDis=${config.schemeDis}`,
      `bundleIdDev=${config.bundleIdDev}`,
      `bundleIdDis=${config.bundleIdDis}`,
      `teamId=${config.teamId || "<placeholder>"}`,
      `signingStyle=${config.signingStyle}`,
      `matchGitUrl=${config.matchGitUrl || "<empty>"}`,
      `enableQualityGate=${config.enableQualityGate}`,
      `enableTests=${config.enableTests}`,
      `enableSwiftlint=${config.enableSwiftlint}`,
      `enableSnapshot=${config.enableSnapshot}`
    ].join("\n");
  }, [config]);

  function patch<K extends keyof ProjectConfig>(key: K, value: ProjectConfig[K]) {
    setConfig((prev) => ({ ...prev, [key]: value }));
  }

  function pickSuggestedSchemes(schemes: string[]) {
    if (schemes.length === 0) {
      return { dev: "", dis: "" };
    }

    const dev = schemes.find((s) => /dev|debug|staging/i.test(s)) ?? schemes[0];
    const dis = schemes.find((s) => /prod|release|appstore/i.test(s)) ?? schemes.find((s) => s !== dev) ?? schemes[0];
    return { dev, dis };
  }

  function isThirdPartyScheme(scheme: string, projectName?: string) {
    const name = scheme.toLowerCase();
    const project = (projectName || "").toLowerCase();
    if (name.startsWith("pods-")) return true;
    if (name.includes("privacy")) return true;
    const thirdPartyKeywords = [
      "kingfisher",
      "snapkit",
      "swiftyjson",
      "adjust",
      "grdb",
      "mbprogresshud",
      "mjrefresh",
      "thinking",
      "jxpaging",
      "jxsegmented",
      "jxphoto"
    ];
    if (thirdPartyKeywords.some((k) => name.includes(k))) return true;
    if (project && name.includes(project)) return false;
    return false;
  }

  const availableSchemes = useMemo(() => {
    if (!scanResult?.schemes?.length) return [];
    if (!hideThirdPartySchemes) return scanResult.schemes;
    const filtered = scanResult.schemes.filter((scheme) => !isThirdPartyScheme(scheme, scanResult.projectName));
    return filtered.length ? filtered : scanResult.schemes;
  }, [hideThirdPartySchemes, scanResult]);

  function suggestMainScheme(schemes: string[], projectName?: string) {
    const exact = schemes.find((s) => s.toLowerCase() === (projectName || "").toLowerCase());
    if (exact) return exact;
    const contains = schemes.find((s) => projectName && s.toLowerCase().includes(projectName.toLowerCase()));
    if (contains) return contains;
    return schemes[0] || "";
  }

  async function onScan() {
    if (!config.projectPath.trim()) {
      setLog("Please input projectPath first.");
      return;
    }
    setBusy(true);
    try {
      const result = await scanProject(config.projectPath.trim());
      const suggested = pickSuggestedSchemes(result.schemes);
      setScanResult(result);
      const suggestedMain = suggestMainScheme(result.schemes, result.projectName);
      setMainScheme(suggestedMain);
      setIdentityDiff([]);
      patch("workspace", result.workspace ?? "");
      patch("xcodeproj", result.xcodeproj ?? "");
      patch("schemeDev", suggested.dev);
      patch("schemeDis", suggested.dis);
      patch("bundleIdDev", result.bundleIdDev ?? "");
      patch("bundleIdDis", result.bundleIdDis ?? "");
      patch("teamId", result.teamId ?? "");
      setLog(`Scan complete for ${result.projectName}.`);
    } catch (error) {
      setLog(`Scan failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onApplyIdentity() {
    if (!config.projectPath.trim()) {
      setLog("projectPath is required to resolve identity.");
      return;
    }
    if (!config.schemeDev.trim() || !config.schemeDis.trim()) {
      setLog("Please select schemeDev and schemeDis first.");
      return;
    }

    setBusy(true);
    try {
      const prevBundleDev = config.bundleIdDev;
      const prevBundleDis = config.bundleIdDis;
      const prevTeamId = config.teamId;
      const identity = await resolveIdentity(
        config.projectPath.trim(),
        config.workspace,
        config.xcodeproj,
        config.schemeDev,
        config.schemeDis
      );
      patch("bundleIdDev", identity.bundleIdDev ?? "");
      patch("bundleIdDis", identity.bundleIdDis ?? "");
      if (identity.teamId) {
        patch("teamId", identity.teamId);
      }
      const diff: string[] = [];
      if ((identity.bundleIdDev ?? "") !== prevBundleDev) {
        diff.push(`bundleIdDev: ${prevBundleDev || "-"} -> ${identity.bundleIdDev || "-"}`);
      }
      if ((identity.bundleIdDis ?? "") !== prevBundleDis) {
        diff.push(`bundleIdDis: ${prevBundleDis || "-"} -> ${identity.bundleIdDis || "-"}`);
      }
      if ((identity.teamId ?? prevTeamId) !== prevTeamId) {
        diff.push(`teamId: ${prevTeamId || "-"} -> ${identity.teamId || "-"}`);
      }
      setIdentityDiff(diff);
      setLog("Identity applied from selected schemes.");
    } catch (error) {
      setLog(`Resolve identity failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  function onLockMainScheme() {
    if (!mainScheme) {
      setLog("No main scheme available to lock.");
      return;
    }
    const devCandidate = availableSchemes.find((s) =>
      s !== mainScheme && /dev|debug|staging/i.test(s)
    );
    patch("schemeDis", mainScheme);
    patch("schemeDev", devCandidate ?? mainScheme);
    setLog(`Main scheme locked to ${mainScheme}.`);
  }

  async function onBrowseProjectPath() {
    setBusy(true);
    try {
      const selectedPath = await selectProjectPath();
      if (selectedPath) {
        patch("projectPath", selectedPath);
        setLog(`Selected project path: ${selectedPath}`);
      } else {
        setLog("Project path selection canceled.");
      }
    } catch (error) {
      setLog(`Open folder failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onGenerate() {
    setBusy(true);
    try {
      const output = await generateFastlaneFiles(config);
      setLog(output);
    } catch (error) {
      setLog(`Generate failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onSaveProfile() {
    if (!config.projectPath.trim()) {
      setLog("projectPath is required to save profile.");
      return;
    }
    setBusy(true);
    try {
      const output = await saveProfile(config);
      setLog(output);
    } catch (error) {
      setLog(`Save profile failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onLoadProfile() {
    if (!config.projectPath.trim()) {
      setLog("projectPath is required to load profile.");
      return;
    }
    setBusy(true);
    try {
      const loaded = await loadProfile(config.projectPath.trim());
      setConfig(loaded);
      setLog("Profile loaded.");
    } catch (error) {
      setLog(`Load profile failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onRunLane(lane: string) {
    if (!config.projectPath.trim()) {
      setLog("projectPath is required to run lane.");
      return;
    }
    setBusy(true);
    try {
      const result = await runLane(config.projectPath.trim(), lane);
      setLog(`[${result.status}] ${lane} (exit=${result.exitCode})\n\n${result.output}`);
    } catch (error) {
      setLog(`Lane run failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  async function onDoctorCheck() {
    setBusy(true);
    try {
      const report = await doctorCheck(config.projectPath.trim() || undefined);
      setDoctorReport(report);
      const passCount = report.checks.filter((c) => c.status === "pass").length;
      setLog(`Doctor completed: ${passCount}/${report.checks.length} checks passed.`);
    } catch (error) {
      setLog(`Doctor failed: ${String(error)}`);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="app-shell">
      <header>
        <h1>iOS Fastlane Desktop</h1>
        <p>Visual configure, generate, validate and run fastlane lanes.</p>
      </header>

      <main className="grid">
        <section className="panel">
          <h2>Project</h2>
          <label>
            Project Path
            <div className="path-picker">
              <input
                value={config.projectPath}
                onChange={(e) => patch("projectPath", e.target.value)}
                placeholder="/abs/path/to/iOS/project"
              />
              <button disabled={busy} onClick={onBrowseProjectPath} type="button">Browse</button>
            </div>
          </label>
          <div className="inline">
            <button disabled={busy} onClick={onScan}>Scan</button>
            <button disabled={busy} onClick={onGenerate}>Generate Files</button>
            <button disabled={busy} onClick={onSaveProfile}>Save Profile</button>
            <button disabled={busy} onClick={onLoadProfile}>Load Profile</button>
          </div>

          <h3>Build Identity</h3>
          <label>
            Workspace
            <input value={config.workspace} onChange={(e) => patch("workspace", e.target.value)} />
          </label>
          <label>
            Xcodeproj
            <input value={config.xcodeproj} onChange={(e) => patch("xcodeproj", e.target.value)} />
          </label>
          <label>
            Scheme Dev
            {availableSchemes.length ? (
              <select value={config.schemeDev} onChange={(e) => patch("schemeDev", e.target.value)}>
                {availableSchemes.map((scheme) => (
                  <option key={scheme} value={scheme}>{scheme}</option>
                ))}
              </select>
            ) : (
              <input value={config.schemeDev} onChange={(e) => patch("schemeDev", e.target.value)} />
            )}
          </label>
          <label>
            Scheme Dis
            {availableSchemes.length ? (
              <select value={config.schemeDis} onChange={(e) => patch("schemeDis", e.target.value)}>
                {availableSchemes.map((scheme) => (
                  <option key={scheme} value={scheme}>{scheme}</option>
                ))}
              </select>
            ) : (
              <input value={config.schemeDis} onChange={(e) => patch("schemeDis", e.target.value)} />
            )}
          </label>
          <div className="scheme-controls">
            <label className="inline-check">
              <input
                type="checkbox"
                checked={hideThirdPartySchemes}
                onChange={(e) => setHideThirdPartySchemes(e.target.checked)}
              />
              Hide third-party schemes
            </label>
            <label>
              Main Scheme
              <select value={mainScheme} onChange={(e) => setMainScheme(e.target.value)}>
                {(availableSchemes.length ? availableSchemes : scanResult?.schemes ?? []).map((scheme) => (
                  <option key={scheme} value={scheme}>{scheme}</option>
                ))}
              </select>
            </label>
            <button
              disabled={busy || !(availableSchemes.length || scanResult?.schemes.length)}
              onClick={onLockMainScheme}
            >
              Lock Main Scheme
            </button>
          </div>
          <div className="inline">
            <button disabled={busy || !(availableSchemes.length || scanResult?.schemes.length)} onClick={onApplyIdentity}>Apply Scheme Identity</button>
          </div>
          {identityDiff.length > 0 && (
            <div className="identity-diff">
              <strong>Identity Updates</strong>
              {identityDiff.map((line) => (
                <div key={line}>{line}</div>
              ))}
            </div>
          )}
          <label>
            Bundle ID Dev
            <input value={config.bundleIdDev} onChange={(e) => patch("bundleIdDev", e.target.value)} />
          </label>
          <label>
            Bundle ID Dis
            <input value={config.bundleIdDis} onChange={(e) => patch("bundleIdDis", e.target.value)} />
          </label>
          <label>
            Team ID
            <input value={config.teamId} onChange={(e) => patch("teamId", e.target.value)} placeholder="ABCD123456" />
          </label>
          <label>
            Signing Style
            <select value={config.signingStyle} onChange={(e) => patch("signingStyle", e.target.value as ProjectConfig["signingStyle"])}>
              <option value="automatic">automatic</option>
              <option value="manual">manual</option>
            </select>
          </label>

          <h3>Distribution and Quality</h3>
          <label>
            Match Git URL
            <input value={config.matchGitUrl} onChange={(e) => patch("matchGitUrl", e.target.value)} placeholder="git@github.com:org/certs.git" />
          </label>
          <label>
            Match Branch
            <input value={config.matchGitBranch} onChange={(e) => patch("matchGitBranch", e.target.value)} />
          </label>
          <label>
            Pgyer API Key
            <input value={config.pgyerApiKey} onChange={(e) => patch("pgyerApiKey", e.target.value)} />
          </label>
          <label>
            ASC API Key Path
            <input value={config.appStoreConnectApiKeyPath} onChange={(e) => patch("appStoreConnectApiKeyPath", e.target.value)} />
          </label>
          <label>
            Metadata Path
            <input value={config.metadataPath} onChange={(e) => patch("metadataPath", e.target.value)} />
          </label>

          <div className="checkboxes">
            <label><input type="checkbox" checked={config.enableQualityGate} onChange={(e) => patch("enableQualityGate", e.target.checked)} />Enable quality gate</label>
            <label><input type="checkbox" checked={config.enableTests} onChange={(e) => patch("enableTests", e.target.checked)} />Enable tests</label>
            <label><input type="checkbox" checked={config.enableSwiftlint} onChange={(e) => patch("enableSwiftlint", e.target.checked)} />Enable swiftlint</label>
            <label><input type="checkbox" checked={config.enableSnapshot} onChange={(e) => patch("enableSnapshot", e.target.checked)} />Enable snapshot</label>
          </div>
        </section>

        <section className="panel">
          <h2>Doctor</h2>
          <div className="inline">
            <button disabled={busy} onClick={onDoctorCheck}>Run Doctor</button>
          </div>
          {doctorReport && (
            <div className="doctor-list">
              {doctorReport.checks.map((check) => (
                <div key={check.name} className={`doctor-item doctor-${check.status}`}>
                  <div className="doctor-head">
                    <strong>{check.name}</strong>
                    <span>{check.status.toUpperCase()}</span>
                  </div>
                  <div>{check.detail}</div>
                  {check.suggestion && <div className="doctor-tip">Suggestion: {check.suggestion}</div>}
                </div>
              ))}
            </div>
          )}

          <h2>Preview</h2>
          <pre>{generatedPreview}</pre>

          <h2>Lane Runner</h2>
          <div className="lane-grid">
            {laneButtons.map((lane) => (
              <button key={lane} disabled={busy} onClick={() => onRunLane(lane)}>{lane}</button>
            ))}
          </div>

          <h2>Execution Log</h2>
          <pre className="log">{log}</pre>

          {scanResult && (
            <div className="scan-card">
              <strong>Detected:</strong>
              <div>project: {scanResult.projectName}</div>
              <div>workspace: {scanResult.workspace ?? "-"}</div>
              <div>xcodeproj: {scanResult.xcodeproj ?? "-"}</div>
              <div>schemes: {scanResult.schemes.join(", ") || "-"}</div>
            </div>
          )}
        </section>
      </main>
    </div>
  );
}

export default App;
