import type { PendingAttachment } from "../lib/attachments";
import { AttachmentThumbnail } from "./AttachmentThumbnail";

interface PendingAttachmentChipsProps {
  items: PendingAttachment[];
  disabled?: boolean;
  onRemove: (path: string) => void;
  onPreview?: (src: string) => void;
}

export function PendingAttachmentChips({
  items,
  disabled,
  onRemove,
  onPreview,
}: PendingAttachmentChipsProps) {
  if (items.length === 0) return null;

  return (
    <div className="flex flex-wrap gap-2.5">
      {items.map((item) => (
        <AttachmentThumbnail
          key={item.path}
          src={item.previewUrl}
          alt="待发送图片"
          size="chip"
          removable={!disabled}
          onPreview={onPreview}
          onRemove={() => onRemove(item.path)}
        />
      ))}
    </div>
  );
}
