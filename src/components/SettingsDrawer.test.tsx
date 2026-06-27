import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { SettingsDrawer } from "./SettingsDrawer";
import { ThemeTestWrapper } from "../test/renderWithTheme";
import { UiScaleProvider } from "../hooks/useUiScale";
import { UI_SCALE_STORAGE_KEY } from "../lib/uiScale";
import { WORKSPACE_LAYOUT_RESET_EVENT } from "../lib/workspaceLayout";

vi.mock("../lib/uiScale", async (importOriginal) => {
  const actual = await importOriginal<typeof import("../lib/uiScale")>();
  return {
    ...actual,
    applyWebviewZoom: vi.fn(async () => undefined),
  };
});

vi.mock("../lib/updater", () => ({
  checkForAppUpdates: vi.fn(),
  fetchLatestReleaseVersion: vi.fn(async () => null),
  isNewerVersion: vi.fn(() => false),
}));

vi.mock("../lib/providerBalance", () => ({
  configuredBalanceProviders: vi.fn(() => []),
  fetchProviderBalances: vi.fn(async () => []),
}));

function renderDrawer() {
  return render(
    <ThemeTestWrapper>
      <UiScaleProvider>
        <SettingsDrawer open apiKeyStatus={{}} onClose={() => undefined} />
      </UiScaleProvider>
    </ThemeTestWrapper>,
  );
}

describe("SettingsDrawer", () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it("restore defaults resets layout and ui scale", () => {
    localStorage.setItem(UI_SCALE_STORAGE_KEY, "1.6");
    const layoutReset = vi.fn();
    window.addEventListener(WORKSPACE_LAYOUT_RESET_EVENT, layoutReset);

    renderDrawer();
    expect(screen.getByText("160%")).toBeTruthy();

    fireEvent.click(screen.getByRole("button", { name: "恢复默认布局" }));

    expect(layoutReset).toHaveBeenCalledTimes(1);
    expect(screen.getByText("100%")).toBeTruthy();
    expect(localStorage.getItem(UI_SCALE_STORAGE_KEY)).toBe("1");

    window.removeEventListener(WORKSPACE_LAYOUT_RESET_EVENT, layoutReset);
  });
});
