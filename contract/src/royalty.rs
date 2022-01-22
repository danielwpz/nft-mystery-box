use crate::*;
use near_sdk::{
    AccountId, require, Balance,
    borsh::{self, BorshSerialize, BorshDeserialize},
};
use std::collections::HashMap;

type Percentage = u16;
const PERCENTAGE_BASIS: Percentage = 10_000;

pub type RoyaltyMap = HashMap<AccountId, Percentage>;
type Payouts = HashMap<AccountId, Balance>;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Royalty {
    royalties: RoyaltyMap,
}

impl Royalty {
    pub fn new(
        royalties: RoyaltyMap,
    ) -> Self {
        Self::assert_valid_royalties(&royalties);

        Self {
            royalties: royalties
        }
    }

    fn assert_valid_royalties(
        royalties: &RoyaltyMap
    ) {
        // sum should be less than ROYALTY_BASIS 
        let sum: Percentage = royalties.values().sum();
        require!(
            sum <= PERCENTAGE_BASIS,
            error::ERR_BAD_ROYALTY_VALUE
        );
    }

    pub fn calculate_payouts(
        &self,
        amount: Balance
    ) -> Payouts {
        let payouts: Payouts = self.royalties.iter().map(|(account, percent)| {
            let pay_amount = Balance::from(*percent) * amount 
                / Balance::from(PERCENTAGE_BASIS);
            (account.clone(), pay_amount)
        })
        .collect();
        return payouts;
    }
}
