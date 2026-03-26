export type SigningStyle = "automatic" | "manual";
export type BootstrapMode = "standard" | "dryRun" | "configFile" | "interactive";

export interface ProjectConfig {
  projectPath: string;
  workspace: string;
  xcodeproj: string;
  schemeDev: string;
  schemeDis: string;
  bundleIdDev: string;
  bundleIdDis: string;
  teamId: string;
  profileDev: string;
  profileDis: string;
  signingStyle: SigningStyle;
  matchGitUrl: string;
  matchGitBranch: string;
  pgyerApiKey: string;
  appStoreConnectApiKeyPath: string;
  enableQualityGate: boolean;
  enableTests: boolean;
  enableSwiftlint: boolean;
  enableSlackNotify: boolean;
  enableWechatNotify: boolean;
  enableSnapshot: boolean;
  snapshotScheme: string;
  snapshotDevices: string;
  snapshotLanguages: string;
  metadataPath: string;
  enableMetadataUpload: boolean;
  enableScreenshotUpload: boolean;
  gymSkipClean: boolean;
  derivedDataPath: string;
  ciBundleInstall: boolean;
  ciCocoapodsDeployment: boolean;
  bootstrapMode: BootstrapMode;
  bootstrapConfigPath: string;
}

export interface ScanResult {
  projectName: string;
  workspace?: string;
  xcodeproj?: string;
  schemes: string[];
  bundleIdDev?: string;
  bundleIdDis?: string;
  teamId?: string;
}

export interface LaneRunResult {
  status: "success" | "failed";
  exitCode: number;
  output: string;
  lane: string;
}

export interface GeneratedFileStatus {
  path: string;
  exists: boolean;
  generated: boolean;
}

export interface GenerateResult {
  status: "success" | "failed";
  mode: string;
  runtimeEnvPath?: string;
  runtimeEnvWritten: boolean;
  files: GeneratedFileStatus[];
  stdout: string;
  stderr: string;
}

export interface IdentityResult {
  bundleIdDev?: string;
  bundleIdDis?: string;
  teamId?: string;
}

export interface DoctorCheck {
  name: string;
  status: "pass" | "warn" | "fail";
  detail: string;
  suggestion?: string;
}

export interface DoctorReport {
  checks: DoctorCheck[];
}
