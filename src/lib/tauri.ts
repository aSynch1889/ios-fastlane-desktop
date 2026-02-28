import { invoke } from "@tauri-apps/api/core";
import type { LaneRunResult, ProjectConfig, ScanResult } from "../types";

export async function scanProject(projectPath: string): Promise<ScanResult> {
  return invoke("scan_project", { projectPath });
}

export async function generateFastlaneFiles(config: ProjectConfig): Promise<string> {
  return invoke("generate_fastlane_files", { config });
}

export async function runLane(projectPath: string, lane: string): Promise<LaneRunResult> {
  return invoke("run_lane", { projectPath, lane });
}

export async function saveProfile(config: ProjectConfig): Promise<string> {
  return invoke("save_profile", { config });
}

export async function loadProfile(projectPath: string): Promise<ProjectConfig> {
  return invoke("load_profile", { projectPath });
}
