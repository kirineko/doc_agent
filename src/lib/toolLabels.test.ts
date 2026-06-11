import { describe, expect, it } from "vitest";
import { REGISTERED_TOOL_NAMES, toolLabel, TOOL_LABELS } from "./toolLabels";

/** 与 registry.rs::default_tools 工具名列表一致 */
const EXPECTED_TOOLS = [
  "fs_list",
  "fs_read",
  "fs_write",
  "fs_search",
  "office_read_to_markdown",
  "office_convert",
  "word_create",
  "excel_read",
  "excel_write",
  "skill_read",
  "skill_run",
  "ooxml_unpack",
  "ooxml_pack",
  "docx_comment",
  "docx_accept_changes",
  "docx_extract_table",
  "excel_describe",
  "excel_normalize",
  "data_query",
  "xlsx_recalc",
  "pdf_merge",
  "pdf_split",
  "pdf_rotate",
  "pdf_delete_pages",
  "web_search",
  "web_extract",
] as const;

describe("toolLabels", () => {
  it("covers all registered tools", () => {
    expect([...REGISTERED_TOOL_NAMES].sort()).toEqual([...EXPECTED_TOOLS].sort());
  });

  it("returns Chinese labels for every registered tool", () => {
    for (const name of EXPECTED_TOOLS) {
      const label = toolLabel(name);
      expect(label).not.toBe(name);
      expect(label).not.toBe("未知工具");
      expect(TOOL_LABELS[name]).toBe(label);
      expect(/[\u4e00-\u9fff]/.test(label)).toBe(true);
    }
  });

  it("falls back to unknown label for unregistered tools", () => {
    expect(toolLabel("not_a_real_tool")).toBe("未知工具");
  });
});
