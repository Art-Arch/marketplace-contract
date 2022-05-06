use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub allowed_native: String,
}

pub const CONFIG: Item<Config> = Item::new("config");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub owner: Addr,
    pub contract_id: Addr,
    pub token_id: String,
    pub price: Uint128,
    pub on_sale: bool,

    pub extension: Option<Metadata>,
    pub token_uri: Option<String>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata {
    pub animation_url: Option<String>,
    pub attributes: Option<String>,
    pub background_color: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub image: Option<String>,
    pub image_data: Option<String>,
    pub name: Option<String>,
    pub youtube_url: Option<String>,

}

impl Token {
    pub fn get_key(&self) -> String{
        return format!("{}/{}",self.contract_id.as_str(), self.token_id)
    }
}

pub const TOKENS: Map<String, Token> = Map::new("tokens");
