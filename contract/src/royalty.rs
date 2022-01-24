use crate::*;
use near_sdk::{
    AccountId, require, Balance, json_types::U128,
    borsh::{self, BorshSerialize, BorshDeserialize},
    serde::{Serialize, Deserialize}, assert_one_yocto,
};
use std::collections::HashMap;

pub type Percentage = u16;
pub const PERCENTAGE_BASIS: Percentage = 10_000;

/// --- Royalty
/// This is a map which shows how the royalty part
/// should be splitted, all in PERCENTAGEs.
/// (royalty part itself is a percentage of total amount
/// of tokens a use paid)
/// e.g. let say the royalty part is 10% of the total payment
/// and there are two parties in royalty, alice and bob.
/// each has 50% royalty rate.
/// then if the token was sold at 100N, both alice and bob
/// will receive 5N. (100N * 10% * 50%)

pub type RoyaltyMap = HashMap<AccountId, Percentage>;
#[derive(BorshSerialize, BorshDeserialize)]
pub struct Royalty {
    royalties: RoyaltyMap,
    royalty_rate: Percentage,
}

impl Royalty {
    pub fn new(
        royalties: RoyaltyMap,
        royalty_rate: Percentage,
    ) -> Self {
        Self::assert_valid_royalties(&royalties, royalty_rate);

        Self {
            royalties: royalties,
            royalty_rate: royalty_rate,
        }
    }

    pub fn get_royalties(
        &self
    ) -> &RoyaltyMap {
        &self.royalties
    }

    fn assert_valid_royalties(
        royalties: &RoyaltyMap,
        royalty_rate: Percentage,
    ) {
        require!(
            royalty_rate <= PERCENTAGE_BASIS,
            error::ERR_BAD_ROYALTY_RATE
        );

        // should have no more than 10 accounts
        require!(
            royalties.len() <= 10,
            error::ERR_TOO_MANY_ROYALTY_ACCOUNT
        );

        // sum should be exactly equal to ROYALTY_BASIS 
        let sum: Percentage = royalties.values().sum();
        require!(
            sum == PERCENTAGE_BASIS,
            error::ERR_BAD_ROYALTY_VALUE
        );
    }
}

/// --- Payout
/// Payout shows how many tokens should be paid to each
/// party.
/// payout is about EXACT tokens!
/// ALL parties should be considered (artist, creator, DAO, seller, etc)

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Payout {
  pub payout: HashMap<AccountId, U128>,
}

impl Payout {
    pub fn calculate_payout(
        total: Balance,
        beneficiary_id: &AccountId,
        royalties: &RoyaltyMap,
        royalty_rate: Percentage,
    ) -> Self {
        require!(
            royalty_rate <= PERCENTAGE_BASIS,
            error::ERR_BAD_ROYALTY_RATE
        );

        let amount_for_royalty = apply_percent(total, &royalty_rate);
        let mut amount_for_beneficiary = total - amount_for_royalty;

        // calculate payout for royalties first
        let mut payouts: HashMap<AccountId, U128> = royalties.iter().map(|(account, percent)| {
            let amount = apply_percent(amount_for_royalty, percent);
            (account.clone(), amount.into())
        })
        .collect();

        // beneficiary could also be listed in royalty
        // so we need to update his amount again
        amount_for_beneficiary +=
            payouts.get(beneficiary_id).map_or(0, |x| x.0);

        // insert payout for beneficary
        payouts.insert(beneficiary_id.clone(), amount_for_beneficiary.into());

        Payout {
            payout: payouts
        }
    }
}

fn apply_percent(
    amount: Balance,
    percent: &Percentage
) -> Balance {
    amount * Balance::from(*percent) / Balance::from(PERCENTAGE_BASIS)
}

/// --- NEP 199

pub trait NFTPayouts {
    /// Given a `token_id` and NEAR-denominated balance, return the `Payout`.
    /// struct for the given token. Panic if the length of the payout exceeds
    /// `max_len_payout.`
    fn nft_payout(&self, token_id: String, balance: U128, max_len_payout: Option<u32>) -> Payout;
    /// Given a `token_id` and NEAR-denominated balance, transfer the token
    /// and return the `Payout` struct for the given token. Panic if the
    /// length of the payout exceeds `max_len_payout.`
    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout;
}

#[near_bindgen]
impl NFTPayouts for Contract {
    fn nft_payout(
        &self, 
        token_id: String, 
        balance: U128, 
        max_len_payout: Option<u32>
    ) -> Payout {
        let owner_id = self.owner_of(&token_id)
            .expect(error::ERR_TOKEN_NOT_EXIST);

        let payouts = self.royalty.as_ref().map_or(
            Payout::default(),
            |royalty| {
                Payout::calculate_payout(
                    balance.into(),
                    &owner_id,
                    &royalty.royalties,
                    royalty.royalty_rate,
                )
            }
        );

        if let Some(max_len) = max_len_payout {
            require!(
                payouts.payout.len() <= max_len as usize,
                error::ERR_TOO_MANY_ROYALTY_ACCOUNT
            );
        }

        return payouts;
    }

    fn nft_transfer_payout(
        &mut self,
        receiver_id: AccountId,
        token_id: String,
        approval_id: Option<u64>,
        memo: Option<String>,
        balance: U128,
        max_len_payout: Option<u32>,
    ) -> Payout {
        assert_one_yocto();
        let payouts = self.nft_payout(
            token_id.clone(),
            balance,
            max_len_payout
        );
        self.nft_transfer(
            receiver_id.clone(),
            token_id.clone(),
            approval_id.clone(),
            memo.clone()
        );
        return payouts;
    }
}
