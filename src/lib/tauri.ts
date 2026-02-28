import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  DoctorReport,
  IdentityResult,
  LaneRunResult,
  ProjectConfig,
  ScanResult
} from "../types";

export async function selectProjectPath(): Promise<string | null> {
  const selected = await open({
    directory: true,
    multiple: false,
    title: "Select iOS Project Folder"
  });
  return typeof selected === "string" ? selected : null;
}

export async function scanProject(projectPath: string): Promise<ScanResult> {
  return invoke("scan_project", { projectPath });
}

export async function doctorCheck(projectPath?: string): Promise<DoctorReport> {
  return invoke("doctor_check", { projectPath: projectPath || null });
}

export async function resolveIdentity(
  projectPath: string,
  workspace: string,
  xcodeproj: string,
  schemeDev: string,
  schemeDis: string
): Promise<IdentityResult> {
  return invoke("resolve_identity", {
    projectPath,
    workspace: workspace || null,
    xcodeproj: xcodeproj || null,
    schemeDev,
    schemeDis
  });
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
