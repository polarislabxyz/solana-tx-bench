use serde::{Deserialize, Serialize};

pub const BASE_SIGNATURE_FEE_LAMPORTS: u64 = 5_000;

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct TransactionCost {
    pub signature_count: u64,
    pub base_fee_lamports: u64,
    pub priority_fee_lamports: u64,
    pub transaction_fee_lamports: u64,
}

impl TransactionCost {
    pub fn one_signature(compute_unit_limit: u32, compute_unit_price_microlamports: u64) -> Self {
        Self::for_signature_count(1, compute_unit_limit, compute_unit_price_microlamports)
    }

    pub fn for_signature_count(
        signature_count: u64,
        compute_unit_limit: u32,
        compute_unit_price_microlamports: u64,
    ) -> Self {
        let base_fee_lamports = signature_count.saturating_mul(BASE_SIGNATURE_FEE_LAMPORTS);
        let priority_fee_lamports =
            priority_fee_lamports(compute_unit_limit, compute_unit_price_microlamports);
        Self {
            signature_count,
            base_fee_lamports,
            priority_fee_lamports,
            transaction_fee_lamports: base_fee_lamports.saturating_add(priority_fee_lamports),
        }
    }
}

pub fn priority_fee_lamports(
    compute_unit_limit: u32,
    compute_unit_price_microlamports: u64,
) -> u64 {
    (compute_unit_limit as u64)
        .saturating_mul(compute_unit_price_microlamports)
        .saturating_add(999_999)
        / 1_000_000
}
