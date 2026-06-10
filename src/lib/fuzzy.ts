export interface FuzzyMatch {
  item: string;
  score: number;
  positions: number[];
}

/** fzf 风格子序列匹配：连续命中与路径段首字母加分 */
export function fuzzyMatch(query: string, items: string[]): FuzzyMatch[] {
  const q = query.trim().toLowerCase();
  if (!q) {
    return items.slice(0, 8).map((item) => ({ item, score: 0, positions: [] }));
  }

  const results: FuzzyMatch[] = [];
  for (const item of items) {
    const lower = item.toLowerCase();
    const match = subsequenceMatch(q, lower);
    if (!match) continue;
    let score = match.positions.length * 10;
    for (let i = 1; i < match.positions.length; i++) {
      if (match.positions[i] === match.positions[i - 1]! + 1) {
        score += 5;
      }
    }
    for (const segment of item.split(/[/\\]/)) {
      if (segment.toLowerCase().startsWith(q[0] ?? "")) {
        score += 3;
      }
    }
    score -= Math.min(item.length, 80) * 0.05;
    results.push({ item, score, positions: match.positions });
  }

  return results.sort((a, b) => b.score - a.score);
}

function subsequenceMatch(
  query: string,
  target: string,
): { positions: number[] } | null {
  const positions: number[] = [];
  let qi = 0;
  for (let ti = 0; ti < target.length && qi < query.length; ti++) {
    if (target[ti] === query[qi]) {
      positions.push(ti);
      qi++;
    }
  }
  return qi === query.length ? { positions } : null;
}
