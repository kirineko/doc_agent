import type { Message, MessageAttachment } from "../types";

export const MAX_ATTACHMENTS_PER_MESSAGE = 4;

export const ALLOWED_IMAGE_MIMES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

export interface PendingAttachment extends MessageAttachment {
  previewUrl: string;
}

export function attachmentsMatch(a: MessageAttachment[], b: MessageAttachment[]): boolean {
  if (a.length !== b.length) return false;
  return a.every((item, index) => {
    const other = b[index];
    return other !== undefined && item.path === other.path && item.mime === other.mime;
  });
}

export function userMessageWasPersisted(
  messages: Message[],
  content: string,
  attachments: MessageAttachment[],
): boolean {
  const trimmed = content.trim();
  return messages.some((message) => {
    if (message.role !== "user") return false;
    if ((message.content ?? "").trim() !== trimmed) return false;
    return attachmentsMatch(parseMessageAttachments(message), attachments);
  });
}

export function parseMessageAttachments(message: Message): MessageAttachment[] {
  if (!message.attachments_json) return [];
  try {
    const parsed = JSON.parse(message.attachments_json) as MessageAttachment[];
    return Array.isArray(parsed) ? parsed : [];
  } catch {
    return [];
  }
}

export function isAllowedImageMime(mime: string): boolean {
  return ALLOWED_IMAGE_MIMES.has(mime);
}

export function extensionForMime(mime: string): string {
  switch (mime) {
    case "image/png":
      return "png";
    case "image/jpeg":
      return "jpg";
    case "image/webp":
      return "webp";
    case "image/gif":
      return "gif";
    default:
      return "png";
  }
}

export function blobToBase64(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const dataUrl = reader.result;
      if (typeof dataUrl !== "string") {
        reject(new Error("failed to read image"));
        return;
      }
      const comma = dataUrl.indexOf(",");
      resolve(comma >= 0 ? dataUrl.slice(comma + 1) : dataUrl);
    };
    reader.onerror = () => reject(reader.error ?? new Error("failed to read image"));
    reader.readAsDataURL(blob);
  });
}

export function readClipboardImageFile(
  clipboard: DataTransfer | null,
): { file: File; mime: string } | null {
  if (!clipboard) return null;
  for (const item of clipboard.items) {
    if (!item.type.startsWith("image/")) continue;
    const file = item.getAsFile();
    if (!file) continue;
    const mime = item.type || file.type || "image/png";
    if (!isAllowedImageMime(mime)) continue;
    return { file, mime };
  }
  return null;
}

export function revokePendingAttachments(items: PendingAttachment[]) {
  for (const item of items) {
    URL.revokeObjectURL(item.previewUrl);
  }
}
