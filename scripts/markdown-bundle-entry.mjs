import matter from "gray-matter";
import hljs from "highlight.js/lib/core";
import bash from "highlight.js/lib/languages/bash";
import css from "highlight.js/lib/languages/css";
import html from "highlight.js/lib/languages/xml";
import javascript from "highlight.js/lib/languages/javascript";
import json from "highlight.js/lib/languages/json";
import markdown from "highlight.js/lib/languages/markdown";
import python from "highlight.js/lib/languages/python";
import rust from "highlight.js/lib/languages/rust";
import sql from "highlight.js/lib/languages/sql";
import typescript from "highlight.js/lib/languages/typescript";
import yaml from "highlight.js/lib/languages/yaml";

const LANGS = [
  ["python", python],
  ["javascript", javascript],
  ["js", javascript],
  ["typescript", typescript],
  ["ts", typescript],
  ["rust", rust],
  ["bash", bash],
  ["sh", bash],
  ["sql", sql],
  ["json", json],
  ["yaml", yaml],
  ["yml", yaml],
  ["html", html],
  ["xml", html],
  ["css", css],
  ["markdown", markdown],
  ["md", markdown],
];

for (const [name, mod] of LANGS) {
  hljs.registerLanguage(name, mod);
}

let markedReady = false;
let codeHighlightEnabled = true;

