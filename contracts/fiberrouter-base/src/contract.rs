use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coins, to_binary, Addr, Api, BankMsg, Binary, Coin, CosmosMsg, Deps, DepsMut, Env, MessageInfo,
    Order, Response, StdError, StdResult, Storage, Uint128,
};
use cw_storage_plus::Bound;

use fiberrouter::{
    FiberRouterExecuteMsg, FiberRouterQueryMsg, MigrateMsg, SetPoolEvent, TransferOwnershipEvent,
};
use multiswap::{MultiswapContract, MultiswapExecuteMsg};

use crate::error::{self, ContractError};
use crate::msg::InstantiateMsg;
use crate::state::{OWNER, POOL};
use cw_utils::Event;

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:fiberrouter-base";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let owner = deps.api.addr_validate(&msg.owner)?;
    OWNER.save(deps.storage, &owner)?;
    let pool = deps.api.addr_validate(&msg.pool)?;
    POOL.save(deps.storage, &pool)?;
    Ok(Response::default())
}

/// To mitigate clippy::too_many_arguments warning
pub struct ExecuteEnv<'a> {
    deps: DepsMut<'a>,
    env: Env,
    info: MessageInfo,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: FiberRouterExecuteMsg,
) -> Result<Response, ContractError> {
    let env = ExecuteEnv { deps, env, info };
    match msg {
        FiberRouterExecuteMsg::TransferOwnership { new_owner } => {
            execute_ownership_transfer(env, new_owner)
        }
        FiberRouterExecuteMsg::SetPool { pool } => execute_set_pool(env, pool),
        FiberRouterExecuteMsg::WithdrawSigned {
            payee,
            token,
            amount,
            salt,
            signature,
        } => execute_withdraw_signed(env, payee, token, amount, salt, signature),
        FiberRouterExecuteMsg::Swap {
            token,
            amount,
            target_chain_id,
            target_token,
            target_address,
        } => execute_swap(
            env,
            token,
            amount,
            target_chain_id,
            target_token,
            target_address,
        ),
    }
}

pub fn execute_ownership_transfer(
    env: ExecuteEnv,
    new_owner: String,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env, info } = env;
    let new_owner_addr = deps.api.addr_validate(&new_owner)?;

    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut rsp = Response::default();
    OWNER.save(deps.storage, &new_owner_addr)?;

    let event = TransferOwnershipEvent {
        prev_owner: info.sender.as_str(),
        new_owner: new_owner.as_str(),
    };
    event.add_attributes(&mut rsp);
    Ok(rsp)
}

pub fn execute_set_pool(env: ExecuteEnv, new_pool: String) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env, info } = env;
    let new_pool_addr = deps.api.addr_validate(&new_pool)?;

    if info.sender != OWNER.load(deps.storage)? {
        return Err(ContractError::Unauthorized {});
    }

    let mut rsp = Response::default();
    POOL.save(deps.storage, &new_pool_addr)?;

    let event = SetPoolEvent {
        from: info.sender.as_str(),
        pool: new_pool.as_str(),
    };
    event.add_attributes(&mut rsp);
    Ok(rsp)
}

pub fn execute_withdraw_signed(
    env: ExecuteEnv,
    payee: String,
    token: String,
    amount: Uint128,
    salt: String,
    signature: String,
) -> Result<Response, ContractError> {
    let deps = env.deps;
    let pool = POOL.load(deps.storage)?;
    let contract_addr = deps.api.addr_validate(pool.as_str())?;
    // MultiswapContract is a function helper that provides several queries and message builder.
    let multiswap = MultiswapContract(contract_addr);
    // Call multiswap withdraw signed
    let msg = multiswap.call(
        MultiswapExecuteMsg::WithdrawSigned {
            payee: payee.to_string(),
            token: token.to_string(),
            amount: amount.clone(),
            salt: salt.to_string(),
            signature: signature.to_string(),
        },
        vec![],
    )?;

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "withdraw_signed")
        .add_attribute("payee", payee.to_string())
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string());
    Ok(res)
}

pub fn execute_swap(
    env: ExecuteEnv,
    token: String,
    amount: Uint128,
    target_chain_id: String,
    target_token: String,
    target_address: String,
) -> Result<Response, ContractError> {
    let ExecuteEnv { deps, env, info } = env;
    let pool = POOL.load(deps.storage)?;
    let contract_addr = deps.api.addr_validate(pool.as_str())?;
    // MultiswapContract is a function helper that provides several queries and message builder.
    let multiswap = MultiswapContract(contract_addr);
    // Call multiswap swap
    let msg = multiswap.call(
        MultiswapExecuteMsg::Swap {
            token: token.to_string(),
            amount: amount.clone(),
            target_chain_id: target_chain_id.to_string(),
            target_token: target_token.to_string(),
            target_address: target_address.to_string(),
        },
        info.funds,
    )?;

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "swap")
        .add_attribute("token", token.to_string())
        .add_attribute("amount", amount.to_string())
        .add_attribute("target_chain_id", target_chain_id.to_string())
        .add_attribute("target_token", target_token.to_string())
        .add_attribute("target_address", target_address.to_string());
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: FiberRouterQueryMsg) -> StdResult<Binary> {
    match msg {
        FiberRouterQueryMsg::Owner {} => to_binary(&query_owner(deps)?),
        FiberRouterQueryMsg::Pool {} => to_binary(&query_pool(deps)?),
    }
}

pub fn query_owner(deps: Deps) -> StdResult<String> {
    let owner = OWNER.load(deps.storage)?;
    return Ok(owner.to_string());
}

pub fn query_pool(deps: Deps) -> StdResult<String> {
    let pool = POOL.load(deps.storage)?;
    return Ok(pool.to_string());
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}
