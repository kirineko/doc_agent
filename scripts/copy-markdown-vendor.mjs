import { execFile } from "node:child_process";
import { copyFile, mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);

const root = path.dirname(fileURLToPath(import.meta.url));
const repo = path.join(root, "..");
const outDir = path.join(repo, "src-tauri/assets/markdown-vendor");

const copies = [
  ["node_modules/katex/dist/katex.min.js", "katex.min.js"],
  ["node_modules/katex/dist/katex.min.css", "katex.min.css"],
  ["node_modules/mermaid/dist/mermaid.min.js", "mermaid.min.js"],
  ["node_modules/highlight.js/styles/github.min.css", "highlight.css"],
  ["node_modules/@marp-team/marp-cli/lib/bespoke.js", "bespoke.js"],
];

async function extractMarpBespokeAssets() {
  const tmp = await mkdtemp(path.join(tmpdir(), "marp-vendor-"));
  try {
    const deck = path.join(tmp, "deck.md");
    const html = path.join(tmp, "deck.html");
    await writeFile(
      deck,
      "---\nmarp: true\ntheme: default\n---\n\n# Slide\n",
      "utf8",
    );
    const marpCli = path.join(repo, "node_modules/@marp-team/marp-cli/marp-cli.js");
    await execFileAsync(process.execPath, [marpCli, deck, "-o", html, "--no-stdin"], {
      cwd: tmp,
    });
    const raw = await readFile(html, "utf8");
    const styles = [...raw.matchAll(/<style>([\s\S]*?)<\/style>/g)].map((m) => m[1]);
    const scripts = [...raw.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]);
    if (!styles[0]) {
      throw new Error("bespoke-viewer.css: style block not found in marp-cli HTML");
    }
    if (!scripts[0]) {
      throw new Error("marp-browser-polyfill.js: script block not found in marp-cli HTML");
    }
    await writeFile(path.join(outDir, "bespoke-viewer.css"), styles[0], "utf8");
    await writeFile(path.join(outDir, "marp-browser-polyfill.js"), scripts[0], "utf8");
    console.log("extracted bespoke-viewer.css");
    console.log("extracted marp-browser-polyfill.js");
  } finally {
    await rm(tmp, { recursive: true, force: true });
  }
}

await mkdir(outDir, { recursive: true });
for (const [src, dest] of copies) {
  await copyFile(path.join(repo, src), path.join(outDir, dest));
  console.log(`copied ${dest}`);
}
await extractMarpBespokeAssets();
