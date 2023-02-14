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

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg id='Layer_2' data-name='Layer 2' xmlns='http://www.w3.org/2000/svg' xmlns:xlink='http://www.w3.org/1999/xlink' viewBox='0 0 71.68 59.2'%3E%3Cdefs%3E%3ClinearGradient id='linear-gradient' x1='35.84' y1='56.93' x2='35.84' y2='-1.76' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0' stop-color='#0e0c0d'/%3E%3Cstop offset='.54' stop-color='#271f1b'/%3E%3Cstop offset='.99' stop-color='#40332a'/%3E%3C/linearGradient%3E%3ClinearGradient id='linear-gradient-2' x1='35.84' y1='54.45' x2='35.84' y2='1.09' xlink:href='#linear-gradient'/%3E%3ClinearGradient id='linear-gradient-3' x1='35.84' y1='54.88' x2='35.84' y2='14.08' xlink:href='#linear-gradient'/%3E%3ClinearGradient id='linear-gradient-4' x1='9.62' y1='19.29' x2='9.62' y2='26' gradientUnits='userSpaceOnUse'%3E%3Cstop offset='0' stop-color='#f1f1f2'/%3E%3Cstop offset='1' stop-color='#03b5b0'/%3E%3C/linearGradient%3E%3ClinearGradient id='linear-gradient-5' x1='17.13' x2='17.13' xlink:href='#linear-gradient-4'/%3E%3ClinearGradient id='linear-gradient-6' x1='24.53' x2='24.53' xlink:href='#linear-gradient-4'/%3E%3ClinearGradient id='linear-gradient-7' x1='32.28' y1='19.3' x2='32.28' xlink:href='#linear-gradient-4'/%3E%3ClinearGradient id='linear-gradient-8' x1='41.2' x2='41.2' xlink:href='#linear-gradient-4'/%3E%3ClinearGradient id='linear-gradient-9' x1='51.45' x2='51.45' xlink:href='#linear-gradient-4'/%3E%3ClinearGradient id='linear-gradient-10' x1='61.64' x2='61.64' xlink:href='#linear-gradient-4'/%3E%3C/defs%3E%3Cg id='Layer_1-2' data-name='Layer 1'%3E%3Ccircle cx='35.84' cy='29.6' r='29.35' style='fill: url(#linear-gradient); stroke: #7f6c60; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Ccircle cx='35.84' cy='29.6' r='26.68' style='fill: url(#linear-gradient-2); stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Ccircle cx='35.84' cy='29.6' r='24.08' style='fill: #9c9b92;'/%3E%3Cpath d='m71.43,29.6c0-5.02-1.04-9.79-2.92-14.12H3.17C1.29,19.8.25,24.58.25,29.6s1.08,9.99,3.03,14.39h10.08c4.74,7.4,13.04,12.3,22.48,12.3s17.73-4.9,22.48-12.3h10.08c1.95-4.4,3.03-9.27,3.03-14.39Z' style='fill: url(#linear-gradient-3); stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Cg%3E%3Cg%3E%3Cpath d='m11.51,19.29h1.46v6.71h-1.49l-3.77-4.49v4.49h-1.44v-6.71h1.49l3.75,4.47v-4.47Z' style='fill: url(#linear-gradient-4);'/%3E%3Cpath d='m20.23,20.74h-4.73v1.17h3.81v1.45h-3.81v1.17h4.73v1.45h-6.19v-6.71h6.19v1.45Z' style='fill: url(#linear-gradient-5);'/%3E%3Cpath d='m26.49,19.29c.76,0,1.39.61,1.39,1.38v5.33h-1.46v-2.16h-3.8v2.16h-1.44v-5.33c0-.76.61-1.38,1.38-1.38h3.94Zm-.07,3.09v-1.64h-3.8v1.64h3.8Z' style='fill: url(#linear-gradient-6);'/%3E%3Cpath d='m35.64,22.41c0,.75-.62,1.38-1.39,1.38h-.07c.48.56.99,1.16,1.45,1.71v.49h-1.49l-1.85-2.21h-1.92s.03.03.03.07c0,0-.02,0-.03,0v2.15h-1.44v-6.7h5.32c.76,0,1.39.62,1.39,1.38v1.73Zm-5.26-.07h3.8v-1.59h-3.8v1.59Z' style='fill: url(#linear-gradient-7);'/%3E%3C/g%3E%3Cg%3E%3Cpath d='m45.76,19.29v6.71h-1.23v-2.74h-6.66v2.74h-1.23v-6.71h1.23v2.74h6.66v-2.74h1.23Z' style='fill: url(#linear-gradient-8);'/%3E%3Cpath d='m54.7,19.29h1.24v5.41c0,.72-.59,1.29-1.3,1.29h-6.38c-.72,0-1.29-.58-1.29-1.29v-5.41h1.23v5.41s.03.07.07.07h6.38s.06-.03.06-.07v-5.41Z' style='fill: url(#linear-gradient-9);'/%3E%3Cpath d='m65.85,21.9c0,.18-.03.34-.09.48.21.22.36.55.36.88v1.44c0,.72-.59,1.29-1.3,1.29h-7.67v-6.71h7.41c.71,0,1.29.58,1.29,1.29v1.31Zm-7.41-1.38s-.07.03-.07.07v1.31s.03.07.07.07h6.11s.06-.03.06-.07v-1.31s-.03-.07-.06-.07h-6.11Zm6.44,2.74s-.03-.06-.07-.06h-6.37s-.07.02-.07.06v1.44s.03.07.07.07h6.37s.07-.03.07-.07v-1.44Z' style='fill: url(#linear-gradient-10);'/%3E%3C/g%3E%3C/g%3E%3Cg%3E%3Cpath d='m6.33,28.62h6.04s0,1.01,0,1.01h-4.98s0,4.16,0,4.16h4.64s0,.99,0,.99h-4.64s0,4.43,0,4.43h4.98s0,1.01,0,1.01h-6.04s-.02-11.59-.02-11.59Z' style='fill: #03b5b0;'/%3E%3Cpath d='m20.78,29.09v1.22c-.83-.61-1.81-.92-2.7-.92-1.23,0-2.37.63-2.37,1.94,0,1.14.88,1.69,2.6,2.46,1.8.84,3.08,1.59,3.08,3.35,0,2.12-1.74,3.3-3.7,3.3-1.33,0-2.47-.54-3.16-1.09v-1.33c.8.84,2.01,1.37,3.17,1.37,1.33,0,2.59-.75,2.59-2.16,0-1.19-.88-1.76-2.64-2.57-1.78-.82-3.03-1.49-3.03-3.26,0-2.01,1.62-3.06,3.49-3.07,1.05,0,2.02.34,2.67.75Z' style='fill: #03b5b0;'/%3E%3Cpath d='m26.29,29.6h-3.75s0-1.01,0-1.01h8.57s0,1,0,1h-3.75s.02,10.59.02,10.59h-1.08s-.02-10.59-.02-10.59Z' style='fill: #03b5b0;'/%3E%3Cpath d='m39.94,40.17l-1.49-3.76h-5.19s-1.48,3.77-1.48,3.77h-1.16s4.62-11.61,4.62-11.61h1.19s4.67,11.59,4.67,11.59h-1.17Zm-1.88-4.79l-2.21-5.54-2.19,5.55h4.4Z' style='fill: #03b5b0;'/%3E%3Cpath d='m44.35,29.57h-3.75s0-1.01,0-1.01h8.57s0,1,0,1h-3.75s.02,10.59.02,10.59h-1.08s-.02-10.59-.02-10.59Z' style='fill: #03b5b0;'/%3E%3Cpath d='m51.23,28.55h6.04s0,1.01,0,1.01h-4.98s0,4.16,0,4.16h4.64s0,.99,0,.99h-4.64s0,4.43,0,4.43h4.98s0,1.01,0,1.01h-6.04s-.02-11.59-.02-11.59Z' style='fill: #03b5b0;'/%3E%3Cpath d='m65.68,29.02v1.22c-.83-.61-1.81-.92-2.7-.92-1.23,0-2.37.63-2.37,1.94,0,1.14.88,1.69,2.6,2.46,1.8.84,3.08,1.59,3.08,3.35,0,2.12-1.74,3.3-3.7,3.3-1.33,0-2.47-.54-3.16-1.09v-1.33c.8.84,2.01,1.37,3.17,1.37,1.33,0,2.59-.75,2.59-2.16,0-1.19-.88-1.76-2.64-2.57-1.78-.82-3.03-1.49-3.03-3.26,0-2.01,1.62-3.06,3.49-3.07,1.05,0,2.02.34,2.67.75Z' style='fill: #03b5b0;'/%3E%3C/g%3E%3Cg%3E%3Cg%3E%3Crect x='29.31' y='8.41' width='12.5' height='7.06' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Cpolygon points='43.48 8.41 27.64 8.41 35.56 3.93 43.48 8.41' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Cpath d='m37.02,11.74s-.26-.87-1.46-.87-1.46.87-1.46.87v3.73h2.91v-3.73Z' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Crect x='23.11' y='10.89' width='6.2' height='4.58' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Crect x='41.81' y='10.89' width='6.2' height='4.58' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3C/g%3E%3Cg%3E%3Cline x1='23.43' y1='11.11' x2='29.14' y2='15.15' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Cline x1='41.81' y1='15.47' x2='48.01' y2='10.89' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3Cpolyline points='41.81 8.62 37.02 11.74 41.81 15.15' style='fill: none; stroke: #966e4d; stroke-linejoin: bevel; stroke-width: .5px;'/%3E%3Cpolyline points='29.31 8.52 34.1 11.74 29.31 15.47' style='fill: none; stroke: #966e4d; stroke-linejoin: bevel; stroke-width: .5px;'/%3E%3Cline x1='35.56' y1='4.18' x2='35.56' y2='8.41' style='fill: #0e0c0d; stroke: #966e4d; stroke-miterlimit: 10; stroke-width: .5px;'/%3E%3C/g%3E%3C/g%3E%3Cg%3E%3Cpath d='m20.73,42.72v.4h-.88v1.77h-.4v-1.77h-.88v-.4h2.16Z' style='fill: #fff;'/%3E%3Cpath d='m28.27,42.72c.23,0,.42.19.42.42v1.33c0,.23-.19.42-.42.42h-1.33c-.23,0-.42-.19-.42-.42v-1.33c0-.23.19-.42.42-.42h1.33Zm0,1.77s.02,0,.02-.02v-1.33s0-.02-.02-.02h-1.33s-.02,0-.02.02v1.33s0,.02.02.02h1.33Z' style='fill: #fff;'/%3E%3Cpath d='m36.68,42.72v.1l-.82.98.82.98v.11h-.44l-.74-.88h-.51v.88h-.4v-2.16h.4v.88h.51c.24-.29.5-.6.74-.88h.44Z' style='fill: #fff;'/%3E%3Cpath d='m44.56,43.12h-1.59v.49h1.28v.4h-1.28v.49h1.59v.4h-1.99v-2.16h1.99v.4Z' style='fill: #fff;'/%3E%3Cpath d='m52.21,42.72h.4v2.16h-.43l-1.33-1.59v1.59h-.4v-2.16h.43l1.33,1.59v-1.59Z' style='fill: #fff;'/%3E%3C/g%3E%3Cpath d='m20.13,47.35c4.17,3.73,9.67,5.99,15.71,5.99s11.54-2.27,15.71-5.99h-31.42Z' style='fill: #2a2924;'/%3E%3C/g%3E%3C/svg%3E";

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
