import type { ProjectConfig } from "../types";

export const defaultConfig: ProjectConfig = {
  projectPath: "",
  workspace: "",
  xcodeproj: "",
  schemeDev: "",
  schemeDis: "",
  bundleIdDev: "",
  bundleIdDis: "",
  teamId: "",
  signingStyle: "automatic",
  matchGitUrl: "",
  matchGitBranch: "main",
  pgyerApiKey: "",
  appStoreConnectApiKeyPath: "",
  enableQualityGate: true,
  enableTests: true,
  enableSwiftlint: false,
  enableSnapshot: false,
  metadataPath: "fastlane/metadata"
};
