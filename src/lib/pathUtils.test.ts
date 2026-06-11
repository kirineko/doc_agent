import { describe, expect, it } from "vitest";
import { joinPath, parentPath, pathBasename, pathSegments, segmentTarget } from "./pathUtils";

describe("pathUtils", () => {
  it("joinPath uses relative root", () => {
    expect(joinPath(".", "docs")).toBe("docs");
    expect(joinPath("docs", "a.md")).toBe("docs/a.md");
  });

  it("parentPath walks up one level", () => {
    expect(parentPath("docs/reports")).toBe("docs");
    expect(parentPath("docs")).toBe(".");
  });

  it("pathBasename handles os separators", () => {
    expect(pathBasename("/Users/foo/project")).toBe("project");
    expect(pathBasename("C:\\data\\file.csv")).toBe("file.csv");
  });

  it("segmentTarget rebuilds prefix", () => {
    const segments = pathSegments("docs/reports");
    expect(segmentTarget(segments, 0)).toBe("docs");
  });
});
