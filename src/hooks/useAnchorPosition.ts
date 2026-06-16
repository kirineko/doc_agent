import { useEffect, useState, type RefObject } from "react";

export interface AnchorPosition {
  top?: number;
  bottom?: number;
  left: number;
  width: number;
  maxHeight: number;
  placement: "above" | "below";
}

interface UseAnchorPositionOptions {
  gap?: number;
  width?: number;
  maxHeight?: number;
}

export function useAnchorPosition(
  triggerRef: RefObject<HTMLElement | null>,
  open: boolean,
  options: UseAnchorPositionOptions = {},
): AnchorPosition | undefined {
  const { gap = 8, width: preferredWidth = 320, maxHeight: preferredMaxHeight = 420 } = options;
  const [position, setPosition] = useState<AnchorPosition>();

  useEffect(() => {
    if (!open) {
      setPosition(undefined);
      return;
    }

    function update() {
      const trigger = triggerRef.current;
      if (!trigger) return;

      const rect = trigger.getBoundingClientRect();
      const width = Math.min(preferredWidth, Math.max(rect.width, 240));
      const left = rect.left;
      const spaceAbove = rect.top - gap;
      const spaceBelow = window.innerHeight - rect.bottom - gap;
      const openAbove = spaceAbove >= 180 || spaceAbove >= spaceBelow;

      if (openAbove) {
        setPosition({
          bottom: window.innerHeight - rect.top + gap,
          left,
          width,
          maxHeight: Math.min(preferredMaxHeight, Math.max(spaceAbove, 160)),
          placement: "above",
        });
        return;
      }

      setPosition({
        top: rect.bottom + gap,
        left,
        width,
        maxHeight: Math.min(preferredMaxHeight, Math.max(spaceBelow, 160)),
        placement: "below",
      });
    }

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [open, triggerRef, gap, preferredWidth, preferredMaxHeight]);

  return position;
}
