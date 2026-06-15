import type { MessageAttachment } from "../types";
import { AttachmentThumbnail } from "./AttachmentThumbnail";

interface MessageAttachmentsProps {
  attachments: MessageAttachment[];
  projectId?: string;
  onPreview?: (src: string) => void;
}

export function MessageAttachments({ attachments, projectId, onPreview }: MessageAttachmentsProps) {
  if (attachments.length === 0) return null;

  return (
    <div className="mb-2.5 flex flex-wrap gap-2.5">
      {attachments.map((item) => (
        <AttachmentThumbnail
          key={item.path}
          attachment={item}
          projectId={projectId}
          alt="消息图片附件"
          size="message"
          onPreview={onPreview}
        />
      ))}
    </div>
  );
}
