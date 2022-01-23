use crate::*;
use near_sdk::{
    near_bindgen, require,
};
use near_contract_standards::non_fungible_token::{
    Token, metadata::TokenMetadata,
};

#[near_bindgen]
impl Contract {
    pub(crate) fn mint_many(
        &mut self,
        n: u64,
        owner_id: &AccountId
    ) -> Vec<Token> {
        let token_ids: Vec<u64> = (0..n)
            .map(|_| self.raffle.draw())
            .collect();

        require!(
            token_ids.len() == n as usize,
            error::ERR_NO_ENOUGH_ITEMS
        );

        let tokens = token_ids.iter()
            .map(|id| self.mint_to(id, owner_id))
            .collect();

        event::NearEvent::log_nft_mint(
            owner_id.to_string(),
            token_ids.iter().map(|id| id.to_string()).collect(),
            None
        );

        return tokens;
    }

    /// Mint NFT to owner
    /// storage refund should be handled by caller
    fn mint_to(
        &mut self,
        token_id: &u64,
        owner_id: &AccountId
    ) -> Token {
        let metadata = TokenMetadata {
            title: Some(token_id.to_string()),
            description: None,
            media: Some("a.png".to_string()),
            media_hash: None,
            extra: None,
            reference: Some("a.json".to_string()),
            reference_hash: None,
            copies: None,
            expires_at: None,
            issued_at: Some(env::block_timestamp().to_string()),
            starts_at: None,
            updated_at: None
        };

        return self.tokens.internal_mint_with_refund(
            token_id.to_string(),
            owner_id.clone(),
            Some(metadata),
            None
        );
    }
}
