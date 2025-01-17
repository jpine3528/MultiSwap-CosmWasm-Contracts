use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, Uint128};

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MultiswapExecuteMsg {
    TransferOwnership {
        new_owner: String,
    },
    AddSigner {
        signer: String,
    },
    RemoveSigner {
        signer: String,
    },
    AddFoundryAsset {
        token: String,
    },
    RemoveFoundryAsset {
        token: String,
    },
    AddLiquidity {
        token: String,
        amount: Uint128,
    },
    RemoveLiquidity {
        token: String,
        amount: Uint128,
    },
    WithdrawSigned {
        payee: String,
        salt: String,
        token: String,
        amount: Uint128,
        signature: String,
    },
    Swap {
        token: String,
        amount: Uint128,
        target_chain_id: String,
        target_token: String,
        target_address: String,
    },
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct MigrateMsg {}
