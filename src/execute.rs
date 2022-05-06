use crate::msg::{ExecuteMsg, InstantiateMsg};
use crate::state::{Config, Token, CONFIG, TOKENS, Metadata};
use crate::ContractError;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, BankMsg, Coin, CosmosMsg, DepsMut, Env, MessageInfo, Response, SubMsg, Uint128,
    WasmMsg,
};
use cw2::set_contract_version;
use cw721::NftInfoResponse;

const CONTRACT_NAME: &str = "crates.io:cw721-native-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    let cfg = Config {
        admin: deps.api.addr_validate(msg.admin.as_str())?,
        allowed_native: msg.allowed_native,
    };
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Buy {
            recipient,
            contract_id,
            token_id
        } => execute_buy(deps, env, info, recipient, contract_id, token_id),
        ExecuteMsg::ListToken { token } => execute_list_token(deps, env, info, token),
        ExecuteMsg::DelistToken { token_key } => execute_delist_token(deps, env, info, token_key),
        ExecuteMsg::UpdatePrice { token_key, price } => {
            execute_update_price(deps, env, info, token_key, price)
        }
        ExecuteMsg::UpdateConfig {
            admin,
            allowed_native,
        } => execute_update_config(deps, env, info, admin, allowed_native),
    }
}

pub fn execute_list_token(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    mut token: Token,
) -> Result<Response, ContractError> {
    let nft_contract = cw721_base::helpers::Cw721Contract(deps.api.addr_validate(token.contract_id.as_str())?.clone());

    if token.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    nft_contract
        .approval(
            &deps.querier,
            token.token_id.clone(),
            env.contract.address.clone().into_string(),
            None,
        )
        .map_err(|_e| ContractError::NotApproved {})?;

    let token_info:NftInfoResponse<Metadata> = nft_contract
        .nft_info(
            &deps.querier,
            token.token_id.clone(),
        )
        .map_err(|_e| ContractError::NotFound {})?;

    token.extension = Some(token_info.extension);
    token.token_uri = token_info.token_uri;
    token.on_sale = true;

    TOKENS.save(deps.storage, token.get_key(), &token)?;

    let res = Response::new()
        .add_attribute("token", format!("token{:?}", token.get_key()))
        .add_attribute("action", "list_token");

    Ok(res)
}

pub fn execute_delist_token(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_key: String,
) -> Result<Response, ContractError> {
    let mut token = TOKENS.load(deps.storage, token_key.clone())?;
    if token.owner != info.sender {
        return Err(ContractError::Unauthorized {});
    }
    token.on_sale = false;
    TOKENS.remove(deps.storage, token_key.clone());

    let res = Response::new()
        .add_attribute("token", format!("token{:?}", token_key))
        .add_attribute("action", "delist_token");
    Ok(res)
}

pub fn execute_buy(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    recipient_opt: Option<String>,
    contract_id: String,
    token_id: String
) -> Result<Response, ContractError> {
    let cfg = CONFIG.load(deps.storage)?;
    if info.funds.len() != 1 {
        return Err(ContractError::SendSingleNativeToken {});
    }
    let sent_fund = info.funds.get(0).unwrap();
    if sent_fund.denom != cfg.allowed_native {
        return Err(ContractError::NativeDenomNotAllowed {
            denom: sent_fund.clone().denom,
        });
    }

    let recipient = match recipient_opt {
        None => Ok(info.sender),
        Some(r) => deps.api.addr_validate(&r),
    }?;

    let token_key = format!("{}/{}", contract_id, token_id);

    let mut nft_token = TOKENS.load(deps.storage, token_key.clone())?;

    // check if nft is on sale
    if !nft_token.on_sale {
        return Err(ContractError::NftNotOnSale {});
    }

    if nft_token.price != sent_fund.amount {
        return Err(ContractError::InsufficientBalance {
            need: nft_token.price,
            sent: sent_fund.amount,
        });
    }

    // now we can buy
    let send_msg = cw721::Cw721ExecuteMsg::TransferNft {
        recipient: recipient.clone().into_string(),
        token_id: token_id.clone(),
    };

    let msg: CosmosMsg = WasmMsg::Execute {
        contract_addr: contract_id.clone(),
        msg: to_binary(&send_msg)?,
        funds: vec![],
    }
    .into();

    // payout
    let bank_send_msg = BankMsg::Send {
        to_address: nft_token.owner.into_string(),
        amount: vec![Coin {
            denom: cfg.allowed_native,
            amount: nft_token.price,
        }],
    };

    nft_token.owner = recipient.clone();
    nft_token.on_sale = false;

    TOKENS.remove(deps.storage, token_key);

    let res = Response::new()
        .add_submessage(SubMsg::new(msg))
        .add_message(bank_send_msg)
        .add_attribute("action", "buy_native")
        .add_attribute("token_id", token_id)
        .add_attribute("recipient", recipient.to_string())
        .add_attribute("price", nft_token.price);

    Ok(res)
}

pub fn execute_update_price(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    token_key: String,
    price: Uint128,
) -> Result<Response, ContractError> {
    TOKENS.update(deps.storage, token_key.clone(), |exists| match exists {
        None => Err(ContractError::NotFound {}),
        Some(mut token) => {
            if token.owner != info.sender {
                return Err(ContractError::Unauthorized {});
            }
            token.price = price;
            Ok(token)
        }
    })?;

    Ok(Response::new()
        .add_attribute("action", "update_price")
        .add_attribute("token_key", token_key)
        .add_attribute("price", price))
}

pub fn execute_update_config(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    admin: Option<String>,
    allowed_native: Option<String>,
) -> Result<Response, ContractError> {
    let mut cfg = CONFIG.load(deps.storage)?;
    if cfg.admin != info.sender {
        return Err(ContractError::Unauthorized {});
    }

    if let Some(admin) = admin {
        cfg.admin = deps.api.addr_validate(&admin)?
    }

    if let Some(allowed_native) = allowed_native {
        cfg.allowed_native = allowed_native
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}
