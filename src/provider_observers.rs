/// Jito bundle tip accounts returned by `getTipAccounts`.
///
/// These are included in benchmark observation filters so route-added Jito tip
/// transactions can be matched without requiring every run to pass extra
/// account filters manually.
pub const JITO_TIP_ACCOUNTS: &[&str; 8] = &[
    "96gYZGLnJYVFmbjzopPSU6QiEV5fGqZNyN9nmNhvrZU5",
    "HFqU5x63VTqvQss8hp11i4wVV8bD44PvwucfZ2bU7gRe",
    "Cw8CFyM9FkoMi7K7Crf6HNQqf4uEMzpKw6QNghXLvLkY",
    "ADaUMid9yfUytqMBgopwjb2DTLSokTSzL1zt6iGPaS49",
    "DfXygSm4jCyNCybVYYK6DwvWqjKee8pbDmJGcLWNDXjh",
    "ADuUkR4vqLUMWXxW9gh6D6L8pMSawimctcNZ5pGwDcEt",
    "DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL",
    "3AVi9Tg9Uo68tJfuvoKvqKNWKkC5wPdSSdeBnizKZ6jT",
];

pub fn add_default_provider_observation_accounts(accounts: &mut Vec<String>) {
    accounts.extend(
        JITO_TIP_ACCOUNTS
            .iter()
            .map(|account| (*account).to_string()),
    );
    accounts.sort();
    accounts.dedup();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn jito_tip_accounts_are_complete_and_unique() {
        assert_eq!(JITO_TIP_ACCOUNTS.len(), 8);
        let unique = JITO_TIP_ACCOUNTS.iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), JITO_TIP_ACCOUNTS.len());
    }

    #[test]
    fn default_provider_accounts_deduplicate_with_manual_accounts() {
        let mut accounts = vec![
            "payer".to_string(),
            JITO_TIP_ACCOUNTS[0].to_string(),
            "payer".to_string(),
        ];
        add_default_provider_observation_accounts(&mut accounts);

        assert_eq!(
            accounts
                .iter()
                .filter(|account| account.as_str() == "payer")
                .count(),
            1
        );
        assert!(JITO_TIP_ACCOUNTS
            .iter()
            .all(|account| accounts.iter().any(|value| value == account)));
    }
}
