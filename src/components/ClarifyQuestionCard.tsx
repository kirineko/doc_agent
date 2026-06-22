import { useMemo, useState } from "react";
import { normalizeBriefEntries } from "../lib/clarifyBrief";
import { InlineMarkdown } from "../lib/inlineMarkdown";
import type { ClarifyAnswer, ClarifyQuestion } from "../types";
import { MarkdownView } from "./MarkdownView";

interface ClarifyQuestionCardProps {
  question: ClarifyQuestion;
  answer?: ClarifyAnswer | null;
  disabled?: boolean;
  onSubmit?: (payload: { selected: string[]; custom?: string | null }) => void;
}

export function ClarifyQuestionCard({
  question,
  answer,
  disabled = false,
  onSubmit,
}: ClarifyQuestionCardProps) {
  const [selected, setSelected] = useState<string[]>([]);
  const [custom, setCustom] = useState("");
  const readonly = Boolean(answer) || disabled || !onSubmit;
  const allowCustom = question.allow_custom ?? true;

  const briefEntries = useMemo(
    () => normalizeBriefEntries(question.brief as Record<string, unknown> | null | undefined),
    [question.brief],
  );

  const display = answer?.display_text || (answer ? "已确认" : "");

  const count = selected.length + (custom.trim() ? 1 : 0);
  const min = question.min_selections ?? 1;
  const max = question.max_selections;
  const canSubmit = (() => {
    if (readonly) return false;
    if (question.kind === "text") return Boolean(custom.trim());
    if (question.kind === "confirm_brief" || question.kind === "confirm_agents_md") {
      return selected.includes("confirm") || Boolean(custom.trim());
    }
    if (question.kind === "multi") {
      if (count < min) return false;
      if (max && count > max) return false;
      return true;
    }
    return selected.length === 1 || Boolean(custom.trim());
  })();

  function toggleOption(id: string) {
    if (readonly) return;
    if (question.kind === "multi") {
      setSelected((prev) =>
        prev.includes(id) ? prev.filter((item) => item !== id) : [...prev, id],
      );
      return;
    }
    setSelected([id]);
  }

  function submit() {
    if (!canSubmit || !onSubmit) return;
    onSubmit({ selected, custom: custom.trim() || null });
  }

  return (
    <div className="rounded-xl border border-border bg-surface-2 p-3 text-sm shadow-sm">
      <div className="mb-1 text-[11px] font-medium text-fg-muted">需求澄清</div>
      <div className="font-medium text-fg-heading">
        <MarkdownView content={question.prompt} variant="compact" />
      </div>
      {question.description && (
        <div className="mt-1 text-fg-secondary">
          <MarkdownView content={question.description} variant="compact" />
        </div>
      )}

      {question.kind === "confirm_agents_md" && question.preview_markdown && (
        <div className="mt-3 max-h-64 overflow-y-auto rounded-lg border border-border-subtle p-2 text-xs">
          {question.changelog_summary && (
            <div className="mb-2 text-fg-muted">{question.changelog_summary}</div>
          )}
          <MarkdownView content={question.preview_markdown} variant="compact" />
        </div>
      )}

      {question.kind === "confirm_brief" && briefEntries.length > 0 && (
        <dl className="mt-3 grid gap-1.5 rounded-lg border border-border-subtle p-2 text-xs">
          {briefEntries.map(([key, value]) => (
            <div key={key} className="grid grid-cols-[5rem_1fr] gap-2">
              <dt className="text-fg-muted">{key}</dt>
              <dd className="text-fg-secondary">
                <MarkdownView content={value} variant="compact" />
              </dd>
            </div>
          ))}
        </dl>
      )}

      {answer ? (
        <div className="mt-3 rounded-lg bg-emerald-500/10 px-2.5 py-2 text-xs text-emerald-700 dark:text-emerald-300">
          已回答：{display}
        </div>
      ) : (
        <>
          {question.kind !== "text" &&
            question.kind !== "confirm_brief" &&
            question.kind !== "confirm_agents_md" && (
            <div className="mt-3 flex flex-wrap gap-1.5">
              {(question.options ?? []).map((option) => {
                const active = selected.includes(option.id);
                return (
                  <button
                    key={option.id}
                    type="button"
                    className={`rounded-lg border px-2.5 py-1.5 text-left text-xs ${
                      active
                        ? "border-sky-400 bg-sky-500/10 text-sky-700 dark:text-sky-300"
                        : "border-border-subtle bg-surface hover:border-border"
                    }`}
                    disabled={readonly}
                    onClick={() => toggleOption(option.id)}
                  >
                    <span className="block font-medium">
                      <InlineMarkdown text={option.label} />
                    </span>
                    {option.hint && (
                      <span className="block text-[11px] text-fg-muted">
                        <InlineMarkdown text={option.hint} />
                      </span>
                    )}
                  </button>
                );
              })}
            </div>
          )}

          {(question.kind === "confirm_brief" || question.kind === "confirm_agents_md") && (
            <div className="mt-3 flex flex-wrap gap-1.5">
              <button
                type="button"
                className={`rounded-lg border px-2.5 py-1.5 text-xs ${
                  selected.includes("confirm")
                    ? "border-sky-400 bg-sky-500/10 text-sky-700 dark:text-sky-300"
                    : "border-border-subtle bg-surface"
                }`}
                disabled={readonly}
                onClick={() => {
                  setSelected(["confirm"]);
                  setCustom("");
                }}
              >
                确认继续
              </button>
            </div>
          )}

          {allowCustom && (
            <textarea
              className="input-field mt-3 min-h-16 w-full resize-none rounded-lg px-2.5 py-2 text-xs"
              placeholder={
                question.custom_placeholder ??
                (question.kind === "confirm_brief" || question.kind === "confirm_agents_md"
                  ? "如需修改，请写下修改意见"
                  : "其他 / 自定义回答")
              }
              value={custom}
              disabled={readonly}
              onChange={(e) => {
                setCustom(e.target.value);
                if (
                  (question.kind === "confirm_brief" || question.kind === "confirm_agents_md") &&
                  e.target.value.trim()
                ) {
                  setSelected([]);
                }
              }}
            />
          )}

          {!readonly && (
            <div className="mt-2 flex items-center justify-between gap-2">
              <div className="text-[11px] text-fg-muted">
                {question.kind === "multi" && max ? `可选 ${min}-${max} 项` : "可选择预设项，也可自定义"}
              </div>
              <button
                type="button"
                className="btn-primary rounded-md px-3 py-1.5 text-xs disabled:opacity-50"
                disabled={!canSubmit}
                onClick={submit}
              >
                提交回答
              </button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
