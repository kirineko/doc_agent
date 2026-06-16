import type { Message, MessageAttachment } from "../types";

export const MAX_ATTACHMENTS_PER_MESSAGE = 4;

export const ALLOWED_IMAGE_MIMES = new Set([
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
]);

/** 图片选择器允许的扩展名（不含 `.`） */
export const IMAGE_FILE_EXTENSIONS = ["png", "jpg", "jpeg", "webp", "gif"] as const;

/** HTML `<input type="file">` 的 accept 值 */
export const IMAGE_FILE_ACCEPT = [
  "image/*",
  ...Array.from(ALLOWED_IMAGE_MIMES),
  ...IMAGE_FILE_EXTENSIONS.map((ext) => `.${ext}`),
].join(",");

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

const ALLOWED_IMAGE_EXTENSIONS = new Set<string>(IMAGE_FILE_EXTENSIONS);

function extensionFromFilename(name: string): string {
  const index = name.lastIndexOf(".");
  if (index < 0) return "";
  return name.slice(index + 1).toLowerCase();
}

/** 校验 file picker 所选文件是否为支持的图片类型（MIME 或扩展名） */
export function isAllowedImageFile(file: File): boolean {
  const mime = file.type.toLowerCase();
  if (mime && isAllowedImageMime(mime)) return true;
  return ALLOWED_IMAGE_EXTENSIONS.has(extensionFromFilename(file.name));
}

export function resolveImageMime(file: File): string | null {
  const mime = file.type.toLowerCase();
  if (mime && isAllowedImageMime(mime)) return mime;
  const ext = extensionFromFilename(file.name);
  switch (ext) {
    case "png":
      return "image/png";
    case "jpg":
    case "jpeg":
      return "image/jpeg";
    case "webp":
      return "image/webp";
    case "gif":
      return "image/gif";
    default:
      return null;
  }
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
