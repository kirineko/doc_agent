import { describe, expect, it } from "vitest";
import {
  applyRightPanelCollapseMode,
  normalizeRightPanelCollapseMode,
  rightPanelCollapseFlags,
  shouldExpandBothOnLayoutChange,
  toggleRightPanelCollapseMode,
} from "./rightPanelCollapse";

describe("rightPanelCollapse", () => {
  it("maps collapse mode to section flags", () => {
    expect(rightPanelCollapseFlags("both")).toEqual({
      toolchainCollapsed: false,
      filesCollapsed: false,
    });
    expect(rightPanelCollapseFlags("toolchain")).toEqual({
      toolchainCollapsed: true,
      filesCollapsed: false,
    });
    expect(rightPanelCollapseFlags("files")).toEqual({
      toolchainCollapsed: false,
      filesCollapsed: true,
    });
  });

  it("normalizes dual-collapsed persisted state to a single collapsed section", () => {
    expect(
      normalizeRightPanelCollapseMode({ toolchainCollapsed: true, filesCollapsed: true }),
    ).toBe("files");
  });

  it("toggles with mutual exclusion so only one section can stay collapsed", () => {
    expect(toggleRightPanelCollapseMode("both", "toolchain")).toBe("toolchain");
    expect(toggleRightPanelCollapseMode("toolchain", "toolchain")).toBe("both");
    expect(toggleRightPanelCollapseMode("files", "toolchain")).toBe("toolchain");
    expect(toggleRightPanelCollapseMode("both", "files")).toBe("files");
    expect(toggleRightPanelCollapseMode("files", "files")).toBe("both");
    expect(toggleRightPanelCollapseMode("toolchain", "files")).toBe("files");
  });

  it("expands both sections when layout changes while one is collapsed", () => {
    expect(shouldExpandBothOnLayoutChange("toolchain", false)).toBe(true);
    expect(shouldExpandBothOnLayoutChange("files", false)).toBe(true);
    expect(shouldExpandBothOnLayoutChange("both", false)).toBe(false);
    expect(shouldExpandBothOnLayoutChange("toolchain", true)).toBe(false);
  });

  it("expands the visible section before collapsing the other", () => {
    const calls: string[] = [];
    const toolchainPanel = {
      collapse: () => calls.push("toolchain.collapse"),
      expand: () => calls.push("toolchain.expand"),
    };
    const filesPanel = {
      collapse: () => calls.push("files.collapse"),
      expand: () => calls.push("files.expand"),
    };

    applyRightPanelCollapseMode(toolchainPanel, filesPanel, "toolchain");
    expect(calls).toEqual(["files.expand", "toolchain.collapse"]);

    calls.length = 0;
    applyRightPanelCollapseMode(toolchainPanel, filesPanel, "files");
    expect(calls).toEqual(["toolchain.expand", "files.collapse"]);
  });
});
