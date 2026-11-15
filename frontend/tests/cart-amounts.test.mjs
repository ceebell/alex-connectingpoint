import test from "node:test";
import assert from "node:assert/strict";
import { discountableRental } from "../src/lib/cart-amounts.ts";

test("shipping stays in the cart but never becomes a rental discount", () => {
  const dress = { candidate: { rental_fee_satang: 60000 }, quantity: 1 };
  const shipping = { candidate: { rental_fee_satang: 4000 }, quantity: 1, is_shipping: true };
  assert.equal(discountableRental([dress, shipping]), 60000);
  assert.equal(discountableRental([dress, shipping]) - 55000, 5000);
  assert.equal(discountableRental([{ ...dress, quantity: 2 }, { ...shipping, quantity: 3 }]), 120000);
  assert.equal(discountableRental([shipping]), 0);
});
