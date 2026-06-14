import { describe, expect, it, vi, beforeEach } from "vitest";

const { checkMock, askMock, messageMock, relaunchMock } = vi.hoisted(() => ({
  checkMock: vi.fn(),
  askMock: vi.fn(),
  messageMock: vi.fn(),
  relaunchMock: vi.fn(),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: checkMock,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: askMock,
  message: messageMock,
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: relaunchMock,
}));

import { invoke } from "@tauri-apps/api/core";
import { checkForAppUpdates, fetchLatestReleaseVersion, isNewerVersion } from "./updater";

describe("isNewerVersion", () => {
  it("compares semver tuples", () => {
    expect(isNewerVersion("1.0.1", "1.0.0")).toBe(true);
    expect(isNewerVersion("1.0.0", "1.0.0")).toBe(false);
    expect(isNewerVersion("1.0.0", "1.0.1")).toBe(false);
    expect(isNewerVersion("2.0.0", "1.9.9")).toBe(true);
  });

  it("compares CalVer against legacy SemVer", () => {
    expect(isNewerVersion("2026.6.14", "1.0.1")).toBe(true);
    expect(isNewerVersion("2026.6.15", "2026.6.14")).toBe(true);
    expect(isNewerVersion("2026.6.14", "2026.6.15")).toBe(false);
  });
});

describe("fetchLatestReleaseVersion", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("returns version from updater manifest command", async () => {
    vi.mocked(invoke).mockResolvedValue("1.0.1");

    await expect(fetchLatestReleaseVersion()).resolves.toBe("1.0.1");
    expect(invoke).toHaveBeenCalledWith("fetch_latest_release_version");
  });

  it("returns null when command fails", async () => {
    vi.mocked(invoke).mockRejectedValue(new Error("network"));

    await expect(fetchLatestReleaseVersion()).resolves.toBeNull();
  });
});

describe("checkForAppUpdates", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("shows message when manual check finds no update", async () => {
    checkMock.mockResolvedValue(null);

    await checkForAppUpdates("manual");

    expect(messageMock).toHaveBeenCalledWith("当前已是最新版本。", {
      title: "检查更新",
      kind: "info",
    });
  });

  it("downloads and relaunches when user confirms", async () => {
    const downloadAndInstall = vi.fn().mockResolvedValue(undefined);
    checkMock.mockResolvedValue({
      available: true,
      version: "1.0.1",
      body: "修复问题",
      downloadAndInstall,
    });
    askMock.mockResolvedValue(true);

    await checkForAppUpdates("manual");

    expect(askMock).toHaveBeenCalled();
    expect(downloadAndInstall).toHaveBeenCalled();
    expect(relaunchMock).toHaveBeenCalled();
  });

  it("does nothing when user declines update", async () => {
    checkMock.mockResolvedValue({
      available: true,
      version: "1.0.1",
      body: "",
      downloadAndInstall: vi.fn(),
    });
    askMock.mockResolvedValue(false);

    await checkForAppUpdates("manual");

    expect(relaunchMock).not.toHaveBeenCalled();
  });
});
