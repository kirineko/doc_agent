import { describe, expect, it } from "vitest";
import {
  extensionForMime,
  IMAGE_FILE_ACCEPT,
  isAllowedImageMime,
  parseMessageAttachments,
} from "./attachments";
import { makeUserMessage } from "../test/fixtures/messages";

describe("attachments helpers", () => {
  it("parses attachments_json from messages", () => {
    const message = makeUserMessage({
      id: "u1",
      session_id: "s1",
      attachments_json: JSON.stringify([
        { path: ".cache/attachments/a.png", mime: "image/png" },
      ]),
    });
    expect(parseMessageAttachments(message)).toEqual([
      { path: ".cache/attachments/a.png", mime: "image/png" },
    ]);
  });

  it("validates allowed image mime types", () => {
    expect(isAllowedImageMime("image/png")).toBe(true);
    expect(isAllowedImageMime("image/jpeg")).toBe(true);
    expect(isAllowedImageMime("text/plain")).toBe(false);
  });

  it("maps mime to file extension", () => {
    expect(extensionForMime("image/webp")).toBe("webp");
    expect(extensionForMime("image/jpeg")).toBe("jpg");
  });

  it("exports image file accept filter", () => {
    expect(IMAGE_FILE_ACCEPT).toContain("image/*");
    expect(IMAGE_FILE_ACCEPT).toContain(".png");
    expect(IMAGE_FILE_ACCEPT).toContain("image/jpeg");
  });
});
