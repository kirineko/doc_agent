import { useCallback, useEffect, useMemo, useRef, type RefObject } from "react";
import {
  shouldAllowComposerFocus,
  type ComposerFocusBlockers,
} from "../lib/composerFocusPolicy";
import { useUpdateProgress } from "./useUpdateProgress";

interface UseComposerFocusOptions {
  textareaRef: RefObject<HTMLTextAreaElement | null>;
  projectId?: string;
  sessionId?: string;
  composerDisabled: boolean;
  importing?: boolean;
  blockers: Omit<ComposerFocusBlockers, "updateInProgress">;
}

export function useComposerFocus({
  textareaRef,
  projectId,
  sessionId,
  composerDisabled,
  importing = false,
  blockers,
}: UseComposerFocusOptions) {
  const updateProgress = useUpdateProgress();
  const prevComposerDisabledRef = useRef<boolean | undefined>(undefined);
  const prevImportingRef = useRef(false);
  const prevSessionIdRef = useRef<string | undefined>(undefined);
  const skipInitialSessionFocusRef = useRef(true);

  const focusContext = useMemo(
    () => ({
      projectSelected: Boolean(projectId),
      composerDisabled,
      blockers: {
        ...blockers,
        updateInProgress: updateProgress.phase !== "idle",
      },
    }),
    [projectId, composerDisabled, blockers, updateProgress.phase],
  );

  const scheduleFocus = useCallback(
    (start = 0, end = start) => {
      requestAnimationFrame(() => {
        if (!shouldAllowComposerFocus(focusContext)) return;
        const textarea = textareaRef.current;
        if (!textarea) return;
        textarea.focus();
        textarea.setSelectionRange(start, end);
      });
    },
    [focusContext, textareaRef],
  );

  // 回合结束：composerDisabled true → false 时 refocus（importing 恢复除外，
  // 避免覆盖导入流程设置的光标）。
  useEffect(() => {
    const wasDisabled = prevComposerDisabledRef.current;
    const wasImporting = prevImportingRef.current;
    prevComposerDisabledRef.current = composerDisabled;
    prevImportingRef.current = importing;
    if (wasDisabled === undefined) return;
    if (!wasDisabled || composerDisabled) return;
    if (wasImporting && !importing) return;
    scheduleFocus(0, 0);
  }, [composerDisabled, importing, scheduleFocus]);

  // 切换会话：sessionId 变化时 refocus（跳过首次 mount）。
  useEffect(() => {
    const prev = prevSessionIdRef.current;
    prevSessionIdRef.current = sessionId;
    if (skipInitialSessionFocusRef.current) {
      skipInitialSessionFocusRef.current = false;
      return;
    }
    if (prev === sessionId) return;
    if (!sessionId) return;
    scheduleFocus(0, 0);
  }, [sessionId, scheduleFocus]);
}
