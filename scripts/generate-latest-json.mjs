#!/usr/bin/env node
/**
 * Generate Tauri updater latest.json from downloaded CI artifacts.
 * Usage: VERSION=1.0.0 OSS_BUCKET=doc-agent OSS_REGION=oss-cn-guangzhou node scripts/generate-latest-json.mjs [distDir]
 */
import { readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, join } from "node:path";

function walkFiles(dir, predicate, results = []) {
  for (const name of readdirSync(dir)) {
    const path = join(dir, name);
    const stat = statSync(path);
    if (stat.isDirectory()) {
      walkFiles(path, predicate, results);
    } else if (predicate(path)) {
      results.push(path);
    }
  }
  return results;
}

const distDir = process.argv[2] ?? "dist";
const version = process.env.VERSION ?? process.env.GITHUB_REF_NAME;
const bucket = process.env.OSS_BUCKET;
const region = process.env.OSS_REGION;

if (!version) {
  console.error("VERSION or GITHUB_REF_NAME is required");
  process.exit(1);
}
if (!bucket || !region) {
  console.error("OSS_BUCKET and OSS_REGION are required");
  process.exit(1);
}

const macTar = walkFiles(distDir, (p) => p.endsWith(".app.tar.gz") && !p.endsWith(".sig"))[0];
const winExe = walkFiles(distDir, (p) => /-setup\.exe$/i.test(p) && !p.endsWith(".sig"))[0];

if (!macTar || !winExe) {
  console.error("Missing updater artifacts", { macTar, winExe });
  process.exit(1);
}

const macSig = `${macTar}.sig`;
const winSig = `${winExe}.sig`;

for (const sig of [macSig, winSig]) {
  try {
    readFileSync(sig, "utf8");
  } catch {
    console.error(`Missing signature file: ${sig}`);
    process.exit(1);
  }
}

const base = `https://${bucket}.${region}.aliyuncs.com/releases/${version}`;
const payload = {
  version,
  notes: `Doc Agent ${version}`,
  pub_date: new Date().toISOString(),
  platforms: {
    "darwin-aarch64": {
      url: `${base}/${basename(macTar)}`,
      signature: readFileSync(macSig, "utf8").trim(),
    },
    "windows-x86_64": {
      url: `${base}/${basename(winExe)}`,
      signature: readFileSync(winSig, "utf8").trim(),
    },
  },
};

writeFileSync("latest.json", `${JSON.stringify(payload, null, 2)}\n`);
console.log("Generated latest.json:");
console.log(JSON.stringify(payload, null, 2));
