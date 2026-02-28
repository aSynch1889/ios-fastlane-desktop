import { useMemo, useState } from "react";
import { defaultConfig } from "./lib/defaultConfig";
import { generateFastlaneFiles, runLane, scanProject } from "./lib/tauri";
import type { ProjectConfig, ScanResult } from "./types";

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

  async function onScan() {
    if (!config.projectPath.trim()) {
      setLog("Please input projectPath first.");
      return;
    }
    setBusy(true);
    try {
      const result = await scanProject(config.projectPath.trim());
      setScanResult(result);
      patch("workspace", result.workspace ?? "");
      patch("xcodeproj", result.xcodeproj ?? "");
      patch("schemeDev", result.schemes[0] ?? "");
      patch("schemeDis", result.schemes[0] ?? "");
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
            <input
              value={config.projectPath}
              onChange={(e) => patch("projectPath", e.target.value)}
              placeholder="/abs/path/to/iOS/project"
            />
          </label>
          <div className="inline">
            <button disabled={busy} onClick={onScan}>Scan</button>
            <button disabled={busy} onClick={onGenerate}>Generate Files</button>
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
            <input value={config.schemeDev} onChange={(e) => patch("schemeDev", e.target.value)} />
          </label>
          <label>
            Scheme Dis
            <input value={config.schemeDis} onChange={(e) => patch("schemeDis", e.target.value)} />
          </label>
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
