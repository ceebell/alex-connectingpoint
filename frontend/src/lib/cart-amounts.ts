type RentalLine = { candidate: { rental_fee_satang: number | null }; quantity: number; is_shipping?: boolean };

// A shipping catalog item is billed in full in the same order, outside the discount base.
export function discountableRental(items: readonly RentalLine[]): number {
  return items.filter(item => !item.is_shipping).reduce((sum, item) => sum + (item.candidate.rental_fee_satang ?? 0) * item.quantity, 0);
}
