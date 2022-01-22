use crate::raffle::Raffle;
use crate::royalty::{Royalty, RoyaltyMap};
use crate::constant::*;
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    near_bindgen, env, PanicOnDefault, Promise, PromiseOrValue,
    BorshStorageKey, ext_contract, assert_one_yocto,
    AccountId,
};
use near_contract_standards::non_fungible_token::{
    NonFungibleToken, TokenId, Token,
    metadata::{
        NFTContractMetadata,
        NonFungibleTokenMetadataProvider,
    }
};
use std::collections::HashMap;

mod error;
mod constant;
mod util;
mod event;
mod raffle;
mod mint;
mod buy;
mod royalty;

#[ext_contract(ext_nft_receiver)]
trait NonFungibleTokenReceiver {
    /// Returns `true` if the token should be returned back to the sender.
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> Promise;
}

#[ext_contract(ext_self)]
trait NonFungibleTokenResolver {
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: NFTContractMetadata,

    raffle: Raffle,

    royalties: Option<Royalty>,
}

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    OwnerById,
    TokenMetadata,
    Enumeration,
    Approval,
    Raffle,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(
        metadata: NFTContractMetadata,
        len: u64,
        royalties: Option<RoyaltyMap>,
    ) -> Self {
        metadata.assert_valid();
        let owner_id = env::predecessor_account_id();

        Self {
            tokens: NonFungibleToken::new(
                StorageKey::OwnerById,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval)
            ),
            metadata: metadata,
            raffle: Raffle::new(StorageKey::Raffle, len),
            royalties: royalties.map(|r| Royalty::new(r) ),
        }
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.clone()
    }
}

impl Contract {
    /// override default nft transfer to log evnet
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
    ) {
        let old_owner_id = self.tokens.owner_by_id.get(&token_id)
            .expect(error::ERR_TOKEN_NOT_EXIST);

        let sender_id = env::predecessor_account_id();
        let authorized_id : Option<AccountId> = if sender_id != old_owner_id {
            Some(sender_id.clone())
        } else {
            None
        };

        self.tokens.nft_transfer(
            receiver_id.clone(),
            token_id.clone(),
            approval_id,
            memo.clone()
        );

        event::NearEvent::log_nft_transfer(
            old_owner_id.to_string(),
            receiver_id.to_string(),
            vec![token_id],
            memo,
            authorized_id.map(|id| id.to_string())
        );
    }

    pub fn nft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        approval_id: Option<u64>,
        memo: Option<String>,
        msg: String,
    ) -> PromiseOrValue<bool> {
        assert_one_yocto();

        // this is to make sure error logs are consistant
        let old_owner_id = self.tokens.owner_by_id.get(&token_id)
            .expect(error::ERR_TOKEN_NOT_EXIST);

        let sender_id = env::predecessor_account_id();
        let (_, old_approvals) = self.tokens.internal_transfer(
            &sender_id,
            &receiver_id,
            &token_id,
            approval_id,
            memo.clone()
        );

        ext_nft_receiver::nft_on_transfer(
            sender_id,
            old_owner_id.clone(),
            token_id.clone(),
            msg,
            receiver_id.clone(),
            NO_DEPOSIT,
            env::prepaid_gas() 
                - GAS_FOR_NFT_TRANSFER_CALL 
                - GAS_FOR_RESOLVE_TRANSFER,
        )
        .then(ext_self::nft_resolve_transfer(
            old_owner_id,
            receiver_id.into(),
            token_id,
            old_approvals,
            env::current_account_id(),
            NO_DEPOSIT,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
        .into()
    }
}

#[near_bindgen]
impl Contract {
    #[allow(dead_code)]
    fn nft_resolve_transfer(
        &mut self,
        previous_owner_id: AccountId,
        receiver_id: AccountId,
        token_id: TokenId,
        approved_account_ids: Option<HashMap<AccountId, u64>>,
    ) -> bool {
        let success = self.tokens.nft_resolve_transfer(
            previous_owner_id.clone(),
            receiver_id.clone(),
            token_id.clone(),
            approved_account_ids
        );

        if success {
            event::NearEvent::log_nft_transfer(
                previous_owner_id.to_string(),
                receiver_id.to_string(),
                vec![token_id],
                None,
                None
            );
        }

        return success;
    }
}
