import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { MessageAttachment } from "../types";

type ThumbnailSize = "chip" | "message";

interface AttachmentThumbnailProps {
  src?: string;
  attachment?: MessageAttachment;
  projectId?: string;
  alt: string;
  size?: ThumbnailSize;
  removable?: boolean;
  onPreview?: (src: string) => void;
  onRemove?: () => void;
}

const sizeClass: Record<ThumbnailSize, string> = {
  chip: "h-[4.5rem] w-[4.5rem]",
  message: "h-28 w-28 sm:h-32 sm:w-32",
};

function Placeholder({ label }: { label: string }) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center gap-1 bg-surface-hover px-2 text-center text-[10px] text-fg-muted">
      <svg viewBox="0 0 24 24" className="h-5 w-5 opacity-60" aria-hidden="true">
        <path
          fill="currentColor"
          d="M21 19V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2ZM8.5 13.5l2.5 3 3.5-4.5 4.5 6H5l3.5-4.5Z"
        />
      </svg>
      <span>{label}</span>
    </div>
  );
}

export function AttachmentThumbnail({
  src,
  attachment,
  projectId,
  alt,
  size = "message",
  removable,
  onPreview,
  onRemove,
}: AttachmentThumbnailProps) {
  const [resolvedSrc, setResolvedSrc] = useState<string | undefined>(src);
  const [loading, setLoading] = useState(Boolean(attachment && projectId && !src));
  const [failed, setFailed] = useState(false);

  useEffect(() => {
    if (src) {
      setResolvedSrc(src);
      setLoading(false);
      setFailed(false);
      return;
    }
    if (!attachment || !projectId) {
      setResolvedSrc(undefined);
      setLoading(false);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setFailed(false);
    invoke<string>("read_attachment_preview", {
      req: {
        project_id: projectId,
        path: attachment.path,
        mime: attachment.mime,
      },
    })
      .then((dataUrl) => {
        if (cancelled) return;
        setResolvedSrc(dataUrl);
        setLoading(false);
      })
      .catch(() => {
        if (cancelled) return;
        setFailed(true);
        setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [attachment, projectId, src]);

  const previewSrc = resolvedSrc;
  const canPreview = Boolean(previewSrc && onPreview);

  return (
    <div
      className={`group relative overflow-hidden rounded-xl border border-border/80 bg-surface shadow-sm ring-1 ring-black/5 ${sizeClass[size]}`}
    >
      {loading && <Placeholder label="加载中" />}
      {!loading && failed && <Placeholder label="无法加载" />}
      {!loading && !failed && previewSrc && (
        <button
          type="button"
          className={`h-full w-full ${canPreview ? "cursor-zoom-in" : "cursor-default"}`}
          onClick={() => {
            if (previewSrc && onPreview) onPreview(previewSrc);
          }}
          aria-label={canPreview ? `${alt}，点击放大` : alt}
        >
          <img src={previewSrc} alt={alt} className="h-full w-full object-cover" loading="lazy" />
        </button>
      )}
      {removable && onRemove && (
        <button
          type="button"
          aria-label="移除图片"
          className="absolute right-1 top-1 rounded-full bg-black/60 px-1.5 py-0.5 text-[11px] leading-none text-white opacity-0 transition group-hover:opacity-100"
          onClick={(event) => {
            event.stopPropagation();
            onRemove();
          }}
        >
          ×
        </button>
      )}
    </div>
  );
}
