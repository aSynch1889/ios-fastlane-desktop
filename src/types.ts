export type SigningStyle = "automatic" | "manual";

export interface ProjectConfig {
  projectPath: string;
  workspace: string;
  xcodeproj: string;
  schemeDev: string;
  schemeDis: string;
  bundleIdDev: string;
  bundleIdDis: string;
  teamId: string;
  signingStyle: SigningStyle;
  matchGitUrl: string;
  matchGitBranch: string;
  pgyerApiKey: string;
  appStoreConnectApiKeyPath: string;
  enableQualityGate: boolean;
  enableTests: boolean;
  enableSwiftlint: boolean;
  enableSnapshot: boolean;
  metadataPath: string;
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
