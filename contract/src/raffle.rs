use near_sdk::{
    borsh::{self, BorshSerialize, BorshDeserialize},
    collections::{LookupMap},
    IntoStorageKey, require,
};

use crate::error::*;
use crate::util::get_random_number;


#[derive(BorshSerialize, BorshDeserialize)]
pub struct Raffle {
    len: u64,
    items: LookupMap<u64, u64>,
}

impl Raffle {
    pub fn new<S>(
        items_key_prefix: S,
        len: u64,
    ) -> Self 
    where
        S: IntoStorageKey,
    {
        Self {
            len: len,
            items: LookupMap::new(items_key_prefix)
        }
    }

    pub fn items_left(&self) -> u64 {
        self.len
    }

    pub fn draw(&mut self) -> u64 {
        require!(self.len > 0, ERR_NO_ENOUGH_ITEMS);

        let i = get_random_number(self.len);
        let result = self.get_item(i);

        self.set_item(i, self.get_item(self.len - 1));

        self.len -= 1;

        return result;
    }

    fn set_item(&mut self, key: u64, value: u64) {
        self.items.insert(&key, &value);
    }

    fn get_item(&self, key: u64) -> u64 {
        self.items.get(&key).unwrap_or(key)
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use super::Raffle;

    #[test]
    pub fn test() {
        // run test_raffle for many times to make sure it's correct
        for k in 0..10 {
            test_raffle(k);
        }
    }

    fn test_raffle(prefix: u8) {
        const N: u64 = 20;
        let mut raffle = Raffle::new(prefix, N);
        let mut results: Vec<u64> = Vec::new();
        for _ in 0..N {
            let id = raffle.draw();
            results.push(id);
        }

        // raffle should be empty
        assert_eq!(0, raffle.items_left());

        // check if all number are in results
        for i in 0..N {
            if !results.contains(&i) {
                println!("{:?}", results);
                println!("{}", i);
            }
            assert!(results.contains(&i));
        }
    }
}
