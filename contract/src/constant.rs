use near_sdk::{
    Balance, Gas,
};
use near_units::parse_gas;

pub const NO_DEPOSIT: Balance = 0;

pub const GAS_FOR_RESOLVE_TRANSFER: Gas = Gas(parse_gas!("10 TGas") as u64);
pub const GAS_FOR_NFT_TRANSFER_CALL: Gas = Gas(parse_gas!("30 TGas") as u64);
