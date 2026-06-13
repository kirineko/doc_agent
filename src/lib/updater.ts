import { check } from "@tauri-apps/plugin-updater";
import { ask, message } from "@tauri-apps/plugin-dialog";
import { relaunch } from "@tauri-apps/plugin-process";

export type UpdateCheckMode = "silent" | "manual";

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

    await update.downloadAndInstall();
    await relaunch();
  } catch (error) {
    const detail = error instanceof Error ? error.message : String(error);
    await message(`更新失败：${detail}`, { title: "更新失败", kind: "error" });
    if (mode === "manual") {
      throw error;
    }
  }
}
