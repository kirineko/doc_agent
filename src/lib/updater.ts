import { invoke } from "@tauri-apps/api/core";
import { check } from "@tauri-apps/plugin-updater";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";
import {
  applyDownloadEvent,
  resetUpdateProgress,
  startUpdateDownload,
} from "./updateProgress";

export const UPDATER_MANIFEST_URL =
  "https://doc-agent.oss-cn-guangzhou.aliyuncs.com/latest.json";

export type UpdateCheckMode = "silent" | "manual";

export function isNewerVersion(latest: string, current: string): boolean {
  const parse = (value: string) => value.split(".").map((part) => Number.parseInt(part, 10) || 0);
  const [latestMajor, latestMinor, latestPatch] = parse(latest);
  const [currentMajor, currentMinor, currentPatch] = parse(current);

  if (latestMajor !== currentMajor) return latestMajor > currentMajor;
  if (latestMinor !== currentMinor) return latestMinor > currentMinor;
  return latestPatch > currentPatch;
}

export async function fetchLatestReleaseVersion(): Promise<string | null> {
  try {
    return await invoke<string | null>("fetch_latest_release_version");
  } catch {
    return null;
  }
}

export async function checkForAppUpdates(mode: UpdateCheckMode): Promise<void> {
  try {
    const update = await check();
    if (!update?.available) {
      if (mode === "manual") {
        await message("当前已是最新版本。", { title: "检查更新", kind: "info" });
      }
      return;
    }

    const notes = update.body?.trim() || "无";
    const confirmed = await ask(
      `发现新版本 ${update.version}。\n\n更新说明：${notes}\n\n是否立即下载并安装？`,
      {
        title: "发现新版本",
        kind: "info",
        okLabel: "更新",
        cancelLabel: "稍后",
      },
    );
    if (!confirmed) return;

    startUpdateDownload(update.version);
    await update.downloadAndInstall((event) => {
      applyDownloadEvent(event);
    });
    await relaunch();
  } catch (error) {
    resetUpdateProgress();
    const detail = error instanceof Error ? error.message : String(error);
    await message(`更新失败：${detail}`, { title: "更新失败", kind: "error" });
    if (mode === "manual") {
      throw error;
    }
  }
}
