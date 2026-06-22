export function isProfileInitMessage(text: string): boolean {
  const trimmed = text.trim();
  return trimmed === "/init" || trimmed.startsWith("/init ");
}
