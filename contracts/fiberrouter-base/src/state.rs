use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

/// Store the owner of the contract to set pool
pub const OWNER: Item<Addr> = Item::new("owner");
/// Store the contract address of multiswap pool
pub const POOL: Item<Addr> = Item::new("pool");
