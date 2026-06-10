import { build } from "esbuild";
import { mkdir } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.dirname(fileURLToPath(import.meta.url));
const outDir = path.join(root, "../src-tauri/assets/js");
await mkdir(outDir, { recursive: true });

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
];

for (const { name, entry, global } of libs) {
  const entryPath = path.join(root, "..", entry);
  // IIFE + globalName: boa_engine 的 eval 不支持 ESM export 语法。
  await build({
    entryPoints: [entryPath],
    bundle: true,
    format: "iife",
    globalName: global,
    platform: "browser",
    outfile: path.join(outDir, `${name}.bundle.js`),
    minify: true,
    define: { "process.env.NODE_ENV": '"production"' },
  });
  console.log(`bundled ${name} (global: ${global})`);
}
