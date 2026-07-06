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

/// Helius Sender designated tip accounts (mainnet-beta). Included in observation
/// filters so tip transfers to Helius Sender are matchable in benchmark streams.
pub const HELIUS_SENDER_TIP_ACCOUNTS: &[&str; 10] = &[
    "4ACfpUFoaSD9bfPdeu6DBt89gB6ENTeHBXCAi87NhDEE",
    "D2L6yPZ2FmmmTKPgzaMKdhu6EWZcTpLy1Vhx8uvZe7NZ",
    "9bnz4RShgq1hAnLnZbP8kbgBg1kEmcJBYQq3gQbmnSta",
    "5VY91ws6B2hMmBFRsXkoAAdsPHBJwRfBht4DXox3xkwn",
    "2nyhqdwKcJZR2vcqCyrYsaPVdAnFoJjiksCXJ7hfEYgD",
    "2q5pghRs6arqVjRvT5gfgWfWcHWmw1ZuCzphgd5KfWGJ",
    "wyvPkWjVZz1M8fHQnMMCDTQDbkManefNNhweYk5WkcF",
    "3KCKozbAaF75qEU33jtzozcJ29yJuaLJTy2jFdzUY8bT",
    "4vieeGHPYPG2MmyPRcYjdiDmmhN3ww7hsFNap8pVN3Ey",
    "4TQLFNWK8AovT1gFvda5jfw2oJeRMKEmw7aH6MGBJ3or",
];

/// Astralane Iris tip account.
pub const ASTRALANE_TIP_ACCOUNT: &str = "astra4uejePWneqNaJKuFFA8oonqCE1sqF6b45kDMZm";

pub fn add_default_provider_observation_accounts(accounts: &mut Vec<String>) {
    accounts.extend(
        JITO_TIP_ACCOUNTS
            .iter()
            .map(|account| (*account).to_string()),
    );
    accounts.extend(
        HELIUS_SENDER_TIP_ACCOUNTS
            .iter()
            .map(|account| (*account).to_string()),
    );
    accounts.push(ASTRALANE_TIP_ACCOUNT.to_string());
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

    #[test]
    fn helius_sender_tip_accounts_are_unique() {
        let unique = HELIUS_SENDER_TIP_ACCOUNTS.iter().collect::<BTreeSet<_>>();
        assert_eq!(unique.len(), HELIUS_SENDER_TIP_ACCOUNTS.len());
    }

    #[test]
    fn external_sender_tip_accounts_are_registered() {
        let mut accounts = vec!["payer".to_string()];
        add_default_provider_observation_accounts(&mut accounts);
        for account in HELIUS_SENDER_TIP_ACCOUNTS {
            assert!(accounts.iter().any(|value| value == account));
        }
        assert!(accounts.iter().any(|value| value == ASTRALANE_TIP_ACCOUNT));
    }
}
