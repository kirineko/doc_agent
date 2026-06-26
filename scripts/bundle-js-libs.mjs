import { build } from "esbuild";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(root, "../src-tauri/assets/js");
await mkdir(outDir, { recursive: true });

async function bundleMarkedUmd() {
  const umdPath = path.join(root, "../node_modules/marked/lib/marked.umd.js");
  const raw = await readFile(umdPath, "utf8");
  const wrapped = [
    "var module=void 0;var exports=void 0;var define=void 0;",
    raw,
    "globalThis.marked=globalThis.marked||marked;",
  ].join("\n");
  await writeFile(path.join(outDir, "marked.bundle.js"), wrapped);
  console.log("bundled marked (global: marked, umd copy)");
}

const libs = [
  {
    name: "docx",
    entry: "node_modules/docx/dist/index.mjs",
    global: "docx",
  },
  {
    name: "pptxgenjs",
    entry: "node_modules/pptxgenjs/dist/pptxgen.es.js",
    global: "PptxGenJS",
  },
  {
    name: "exceljs",
    entry: "node_modules/exceljs/dist/exceljs.bare.js",
    global: "ExcelJS",
  },
  {
    name: "pdf-lib",
    entry: "node_modules/pdf-lib/dist/pdf-lib.esm.js",
    global: "PDFLib",
  },
  {
    name: "markdown",
    entry: "scripts/markdown-bundle-entry.mjs",
    global: "MarkdownConvert",
  },
  {
    name: "marp-core",
    entry: "scripts/marp-bundle-entry.mjs",
    global: "Marpit",
  },
];

await bundleMarkedUmd();

for (const { name, entry, global } of libs) {
  const entryPath = path.join(root, "..", entry);
  const buildOpts = {
    entryPoints: [entryPath],
    bundle: true,
    format: "iife",
    globalName: global,
    platform: "browser",
    outfile: path.join(outDir, `${name}.bundle.js`),
    minify: name === "marp-core" || !["markdown", "marked"].includes(name),
    define: { "process.env.NODE_ENV": '"production"' },
  };
  await build(buildOpts);
  console.log(`bundled ${name} (global: ${global})`);
}
