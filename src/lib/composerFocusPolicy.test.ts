import { describe, expect, it } from "vitest";
import { shouldAllowComposerFocus } from "./composerFocusPolicy";

function ctx(
  overrides: Partial<Parameters<typeof shouldAllowComposerFocus>[0]> = {},
) {
  return {
    projectSelected: true,
    composerDisabled: false,
    blockers: {},
    ...overrides,
  };
}

describe("shouldAllowComposerFocus", () => {
  it("allows focus when project selected and composer enabled", () => {
    expect(shouldAllowComposerFocus(ctx())).toBe(true);
  });

  it("blocks when no project", () => {
    expect(shouldAllowComposerFocus(ctx({ projectSelected: false }))).toBe(false);
  });

  it("blocks when composer disabled", () => {
    expect(shouldAllowComposerFocus(ctx({ composerDisabled: true }))).toBe(false);
  });

  it("blocks when settings drawer open", () => {
    expect(
      shouldAllowComposerFocus(ctx({ blockers: { settingsOpen: true } })),
    ).toBe(false);
  });

  it("blocks when any overlay blocker is open", () => {
    const cases = [
      { credentialsOpen: true },
      { imagePreviewOpen: true },
      { slashMenuOpen: true },
      { mentionPopupOpen: true },
      { slashPopupOpen: true },
      { modelFlyoutOpen: true },
      { commandPaletteOpen: true },
      { updateInProgress: true },
    ] as const;
    for (const blockers of cases) {
      expect(shouldAllowComposerFocus(ctx({ blockers }))).toBe(false);
    }
  });
});
