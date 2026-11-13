import test from "node:test";
import assert from "node:assert/strict";
import { matchBrand, similarBrands } from "../src/lib/brand-matching.ts";

const brands = [
  { code: "BG", name: "Blush Garden" }, { code: "B", name: "Blush" },
  { code: "BS", name: "Blue Sky" }, { code: "HT", name: "Haatiya" },
  { code: "VM", name: "Vietnam" }, { code: "WP", name: "White.Plan" },
];
test("brand and model title resolves to the full leading catalog brand", () => {
  assert.equal(matchBrand("Blush Garden - Long Dress 20", brands)?.code, "BG");
  assert.equal(matchBrand("Blush Garden / Long Dress 20", brands)?.code, "BG");
  assert.equal(matchBrand("Blush Garden (BG)", brands)?.code, "BG");
});
test("minor spelling errors and case resolve only when there is a clear match", () => {
  assert.equal(matchBrand("Blush Gardem - Long Dress 20", brands)?.code, "BG");
  assert.equal(matchBrand("Hatiya", brands)?.code, "HT");
  assert.equal(matchBrand(" ht ", brands)?.name, "Haatiya");
  assert.equal(matchBrand("White Plan", brands)?.code, "WP");
});
test("typed prefixes filter the list without guessing a brand", () => {
  assert.deepEqual(similarBrands("blu", brands).map(match => match.option.code).sort(), ["B", "BG", "BS"]);
  assert.equal(matchBrand("Haati", brands), undefined);
  assert.equal(matchBrand("blu", brands), undefined);
});
test("unrelated terms yield no results and empty search shows all brands", () => {
  assert.equal(similarBrands("zzzzzz", brands).length, 0);
  assert.equal(similarBrands("", brands).length, brands.length);
});
test("ambiguous spellings or duplicate names need the admin to choose", () => {
  const ambiguous = [{ code: "A", name: "Garden" }, { code: "B", name: "Gardan" }];
  assert.equal(matchBrand("Gardun", ambiguous), undefined);
  assert.equal(matchBrand("Garden", [...ambiguous, { code: "C", name: "Garden" }]), undefined);
});
