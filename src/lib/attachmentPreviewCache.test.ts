import { describe, expect, it, beforeEach } from "vitest";
import {
  attachmentPreviewCacheKey,
  clearAttachmentPreviewCache,
  getCachedAttachmentPreview,
  setCachedAttachmentPreview,
} from "./attachmentPreviewCache";

describe("attachmentPreviewCache", () => {
  beforeEach(() => {
    clearAttachmentPreviewCache();
  });

  it("stores and retrieves by project and path", () => {
    setCachedAttachmentPreview("p1", ".cache/attachments/a.png", "data:image/png;base64,abc");
    expect(getCachedAttachmentPreview("p1", ".cache/attachments/a.png")).toBe(
      "data:image/png;base64,abc",
    );
    expect(getCachedAttachmentPreview("p2", ".cache/attachments/a.png")).toBeUndefined();
  });

  it("uses stable cache keys", () => {
    expect(attachmentPreviewCacheKey("proj", "file.jpg")).toBe("proj:file.jpg");
  });
});
