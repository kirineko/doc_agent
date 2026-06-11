import { memo } from "react";
import ReactMarkdown from "react-markdown";
import type { Components } from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import rehypeHighlight from "rehype-highlight";
import "katex/dist/katex.min.css";

const remarkPlugins = [remarkGfm, remarkMath];
const rehypePlugins = [rehypeKatex, rehypeHighlight];

const inlineComponents: Components = {
  p: ({ children }) => <>{children}</>,
};

export const MarkdownView = memo(function MarkdownView({
  content,
  variant = "default",
  inline = false,
}: {
  content: string;
  variant?: "default" | "compact";
  inline?: boolean;
}) {
  const className =
    variant === "compact"
      ? "markdown-body max-w-none text-xs leading-5"
      : "markdown-body max-w-none text-sm leading-7";
  return (
    <div className={className}>
      <ReactMarkdown
        remarkPlugins={remarkPlugins}
        rehypePlugins={rehypePlugins}
        components={inline ? inlineComponents : undefined}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
});
