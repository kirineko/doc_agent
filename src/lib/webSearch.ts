import { invoke } from "@tauri-apps/api/core";

export async function loadWebSearchActive(): Promise<boolean> {
  return invoke<boolean>("get_web_search_enabled");
}

export async function setWebSearchActive(enabled: boolean): Promise<void> {
  await invoke("set_web_search_enabled", { enabled });
}

export async function loadTavilyHasKey(): Promise<boolean> {
  return invoke<boolean>("has_api_key", { provider: "tavily" });
}

export async function refreshWebSearchState(): Promise<{
  hasKey: boolean;
  active: boolean;
}> {
  const [hasKey, active] = await Promise.all([loadTavilyHasKey(), loadWebSearchActive()]);
  return { hasKey, active };
}
