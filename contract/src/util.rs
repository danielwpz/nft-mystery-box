use near_sdk::{
    env,
};
use core::convert::TryInto;

pub fn get_random_number(n: u64) -> u64 {
    let mut seed = env::random_seed();
    let seed_len = seed.len();
    let mut arr: [u8; 8] = Default::default();
    seed.rotate_left(0 as usize % seed_len);
    arr.copy_from_slice(&seed[..8]);
    let r: u64 = u64::from_le_bytes(arr).try_into().unwrap();
    return r % n;
}
