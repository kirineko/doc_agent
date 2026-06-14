/**
 * SemVer-compatible calendar version (YYYY.M.D).
 * MAJOR = year, MINOR = month (1-12), PATCH = day (1-31).
 * Numeric segments MUST NOT use leading zeros.
 */

const CALVER_PATTERN = /^(\d{4})\.(\d{1,2})\.(\d{1,2})$/;

function hasLeadingZero(segment: string): boolean {
  return segment.length > 1 && segment.startsWith("0");
}

export function isValidCalVerVersion(version: string): boolean {
  const trimmed = version.trim();
  const match = CALVER_PATTERN.exec(trimmed);
  if (!match) return false;

  const [, yearText, monthText, dayText] = match;
  if (hasLeadingZero(monthText) || hasLeadingZero(dayText)) return false;

  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);

  if (month < 1 || month > 12) return false;
  if (day < 1 || day > 31) return false;
  return year >= 2000;
}

/** ISO date for display (e.g. 2026.6.14 → 2026-06-14). */
export function formatCalVerDisplay(version: string): string {
  const match = CALVER_PATTERN.exec(version.trim());
  if (!match) return version;

  const [, year, month, day] = match;
  return `${year}-${month.padStart(2, "0")}-${day.padStart(2, "0")}`;
}

/** Today's CalVer tag in local timezone. */
export function calVerForDate(date: Date = new Date()): string {
  return `${date.getFullYear()}.${date.getMonth() + 1}.${date.getDate()}`;
}
