use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const TOLERANCE_SATANG: i64 = 10_000;
pub const BANK_TRANSFER: &str = "โอนเงิน";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BankTransferSlip {
    pub source_ref: String,
    pub amount_satang: i64,
    pub currency: String,
    pub bank: String,
    pub transaction_ref: Option<String>,
    pub completed: bool,
    pub quote: String,
}

impl BankTransferSlip {
    pub fn matches(&self, rental_net: i64, deposit_net: i64) -> bool {
        rental_net >= 0
            && deposit_net >= 0
            && self.completed
            && self.currency == "THB"
            && self
                .transaction_ref
                .as_ref()
                .is_some_and(|reference| !reference.trim().is_empty())
            && rental_net.checked_add(deposit_net).is_some_and(|net| {
                net > 0 && self.amount_satang.abs_diff(net) <= TOLERANCE_SATANG as u64
            })
    }

    pub fn fingerprint(&self) -> String {
        format!(
            "{}:{}",
            self.bank.trim().to_lowercase(),
            self.transaction_ref
                .as_deref()
                .unwrap_or("")
                .trim()
                .to_lowercase()
        )
    }
}

pub fn validate(
    slips: &[BankTransferSlip],
    selected_images: &BTreeSet<String>,
) -> Result<(), AppError> {
    let mut seen = BTreeSet::new();
    if slips.len() > 5
        || slips.iter().any(|slip| {
            !selected_images.contains(&slip.source_ref)
                || !seen.insert(&slip.source_ref)
                || !(1..=10_000_000_000).contains(&slip.amount_satang)
                || slip.currency.len() > 8
                || slip.currency.trim().is_empty()
                || slip.bank.chars().count() > 100
                || slip.bank.trim().is_empty()
                || slip.quote.chars().count() > 2000
                || slip.quote.trim().is_empty()
                || slip
                    .transaction_ref
                    .as_ref()
                    .is_some_and(|reference| reference.chars().count() > 200)
        })
    {
        return Err(AppError::Validation(
            "Bank slips must reference selected images and readable transfer evidence",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slip(amount: i64) -> BankTransferSlip {
        BankTransferSlip {
            source_ref: "image-1".into(),
            amount_satang: amount,
            currency: "THB".into(),
            bank: "Test bank".into(),
            transaction_ref: Some("TX-1".into()),
            completed: true,
            quote: "Transfer completed 900.00 THB TX-1".into(),
        }
    }

    #[test]
    fn completed_thb_slips_match_the_final_net_with_a_hundred_baht_tolerance() {
        for amount in [80_000, 90_000, 100_000] {
            assert!(slip(amount).matches(60_000, 30_000));
        }
        for amount in [79_999, 100_001] {
            assert!(!slip(amount).matches(60_000, 30_000));
        }
        assert!(!slip(90_000).matches(70_001, 30_000));
        assert!(!slip(1).matches(0, 0));
        assert!(!slip(90_000).matches(-1, 90_001));
        assert!(!slip(90_000).matches(i64::MAX, 1));
        let mut value = slip(90_000);
        value.completed = false;
        assert!(!value.matches(60_000, 30_000));
        value.completed = true;
        value.currency = "USD".into();
        assert!(!value.matches(60_000, 30_000));
        value.currency = "THB".into();
        value.transaction_ref = None;
        assert!(!value.matches(60_000, 30_000));
    }

    #[test]
    fn slip_evidence_cannot_come_from_another_image_or_from_text() {
        let images = BTreeSet::from(["image-1".into()]);
        assert!(validate(&[slip(90_000)], &images).is_ok());
        assert!(validate(&[slip(90_000)], &BTreeSet::new()).is_err());
        assert!(validate(&[slip(90_000), slip(90_000)], &images).is_err());
        assert!(validate(&[slip(0)], &images).is_err());
    }
}
