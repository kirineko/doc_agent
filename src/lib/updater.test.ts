import { describe, expect, it, vi, beforeEach } from "vitest";

const { checkMock, askMock, messageMock, relaunchMock } = vi.hoisted(() => ({
  checkMock: vi.fn(),
  askMock: vi.fn(),
  messageMock: vi.fn(),
  relaunchMock: vi.fn(),
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

import { checkForAppUpdates } from "./updater";

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
