/*!
Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg id='Layer_2' data-name='Layer 2' xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' viewBox='0 0 71.68 59.2'%3E%3Cdefs%3E%3ClinearGradient id='linear-gradient' x1='35.84' y1='56.93' x2='35.84' y2='-1.76' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0' stop-color='%230e0c0d'/%3E%3Cstop offset='.54' stop-color='%23271f1b'/%3E%3Cstop offset='.99' stop-color='%2340332a'/%3E%3C/linearGradient%3E%3ClinearGradient id='linear-gradient-2' x1='35.84' y1='54.45' x2='35.84' y2='1.09' xlink:href='%23linear-gradient'/%3E%3ClinearGradient id='linear-gradient-3' x1='35.84' y1='54.88' x2='35.84' y2='14.08' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0' stop-color='%230e0c0d'/%3E%3Cstop offset='.29' stop-color='%23272322'/%3E%3Cstop offset='.89' stop-color='%23675f59'/%3E%3Cstop offset='.99' stop-color='%23736a63'/%3E%3C/linearGradient%3E%3C/defs%3E%3Cg id='Layer_1-2' data-name='Layer 1'%3E%3Cg%3E%3Ccircle cx='35.84' cy='29.6' r='29.35' style='fill: url(%23linear-gradient);'/%3E%3Cpath d='m35.84,59.2c-16.32,0-29.6-13.28-29.6-29.6S19.52,0,35.84,0s29.6,13.28,29.6,29.6-13.28,29.6-29.6,29.6Zm0-58.7C19.79.5,6.74,13.55,6.74,29.6s13.05,29.1,29.1,29.1,29.1-13.05,29.1-29.1S51.88.5,35.84.5Z' style='fill: %237f6c60;'/%3E%3C/g%3E%3Cg%3E%3Ccircle cx='35.84' cy='29.6' r='26.68' style='fill: url(%23linear-gradient-2);'/%3E%3Cpath d='m35.84,56.53c-14.85,0-26.93-12.08-26.93-26.93S20.99,2.67,35.84,2.67s26.93,12.08,26.93,26.93-12.08,26.93-26.93,26.93Zm0-53.37c-14.58,0-26.43,11.86-26.43,26.43s11.86,26.43,26.43,26.43,26.43-11.86,26.43-26.43S50.42,3.17,35.84,3.17Z' style='fill: %23966e4d;'/%3E%3C/g%3E%3Ccircle cx='35.84' cy='29.6' r='24.08' style='fill: %233e657e;'/%3E%3Cg%3E%3Cg%3E%3Cg%3E%3Crect x='29.31' y='8.41' width='12.5' height='7.06' style='fill: %23c6b673;'/%3E%3Cpath d='m42.06,15.72h-13v-7.56h13v7.56Zm-12.5-.5h12v-6.56h-12v6.56Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Cpolygon points='43.48 8.41 27.64 8.41 35.56 3.93 43.48 8.41' style='fill: %23c6b673;'/%3E%3Cpath d='m44.43,8.66h-17.73l8.87-5.01,8.87,5.01Zm-15.83-.5h13.93l-6.97-3.94-6.97,3.94Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Cpath d='m37.02,11.74s-.26-.87-1.46-.87-1.46.87-1.46.87v3.73h2.91v-3.73Z' style='fill: %230e0c0d;'/%3E%3Cpath d='m37.27,15.72h-3.41v-4.05s.34-1.05,1.71-1.05,1.68,1.01,1.7,1.05v.07s.01,3.98.01,3.98Zm-2.91-.5h2.41v-3.44c-.06-.14-.33-.67-1.21-.67s-1.15.53-1.21.67v3.44Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Crect x='23.11' y='10.89' width='6.2' height='4.58' style='fill: %23c6b673;'/%3E%3Cpath d='m29.56,15.72h-6.7v-5.08h6.7v5.08Zm-6.2-.5h5.7v-4.08h-5.7v4.08Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Crect x='41.81' y='10.89' width='6.2' height='4.58' style='fill: %23c6b673;'/%3E%3Cpath d='m48.26,15.72h-6.7v-5.08h6.7v5.08Zm-6.2-.5h5.7v-4.08h-5.7v4.08Z' style='fill: %23fff;'/%3E%3C/g%3E%3C/g%3E%3Cg%3E%3Cg%3E%3Cline x1='23.43' y1='11.11' x2='29.14' y2='15.15' style='fill: %230e0c0d;'/%3E%3Crect x='26.03' y='9.63' width='.5' height='6.99' transform='translate(.39 27) rotate(-54.72)' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Cline x1='41.81' y1='15.47' x2='48.01' y2='10.89' style='fill: %230e0c0d;'/%3E%3Crect x='41.06' y='12.93' width='7.71' height='.5' transform='translate(.95 29.24) rotate(-36.42)' style='fill: %23fff;'/%3E%3C/g%3E%3Cpolygon points='41.67 15.35 36.87 11.95 36.88 11.53 41.67 8.41 41.95 8.83 37.46 11.75 41.96 14.95 41.67 15.35' style='fill: %23fff;'/%3E%3Cpolygon points='29.47 15.67 29.16 15.28 33.68 11.76 29.17 8.73 29.45 8.31 34.24 11.54 34.26 11.94 29.47 15.67' style='fill: %23fff;'/%3E%3Crect x='35.31' y='4.18' width='.5' height='4.23' style='fill: %23fff;'/%3E%3C/g%3E%3C/g%3E%3Cg%3E%3Cpath d='m71.43,29.6c0-5.02-1.04-9.79-2.92-14.12H3.17C1.29,19.8.25,24.58.25,29.6s1.08,9.99,3.03,14.39h10.08c4.74,7.4,13.04,12.3,22.48,12.3s17.73-4.9,22.48-12.3h10.08c1.95-4.4,3.03-9.27,3.03-14.39Z' style='fill: url(%23linear-gradient-3);'/%3E%3Cpath d='m35.84,56.53c-9.18,0-17.62-4.59-22.61-12.3H3.12l-.07-.15c-2.02-4.58-3.05-9.45-3.05-14.49s.99-9.72,2.94-14.22l.07-.15h65.68l.07.15c1.95,4.5,2.94,9.29,2.94,14.22s-1.03,9.91-3.05,14.49l-.07.15h-10.11c-5,7.7-13.44,12.3-22.61,12.3ZM3.44,43.73h10.06l.07.12c4.89,7.63,13.22,12.18,22.27,12.18s17.37-4.55,22.27-12.18l.07-.12h10.06c1.95-4.47,2.94-9.22,2.94-14.14s-.95-9.48-2.83-13.87H3.33c-1.88,4.39-2.83,9.06-2.83,13.87s.99,9.67,2.94,14.14Z' style='fill: %23966e4d;'/%3E%3C/g%3E%3Cg%3E%3Cg%3E%3Cpath d='m11.51,19.29h1.46v6.71h-1.49l-3.77-4.49v4.49h-1.44v-6.71h1.49l3.75,4.47v-4.47Z' style='fill: %23fff;'/%3E%3Cpath d='m20.23,20.74h-4.73v1.17h3.81v1.45h-3.81v1.17h4.73v1.45h-6.19v-6.71h6.19v1.45Z' style='fill: %23fff;'/%3E%3Cpath d='m26.49,19.29c.76,0,1.39.61,1.39,1.38v5.33h-1.46v-2.16h-3.8v2.16h-1.44v-5.33c0-.76.61-1.38,1.38-1.38h3.94Zm-.07,3.09v-1.64h-3.8v1.64h3.8Z' style='fill: %23fff;'/%3E%3Cpath d='m35.64,22.41c0,.75-.62,1.38-1.39,1.38h-.07c.48.56.99,1.16,1.45,1.71v.49h-1.49l-1.85-2.21h-1.92s.03.03.03.07c0,0-.02,0-.03,0v2.15h-1.44v-6.7h5.32c.76,0,1.39.62,1.39,1.38v1.73Zm-5.26-.07h3.8v-1.59h-3.8v1.59Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Cpath d='m45.76,19.29v6.71h-1.23v-2.74h-6.66v2.74h-1.23v-6.71h1.23v2.74h6.66v-2.74h1.23Z' style='fill: %23fff;'/%3E%3Cpath d='m54.7,19.29h1.24v5.41c0,.72-.59,1.29-1.3,1.29h-6.38c-.72,0-1.29-.58-1.29-1.29v-5.41h1.23v5.41s.03.07.07.07h6.38s.06-.03.06-.07v-5.41Z' style='fill: %23fff;'/%3E%3Cpath d='m65.85,21.9c0,.18-.03.34-.09.48.21.22.36.55.36.88v1.44c0,.72-.59,1.29-1.3,1.29h-7.67v-6.71h7.41c.71,0,1.29.58,1.29,1.29v1.31Zm-7.41-1.38s-.07.03-.07.07v1.31s.03.07.07.07h6.11s.06-.03.06-.07v-1.31s-.03-.07-.06-.07h-6.11Zm6.44,2.74s-.03-.06-.07-.06h-6.37s-.07.02-.07.06v1.44s.03.07.07.07h6.37s.07-.03.07-.07v-1.44Z' style='fill: %23fff;'/%3E%3C/g%3E%3C/g%3E%3Cg%3E%3Cpath d='m6.33,28.62h6.04s0,1.01,0,1.01h-4.98s0,4.16,0,4.16h4.64s0,.99,0,.99h-4.64s0,4.43,0,4.43h4.98s0,1.01,0,1.01h-6.04s-.02-11.59-.02-11.59Z' style='fill: %23fff;'/%3E%3Cpath d='m20.78,29.09v1.22c-.83-.61-1.81-.92-2.7-.92-1.23,0-2.37.63-2.37,1.94,0,1.14.88,1.69,2.6,2.46,1.8.84,3.08,1.59,3.08,3.35,0,2.12-1.74,3.3-3.7,3.3-1.33,0-2.47-.54-3.16-1.09v-1.33c.8.84,2.01,1.37,3.17,1.37,1.33,0,2.59-.75,2.59-2.16,0-1.19-.88-1.76-2.64-2.57-1.78-.82-3.03-1.49-3.03-3.26,0-2.01,1.62-3.06,3.49-3.07,1.05,0,2.02.34,2.67.75Z' style='fill: %23fff;'/%3E%3Cpath d='m26.29,29.6h-3.75s0-1.01,0-1.01h8.57s0,1,0,1h-3.75s.02,10.59.02,10.59h-1.08s-.02-10.59-.02-10.59Z' style='fill: %23fff;'/%3E%3Cpath d='m39.94,40.17l-1.49-3.76h-5.19s-1.48,3.77-1.48,3.77h-1.16s4.62-11.61,4.62-11.61h1.19s4.67,11.59,4.67,11.59h-1.17Zm-1.88-4.79l-2.21-5.54-2.19,5.55h4.4Z' style='fill: %23fff;'/%3E%3Cpath d='m44.35,29.57h-3.75s0-1.01,0-1.01h8.57s0,1,0,1h-3.75s.02,10.59.02,10.59h-1.08s-.02-10.59-.02-10.59Z' style='fill: %23fff;'/%3E%3Cpath d='m51.23,28.55h6.04s0,1.01,0,1.01h-4.98s0,4.16,0,4.16h4.64s0,.99,0,.99h-4.64s0,4.43,0,4.43h4.98s0,1.01,0,1.01h-6.04s-.02-11.59-.02-11.59Z' style='fill: %23fff;'/%3E%3Cpath d='m65.68,29.02v1.22c-.83-.61-1.81-.92-2.7-.92-1.23,0-2.37.63-2.37,1.94,0,1.14.88,1.69,2.6,2.46,1.8.84,3.08,1.59,3.08,3.35,0,2.12-1.74,3.3-3.7,3.3-1.33,0-2.47-.54-3.16-1.09v-1.33c.8.84,2.01,1.37,3.17,1.37,1.33,0,2.59-.75,2.59-2.16,0-1.19-.88-1.76-2.64-2.57-1.78-.82-3.03-1.49-3.03-3.26,0-2.01,1.62-3.06,3.49-3.07,1.05,0,2.02.34,2.67.75Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cg%3E%3Cpath d='m20.73,42.72v.4h-.88v1.77h-.4v-1.77h-.88v-.4h2.16Z' style='fill: %23fff;'/%3E%3Cpath d='m28.27,42.72c.23,0,.42.19.42.42v1.33c0,.23-.19.42-.42.42h-1.33c-.23,0-.42-.19-.42-.42v-1.33c0-.23.19-.42.42-.42h1.33Zm0,1.77s.02,0,.02-.02v-1.33s0-.02-.02-.02h-1.33s-.02,0-.02.02v1.33s0,.02.02.02h1.33Z' style='fill: %23fff;'/%3E%3Cpath d='m36.68,42.72v.1l-.82.98.82.98v.11h-.44l-.74-.88h-.51v.88h-.4v-2.16h.4v.88h.51c.24-.29.5-.6.74-.88h.44Z' style='fill: %23fff;'/%3E%3Cpath d='m44.56,43.12h-1.59v.49h1.28v.4h-1.28v.49h1.59v.4h-1.99v-2.16h1.99v.4Z' style='fill: %23fff;'/%3E%3Cpath d='m52.21,42.72h.4v2.16h-.43l-1.33-1.59v1.59h-.4v-2.16h.43l1.33,1.59v-1.59Z' style='fill: %23fff;'/%3E%3C/g%3E%3Cpath d='m20.13,47.35c4.17,3.73,9.67,5.99,15.71,5.99s11.54-2.27,15.71-5.99h-31.42Z' style='fill: %2384763a;'/%3E%3C/g%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: AccountId, total_supply: U128) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: "NEAR Hub Estates".to_string(),
                symbol: "ESTATES".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    /// Initializes the contract with the given total supply owned by the given `owner_id` with
    /// the given fungible token metadata.
    #[init]
    pub fn new(
        owner_id: AccountId,
        total_supply: U128,
        metadata: FungibleTokenMetadata,
    ) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        near_contract_standards::fungible_token::events::FtMint {
            owner_id: &owner_id,
            amount: &total_supply,
            memo: Some("Initial tokens supply is minted"),
        }
        .emit();
        this
    }

    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into(), TOTAL_SUPPLY.into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.ft_total_supply().0, TOTAL_SUPPLY);
        assert_eq!(contract.ft_balance_of(accounts(1)).0, TOTAL_SUPPLY);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(2).into(), TOTAL_SUPPLY.into());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());
        let transfer_amount = TOTAL_SUPPLY / 3;
        contract.ft_transfer(accounts(1), transfer_amount.into(), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert_eq!(contract.ft_balance_of(accounts(2)).0, (TOTAL_SUPPLY - transfer_amount));
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transfer_amount);
    }
}