function escapeHtml(text) {
  return text
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function ensureMarked() {
  const marked = globalThis.marked;
  if (!marked || typeof marked.parse !== "function") {
    throw new Error("marked global required before MarkdownConvert");
  }
  if (!markedReady) {
    marked.setOptions({ gfm: true, breaks: false });
    const baseRenderer = new marked.Renderer();
    marked.use({
      renderer: {
        table(token) {
          const inner = baseRenderer.table.call(this, token);
          return `<div class="table-wrap">${inner}</div>`;
        },
        code({ text, lang }) {
          const langNorm = (lang || "").trim().toLowerCase();
          if (langNorm === "mermaid") {
            const escaped = text
              .replace(/&/g, "&amp;")
              .replace(/</g, "&lt;")
              .replace(/>/g, "&gt;");
            return `<pre><code class="language-mermaid">${escaped}</code></pre>`;
          }
          if (!codeHighlightEnabled) {
            const escaped = escapeHtml(text);
            const cls = langNorm ? `language-${langNorm}` : "";
            return cls
              ? `<pre><code class="${cls}">${escaped}</code></pre>`
              : `<pre><code>${escaped}</code></pre>`;
          }
          const language = lang && hljs.getLanguage(lang) ? lang : "plaintext";
          let highlighted;
          try {
            highlighted =
              language === "plaintext"
                ? escapeHtml(text)
                : hljs.highlight(text, { language }).value;
          } catch {
            highlighted = escapeHtml(text);
          }
          const cls =
            language === "plaintext" ? "hljs" : `hljs language-${language}`;
          return `<pre><code class="${cls}">${highlighted}</code></pre>`;
        },
      },
    });
    markedReady = true;
  }
  return marked;
}

function isTableRow(line) {
  const t = line.trim();
  return t.includes("|") && /^\|?.+\|/.test(t);
}

function isTableSeparator(line) {
  const t = line.trim();
  return t.includes("|") && t.includes("-") && /^[\|\s:\-]+$/.test(t);
}

function preprocessGfmTables(md) {
  const lines = md.split("\n");
  const out = [];
  let i = 0;
  while (i < lines.length) {
    if (i + 1 < lines.length && isTableRow(lines[i]) && isTableSeparator(lines[i + 1])) {
      const block = [];
      while (i < lines.length && (isTableRow(lines[i]) || isTableSeparator(lines[i]))) {
        block.push(lines[i]);
        i++;
      }
      const html = ensureMarked().parse(block.join("\n")).trim();
      out.push(html);
      continue;
    }
    out.push(lines[i]);
    i++;
  }
  return out.join("\n");
}

function detectNeeds(src) {
  return {
    katex: /\$\$[\s\S]+?\$\$|\$[^$\n]+\$/.test(src),
    mermaid: /```mermaid[\s\S]*?```/.test(src),
  };
}

function injectFigureCaptionClasses(html) {
  return html.replace(
    /<p>(\s*(?:<em>)?\s*(?:图[：:]|Figure\s*\d*[：:.]))/gi,
    '<p class="figure-caption">$1',
  );
}

function isImageOnlyParagraph(inner) {
  return /^\s*<img[\s\S]*?>\s*$/i.test(inner.trim());
}

function isCaptionParagraph(inner) {
  const t = inner.trim();
  if (/^<em>[\s\S]*<\/em>$/i.test(t)) return true;
  if (/^(?:<em>)?\s*(?:图[：:]|Figure\s*\d*[：:.])/i.test(t)) return true;
  return false;
}

function wrapFigureBlocks(html) {
  let out = html.replace(
    /<p>(\s*<img[\s\S]*?>)\s*(<em>[\s\S]*?<\/em>)\s*<\/p>/gi,
    (_, img, em) =>
      `<figure class="md-figure">${img.trim()}<figcaption>${em}</figcaption></figure>`,
  );
  out = out.replace(
    /<p>([\s\S]*?)<\/p>\s*<p>([\s\S]*?)<\/p>/gi,
    (full, imgBlock, captionBlock) => {
      if (!isImageOnlyParagraph(imgBlock) || !isCaptionParagraph(captionBlock)) {
        return full;
      }
      const caption = captionBlock.trim();
      return `<figure class="md-figure">${imgBlock.trim()}<figcaption>${caption}</figcaption></figure>`;
    },
  );
  return out;
}

/** Wrap each h3 block (heading + following content until next h2/h3) for multi-column resume layouts. */
function wrapResumeEntries(html) {
  return html.replace(
    /(<h3\b[^>]*>[\s\S]*?<\/h3>(?:\s*(?!<h[23]\b)[\s\S])*?)(?=<h[23]\b|$)/gi,
    (block) => `<div class="resume-entry">${block.trim()}</div>\n`,
  );
}

/** Minimum plain-text chars in a section (excluding h2) before enabling two-column layout. */
const RESUME_SECTION_COLS_MIN_CHARS = 80;

function sectionBodyPlainLength(block) {
  return block
    .replace(/^<h2\b[\s\S]*?<\/h2>/i, "")
    .replace(/<[^>]+>/g, "")
    .replace(/\s+/g, " ")
    .trim().length;
}

/** Group h2 + entries; dual-column class only when section has enough content. */
function wrapResumeSections(html) {
  return html.replace(
    /(<h2\b[^>]*>[\s\S]*?<\/h2>(?:\s*<div class="resume-entry">[\s\S]*?<\/div>\s*)*)/gi,
    (block) => {
      const h2Match = block.match(/^<h2\b[\s\S]*?<\/h2>/i);
      if (!h2Match) return block;
      const h2 = h2Match[0];
      const entries = block.match(/<div class="resume-entry">[\s\S]*?<\/div>/gi) || [];
      const useCols =
        entries.length >= 2 && sectionBodyPlainLength(block) >= RESUME_SECTION_COLS_MIN_CHARS;
      const bodyEntries = entries.map((entry, index) => {
        const spanLastOdd = useCols && entries.length >= 3 && entries.length % 2 === 1;
        if (spanLastOdd && index === entries.length - 1) {
          return entry.replace(
            /^<div class="resume-entry"/,
            '<div class="resume-entry resume-entry--span"',
          );
        }
        return entry;
      });
      const sectionCls = useCols ? "resume-section resume-section--cols" : "resume-section";
      const bodyHtml = bodyEntries.length > 0 ? bodyEntries.join("\n") : "";
      return `<section class="${sectionCls}">${h2}\n<div class="resume-section-body">\n${bodyHtml}\n</div></section>\n`;
    },
  );
}

function extractHeadings(html) {
  const toc = [];
  const re = /<h([23])[^>]*(?:id="([^"]*)")?[^>]*>([\s\S]*?)<\/h\1>/gi;
  let m;
  let idx = 0;
  while ((m = re.exec(html)) !== null) {
    const level = Number(m[1]);
    const text = m[3].replace(/<[^>]+>/g, "").trim();
    if (!text) continue;
    const id = m[2] || `heading-${idx++}`;
    toc.push({ id, text, level });
  }
  return toc;
}

