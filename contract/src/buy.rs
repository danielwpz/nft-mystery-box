use crate::*;

use near_sdk::{
    near_bindgen, env, require, Balance, Promise
};
use near_contract_standards::non_fungible_token::{
    Token
};
use near_units::{parse_near};

/// cost related functions
#[near_bindgen]
impl Contract {
    pub fn unit_price(
        &self
    ) -> Balance {
        parse_near!("1N")
    }

    fn mint_cost_for(
        &self,
        n: u64
    ) -> Balance {
        Balance::from(n) * self.unit_price()
    }

    /// return the cost for buying N tokens
    /// including minting + storage cost
    pub fn cost_for(
        &self,
        n: u64
    ) -> Balance {
        let mint_cost = self.mint_cost_for(n);

        let storage_cost_per_token = env::storage_byte_cost() *
            self.tokens.extra_storage_in_bytes_per_token as Balance;
        let storage_cost = storage_cost_per_token * n as Balance;

        return mint_cost + storage_cost;
    }
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn buy(
        &mut self,
        n: u64
    ) -> Vec<Token> {
        require!(self.raffle.items_left() > 0, error::ERR_NO_ENOUGH_ITEMS);

        let init_storage_usage = env::storage_usage();

        let buyer_id = env::signer_account_id();
        let deposit = env::attached_deposit();
        self.assert_deposit(n, deposit);

        // draw and mint tokens
        let tokens = self.mint_many(n, &buyer_id);

        // refund extra storage deposit to buyer
        let deposit_for_storage = deposit - self.mint_cost_for(n);
        self.refund_storage_deposit(
            init_storage_usage,
            deposit_for_storage,
            &buyer_id
        );

        return tokens;
    }

    /// deposit_for_storage = total_deposit - mint_price
    fn refund_storage_deposit(
        & self,
        init_storage_usage: u64,
        deposit_for_storage: Balance,
        account_id: &AccountId
    ) {
        let current_storage_usage = env::storage_usage();
        let storage_cost = env::storage_byte_cost() * 
            Balance::from(current_storage_usage - init_storage_usage);

        require!(
            storage_cost <= deposit_for_storage,
            format!(
                "{} Must attach {} yoctoNEAR to cover storage", 
                error::ERR_NO_ENOUGH_STORAGE_DEPOSIT, 
                storage_cost
            )
        );

        // refund deposit fee to user
        let refund = deposit_for_storage - storage_cost;
        if refund > 1 {
            Promise::new(account_id.clone()).transfer(refund);
        }
    }

    /// assert buyer has enough deposit for buying nfts
    fn assert_deposit(
        &self,
        n: u64,
        deposit: Balance
    ) {
        let cost = self.cost_for(n);

        require!(
            deposit >= cost,
            format!(
                "{} Require {}.",
                error::ERR_NO_ENOUGH_DEPOSIT,
                cost,
            )
        );
    }
}
