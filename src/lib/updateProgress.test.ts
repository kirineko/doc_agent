import { beforeEach, describe, expect, it } from "vitest";
import {
  applyDownloadEvent,
  computeUpdatePercent,
  getUpdateProgressSnapshot,
  resetUpdateProgress,
  startUpdateDownload,
  type UpdateProgressState,
} from "./updateProgress";

describe("updateProgress", () => {
  beforeEach(() => {
    resetUpdateProgress();
  });

  it("tracks download bytes and percent", () => {
    startUpdateDownload("2026.6.16");
    applyDownloadEvent({ event: "Started", data: { contentLength: 1000 } });
    applyDownloadEvent({ event: "Progress", data: { chunkLength: 250 } });
    applyDownloadEvent({ event: "Progress", data: { chunkLength: 250 } });

    const progress: UpdateProgressState = {
      phase: "downloading",
      version: "2026.6.16",
      downloadedBytes: 500,
      totalBytes: 1000,
    };

    expect(computeUpdatePercent(progress)).toBe(50);
    expect(getUpdateProgressSnapshot().downloadedBytes).toBe(500);
  });

  it("switches to installing on Finished", () => {
    startUpdateDownload("2026.6.16");
    applyDownloadEvent({ event: "Finished" });

    expect(getUpdateProgressSnapshot().phase).toBe("installing");
  });

  it("returns undefined percent without total bytes", () => {
    const progress: UpdateProgressState = {
      phase: "downloading",
      version: "2026.6.16",
      downloadedBytes: 500,
    };

    expect(computeUpdatePercent(progress)).toBeUndefined();
  });
});
