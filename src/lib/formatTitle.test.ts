import { describe, expect, it } from "vitest";
import { plainSessionTitle } from "./formatTitle";

describe("plainSessionTitle", () => {
  it("strips inline markdown markers", () => {
    expect(plainSessionTitle("**报告** `draft`")).toBe("报告 draft");
  });

  it("unwraps markdown links", () => {
    expect(plainSessionTitle("参考 [文档](https://example.com)")).toBe("参考 文档");
  });
});
