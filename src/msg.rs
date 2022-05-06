use crate::state::{Config, Token};

use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct InstantiateMsg {
    pub admin: String,
    pub allowed_native: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Buy buys nft using native token
    Buy {
        /// recipient if None, tx sender is used
        recipient: Option<String>,
        contract_id: String,
        token_id: String,
    },
    /// ListToken registers or relists tokens
    ListToken {
        token: Token,
    },
    /// Delist tokens removes tokens from marketplace
    DelistToken {
        token_key: String,
    },
    UpdatePrice {
        token_key: String,
        price: Uint128,
    },
    UpdateConfig {
        admin: Option<String>,
        allowed_native: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Token {
        token_key: String,
    },
    RangeTokens {
        start_after: Option<String>,
        limit: Option<u32>,
    },
    ListTokens {
        ids: Vec<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokenResponse {
    pub token: Token,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct TokensResponse {
    pub tokens: Vec<Token>,
}