function injectHeadingIds(html) {
  let idx = 0;
  return html.replace(/<h([23])([^>]*)>([\s\S]*?)<\/h\1>/gi, (full, level, attrs, inner) => {
    if (/\bid\s*=/.test(attrs)) return full;
    const id = `heading-${idx++}`;
    return `<h${level}${attrs} id="${id}">${inner}</h${level}>`;
  });
}

function parseScalarYamlValue(raw) {
  let val = raw.trim();
  if (
    (val.startsWith('"') && val.endsWith('"')) ||
    (val.startsWith("'") && val.endsWith("'"))
  ) {
    return val.slice(1, -1);
  }
  if (val === "true") return true;
  if (val === "false") return false;
  if (/^-?\d+$/.test(val)) return Number.parseInt(val, 10);
  return val;
}

function parseSimpleFrontmatter(src) {
  if (!src.startsWith("---")) return { data: {}, content: src };
  const end = src.indexOf("\n---", 3);
  if (end < 0) return { data: {}, content: src };
  const yamlText = src.slice(3, end).trim();
  const content = src.slice(end + 4).replace(/^\n?/, "");
  const data = {};
  for (const line of yamlText.split("\n")) {
    const m = line.match(/^([\w-]+):\s*(.*)$/);
    if (!m) continue;
    data[m[1]] = parseScalarYamlValue(m[2]);
  }
  return { data, content };
}

function contentStillHasFrontmatter(content) {
  const trimmed = content.replace(/^\uFEFF/, "").trimStart();
  return trimmed.startsWith("---") || /^marp:\s/i.test(trimmed);
}

function stripEmbeddedFrontmatter(content) {
  let s = content.replace(/^\uFEFF/, "").trimStart();
  if (!s.startsWith("---")) return content.trimStart();
  const end = s.indexOf("\n---", 3);
  if (end < 0) return content.trimStart();
  return s.slice(end + 4).replace(/^\n?/, "");
}

function stripLeadingSlideSeparator(content) {
  const s = content.replace(/^\uFEFF/, "");
  if (/^---\s*\n(?!\s*[\w-]+\s*:)/.test(s)) {
    return s.replace(/^---\s*\n+/, "");
  }
  return content;
}

function normalizeFrontmatterBody(content) {
  return stripLeadingSlideSeparator(stripEmbeddedFrontmatter(content)).trimStart();
}

function parseFrontmatter(src) {
  try {
    const parsed = matter(src);
    const data = parsed.data || {};
    const content = parsed.content || "";
    if (Object.keys(data).length > 0 && !contentStillHasFrontmatter(content)) {
      return { data, content: normalizeFrontmatterBody(content) };
    }
  } catch {
    /* fall through */
  }
  const simple = parseSimpleFrontmatter(src);
  return { data: simple.data, content: normalizeFrontmatterBody(simple.content) };
}

globalThis.MarkdownConvert = {
  matter: parseFrontmatter,
  parseMarkdown(src, options = {}) {
    codeHighlightEnabled = options.highlight !== false;
    return ensureMarked().parse(src);
  },
  detectNeeds,
  extractHeadings,
  injectHeadingIds,
  injectFigureCaptionClasses,
  wrapFigureBlocks,
  wrapResumeEntries,
  wrapResumeSections,
  preprocessGfmTables,
};

export default globalThis.MarkdownConvert;
