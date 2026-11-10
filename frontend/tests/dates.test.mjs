import assert from "node:assert/strict";
import test from "node:test";
import { DATE_PLACEHOLDER, formatDate, formatDateTime, parseDateInput } from "../src/lib/dates.ts";

test("dates use padded day/month and a Gregorian year", () => {
  assert.equal(DATE_PLACEHOLDER, "dd/mm/yyyy");
  assert.equal(formatDate("2026-01-05"), "05/01/2026");
  assert.equal(formatDate("2026-11-18"), "18/11/2026");
});

test("timestamps follow the Bangkok day at midnight and New Year", () => {
  assert.equal(formatDateTime("2026-11-09T16:59:00Z"), "09/11/2026 23:59");
  assert.equal(formatDateTime("2026-11-09T17:00:00Z"), "10/11/2026 00:00");
  assert.equal(formatDateTime("2026-12-31T17:05:00Z"), "01/01/2027 00:05");
});

test("booking days and timestamps do not depend on the host time zone", () => {
  const original = process.env.TZ;
  try {
    for (const zone of ["UTC", "America/Los_Angeles", "Asia/Tokyo"]) {
      process.env.TZ = zone;
      assert.equal(formatDate("2026-01-01"), "01/01/2026");
      assert.equal(formatDateTime("2025-12-31T17:00:00Z"), "01/01/2026 00:00");
    }
  } finally {
    if (original === undefined) delete process.env.TZ;
    else process.env.TZ = original;
  }
});

test("dates accept ISO timestamps, Date objects and millisecond timestamps", () => {
  const value = new Date("2026-11-09T17:30:00Z");
  assert.equal(formatDate(value), "10/11/2026");
  assert.equal(formatDateTime(value.getTime()), "10/11/2026 00:30");
  assert.equal(formatDateTime(0), "01/01/1970 07:00");
});

test("missing and invalid values render a safe placeholder", () => {
  for (const value of [null, undefined, "", "invalid", "2026-02-30", "2026-13-01", new Date(NaN)]) {
    assert.equal(formatDate(value), "—");
    assert.equal(formatDateTime(value), "—");
  }
});

test("typed dates convert to the API's ISO calendar format", () => {
  assert.equal(parseDateInput("05/01/2026"), "2026-01-05");
  assert.equal(parseDateInput("31/12/2026"), "2026-12-31");
  assert.equal(parseDateInput("18/11/1900"), "1900-11-18");
  assert.equal(parseDateInput("18/11/2199"), "2199-11-18");
});

test("calendar validation includes leap years and century exceptions", () => {
  assert.equal(parseDateInput("29/02/2024"), "2024-02-29");
  assert.equal(parseDateInput("29/02/2000"), "2000-02-29");
  for (const value of ["29/02/2026", "29/02/1900", "29/02/2100", "31/04/2026", "00/01/2026", "01/00/2026", "01/13/2026"]) assert.equal(parseDateInput(value), null);
});

test("incomplete, ambiguous and unsupported year inputs cannot reach the API", () => {
  for (const value of ["", "1/2/2026", "01/02/26", "02/2026", "2026-02-01", "01/02/2569", "01/01/1899", "01/01/2200", " 01/02/2026", "01/02/2026 extra"]) assert.equal(parseDateInput(value), null);
});
