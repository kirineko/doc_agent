import { describe, expect, it } from "vitest";
import { formatUpdateOverlayCopy } from "./UpdateProgressOverlay";

describe("formatUpdateOverlayCopy", () => {
  it("uses two-line installing copy without ellipsis", () => {
    expect(formatUpdateOverlayCopy("installing")).toEqual({
      primary: "正在安装",
      secondary: "即将重启",
    });
  });

  it("splits version and percent while downloading", () => {
    expect(formatUpdateOverlayCopy("downloading", "2026.6.17", 45)).toEqual({
      primary: "正在下载",
      secondary: "v2026.6.17",
      percentLine: "45%",
    });
  });

  it("omits version line when version is missing", () => {
    expect(formatUpdateOverlayCopy("downloading", undefined, 12)).toEqual({
      primary: "正在下载",
      percentLine: "12%",
    });
  });

  it("never includes ellipsis characters", () => {
    for (const copy of [
      formatUpdateOverlayCopy("installing"),
      formatUpdateOverlayCopy("downloading", "2026.6.17", 99),
      formatUpdateOverlayCopy("downloading", "2026.6.17"),
      formatUpdateOverlayCopy("downloading"),
    ]) {
      const text = JSON.stringify(copy);
      expect(text).not.toMatch(/…|\.\.\./);
    }
  });
});
