import type { CodeName } from "./api";

const normalize = (text: string) => text.normalize("NFKC").toLowerCase().replace(/[^\p{L}\p{M}\p{N}]+/gu, " ").trim().replace(/\s+/g, " ");
function distance(left: string, right: string) {
  const a = [...left], b = [...right];
  let row = Array.from({ length: b.length + 1 }, (_, index) => index);
  for (let i = 0; i < a.length; i++) {
    const next = [i + 1];
    for (let j = 0; j < b.length; j++) next.push(Math.min(next[j] + 1, row[j + 1] + 1, row[j] + Number(a[i] !== b[j])));
    row = next;
  }
  return row[b.length];
}

export function brandScore(written: string, option: CodeName): number {
  const wanted = normalize(written).slice(0, 160), name = normalize(option.name), code = normalize(option.code);
  if (!wanted) return 1;
  if (wanted === name || wanted === code || wanted === normalize(`${option.name} ${option.code}`)) return 100;
  if (normalize(written.split(/[-–—|/]/)[0]) === name) return 100;
  if (name.length >= 3 && ` ${wanted} `.includes(` ${name} `)) return 88 + Math.min(6, (name.split(" ").length - 1) * 6) + Math.min(3, name.length / wanted.length * 3);
  if (name.includes(wanted) || code.startsWith(wanted)) return 70 + Math.min(8, wanted.length / name.length * 8);
  const words = wanted.split(" "), count = name.split(" ").length;
  const variants = [wanted, normalize(written.split(/[-–—|/]/)[0]), words.slice(0, count).join(" ")];
  const similarity = Math.max(...variants.filter(value => value.length >= 4).map(value => 1 - distance(value, name) / Math.max(value.length, name.length)), 0);
  return similarity >= 0.78 ? 90 + similarity * 8 : 0;
}

export function similarBrands(written: string, options: CodeName[]) {
  return options.map(option => ({ option, score: brandScore(written, option) })).filter(match => match.score > 0)
    .sort((a, b) => b.score - a.score || a.option.name.localeCompare(b.option.name, "th"));
}

// Fill only a strong unique match; typing a short prefix still requires a choice.
export function matchBrand(written: string | null | undefined, options: CodeName[]) {
  if (!written?.trim()) return undefined;
  const matches = similarBrands(written, options), first = matches[0], second = matches[1];
  return first && first.score >= 88 && (!second || (first.score === 100 ? second.score < 100 : first.score - second.score >= 5)) ? first.option : undefined;
}

export const brandLabel = (option: CodeName) => `${option.name} (${option.code})`;
