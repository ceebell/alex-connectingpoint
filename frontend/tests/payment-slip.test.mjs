import test from "node:test";
import assert from "node:assert/strict";
import { matchPaymentSlip } from "../src/lib/payment-slip.ts";

const receipt = amount => ({ source_ref: "image-1", amount_satang: amount, currency: "THB", bank: "Test bank", transaction_ref: "TX-1", completed: true, quote: "Transfer completed" });

test("a completed bank slip covers net rent plus net deposit within 100 baht", () => {
  for (const amount of [80000, 90000, 100000]) assert.equal(matchPaymentSlip([receipt(amount)], 60000, 30000, 10000)?.slip.amount_satang, amount);
  for (const amount of [79999, 100001]) assert.equal(matchPaymentSlip([receipt(amount)], 60000, 30000, 10000), null);
});

test("final admin discounts change the amount used to match a slip", () => {
  assert(matchPaymentSlip([receipt(90000)], 60000, 30000, 10000));
  assert.equal(matchPaymentSlip([receipt(90000)], 70001, 30000, 10000), null);
  assert(matchPaymentSlip([receipt(90000)], 50000, 30000, 10000));
});

test("pending, foreign currency, unreadable references and invalid amounts never mark payments", () => {
  for (const changed of [{ completed: false }, { currency: "USD" }, { transaction_ref: null }, { bank: "" }, { amount_satang: 0 }, { amount_satang: 90000.5 }]) assert.equal(matchPaymentSlip([{ ...receipt(90000), ...changed }], 60000, 30000, 10000), null);
  for (const [rent, deposit] of [[0, 0], [-1, 90001], [NaN, 30000], [Number.MAX_SAFE_INTEGER, 1]]) assert.equal(matchPaymentSlip([receipt(90000)], rent, deposit, 10000), null);
});

test("select the closest single receipt without combining separate slips", () => {
  const closest = { ...receipt(90000), source_ref: "image-2" };
  assert.equal(matchPaymentSlip([receipt(80000), closest], 60000, 30000, 10000)?.slip.source_ref, "image-2");
  assert.equal(matchPaymentSlip([receipt(40000), { ...receipt(50000), source_ref: "image-2" }], 60000, 30000, 10000), null);
});
