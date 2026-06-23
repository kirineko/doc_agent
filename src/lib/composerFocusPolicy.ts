export interface ComposerFocusBlockers {
  settingsOpen?: boolean;
  credentialsOpen?: boolean;
  imagePreviewOpen?: boolean;
  slashMenuOpen?: boolean;
  mentionPopupOpen?: boolean;
  slashPopupOpen?: boolean;
  modelFlyoutOpen?: boolean;
  updateInProgress?: boolean;
}

export interface ComposerFocusContext {
  projectSelected: boolean;
  composerDisabled: boolean;
  blockers: ComposerFocusBlockers;
}

/** 是否允许自动 refocus（Overlay / disabled 时抑制）。 */
export function shouldAllowComposerFocus(ctx: ComposerFocusContext): boolean {
  if (!ctx.projectSelected) return false;
  if (ctx.composerDisabled) return false;
  const b = ctx.blockers;
  if (b.settingsOpen) return false;
  if (b.credentialsOpen) return false;
  if (b.imagePreviewOpen) return false;
  if (b.slashMenuOpen) return false;
  if (b.mentionPopupOpen) return false;
  if (b.slashPopupOpen) return false;
  if (b.modelFlyoutOpen) return false;
  if (b.updateInProgress) return false;
  return true;
}
