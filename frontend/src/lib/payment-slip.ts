export type BankTransferSlip = { source_ref: string; amount_satang: number; currency: string; bank: string; transaction_ref: string | null; completed: boolean; quote: string };
export type PaymentSlipMatch = { slip: BankTransferSlip; net_satang: number; tolerance_satang: number };

export function matchPaymentSlip(slips: BankTransferSlip[], rentalNet: number, depositNet: number, tolerance: number): PaymentSlipMatch | null {
  const net = rentalNet + depositNet;
  if (![rentalNet, depositNet, net, tolerance].every(Number.isSafeInteger) || rentalNet < 0 || depositNet < 0 || net <= 0 || tolerance < 0) return null;
  const candidates = slips.filter(slip => slip.completed && slip.currency === "THB" && slip.bank.trim() && slip.transaction_ref?.trim() && Number.isSafeInteger(slip.amount_satang) && slip.amount_satang > 0 && Math.abs(slip.amount_satang - net) <= tolerance);
  candidates.sort((a, b) => Math.abs(a.amount_satang - net) - Math.abs(b.amount_satang - net) || a.source_ref.localeCompare(b.source_ref));
  return candidates[0] ? { slip: candidates[0], net_satang: net, tolerance_satang: tolerance } : null;
}
