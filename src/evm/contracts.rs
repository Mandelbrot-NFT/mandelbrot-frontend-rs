use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;
use web3::{
    contract::{tokens::Tokenize, Contract, Options},
    types::{Address, H256, U256, TransactionReceipt},
    transports::{eip_1193::Eip1193, Either, Http},
    Web3
};

use super::types::{Field, Metadata};


const FUEL: U256 = U256([0, 0, 0, 0]);
const CALLDATA: &[u8] = &[87, 114, 97, 112, 112, 101, 100, 32, 77, 97, 110, 100, 101, 108, 98, 114, 111, 116, 32, 70, 85, 69, 76, 0, 0, 0, 0, 0, 0, 0, 0, 46, 119, 70, 85, 69, 76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 18];


pub enum Error {
    TokenNotFound,
    NoRightsToBurn, // Only the token owner can burn it
    TokenNotEmpty, // Cannot burn token if it has children
    BidNotFound,
    BidTooLow, // Bid must exceed or equal minimum bid price
    MinimumBidTooLow, // Child's minimum bid has to be at least as much as parent's
    TooManyChildTokens, // A maximum of MAX_CHILDREN child tokens can be minted
    NoRightsToApproveBid, // Only the owner of parent token can approve the bid
    NoRightsToDeleteBid, // Only the bid creator can delete it
    FieldOutside, // Token has to be within the field of its parent
    FieldsOverlap, // Sibling fields cannot overlap
    FieldTooLarge, // Token's field cannot exceed MAXIMUM_FIELD_PORTION % of its parent's
    Other(String),
}

impl Error {
    fn from_code(code: &str, message: &str) -> Self {
        match code {
            "0xcbdb7b30" => Self::TokenNotFound,
            "0x5c7e3cf7" => Self::NoRightsToBurn,
            "0xc4b840d3" => Self::TokenNotEmpty,
            "0x3f077648" => Self::BidNotFound,
            "0xa0d26eb6" => Self::BidTooLow,
            "0x7b909a70" => Self::MinimumBidTooLow,
            "0x08bfb2fd" => Self::TooManyChildTokens,
            "0xaf6f94fc" => Self::NoRightsToApproveBid,
            "0x98f2d62f" => Self::NoRightsToDeleteBid,
            "0xb419f128" => Self::FieldOutside,
            "0x62320032" => Self::FieldsOverlap,
            "0x82d6f713" => Self::FieldTooLarge,
            _ => Self::Other(message.into()),
        }
    }
}


#[async_trait(?Send)]
trait CallWrapper {
    fn contract(&self) -> &Contract<Either<Eip1193, Http>>;

    fn _handle_error(&self, error: Error);

    fn process_error(&self, error: web3::contract::Error) {
        if let web3::contract::Error::Api(web3::error::Error::Rpc(rpc_error)) = &error {
            if let Some(object) = &rpc_error.data {
                if let Some(object) = object.get("originalError") {
                    if let (Some(jsonrpc_core::types::Value::String(code)), Some(jsonrpc_core::types::Value::String(message))) = (object.get("data"), object.get("message")) {
                        self._handle_error(Error::from_code(code, message));
                    }
                }
            }
        }
    }

    async fn call<T: Clone + Tokenize + std::marker::Send>(&self, method: &str, params: T, sender: Address) -> Option<H256> {
        match self.contract().estimate_gas(method, params.clone(), sender, Options::default()).await {
            Ok(gas) => {
                log::info!("{} GAS: {:?}", method, gas);
            }
            Err(error) => {
                self.process_error(error);
                return None
            }
        }

        match self.contract().call(method, params, sender, Options::default()).await {
            Ok(tx_hash) => {
                Some(tx_hash)
            }
            Err(error) => {
                self.process_error(error);
                return None
            }
        }
    }

    async fn call_with_confirmations<T: Clone + Tokenize + std::marker::Send>(&self, method: &str, params: T, sender: Address) -> Option<TransactionReceipt> {
        match self.contract().estimate_gas(method, params.clone(), sender, Options::default()).await {
            Ok(gas) => {
                log::info!("{} GAS: {:?}", method, gas);
            }
            Err(error) => {
                self.process_error(error);
                return None
            }
        }

        match self.contract().call_with_confirmations(method, params, sender, Options::default(), 1).await {
            Ok(receipt) => {
                Some(receipt)
            }
            Err(error) => {
                // self.process_error(error);
                return None
            }
        }
    }
}


#[derive(Clone)]
pub struct ERC1155Contract {
    contract: Contract<Either<Eip1193, Http>>,
    handle_error: Arc<dyn Fn(Error)>,
}

#[async_trait]
impl CallWrapper for ERC1155Contract {
    fn contract(&self) -> &Contract<Either<Eip1193, Http>> {
        &self.contract
    }

    fn _handle_error(&self, error: Error) {
        (self.handle_error)(error);
    }
}

impl ERC1155Contract {
    pub fn new(web3: &Web3<Either<Eip1193, Http>>, handle_error: Arc<dyn Fn(Error)>) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("ERC1155_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/MandelbrotNFT.json"),
            ).unwrap(),
            handle_error,
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn get_fuel_balance(&self, address: Address) -> Result<f64> {
        let result: web3::contract::Result<U256> = self.contract.query(
            "balanceOf",
            (address, FUEL,),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?.as_u128() as f64 / 10_f64.powi(18))
    }

    pub async fn transfer_fuel(&self, from: Address, to: Address, amount: f64) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "safeTransferFrom",
            (
                from,
                to,
                FUEL,
                U256::from((amount * 10_f64.powi(18)) as u128),
                CALLDATA.to_vec(),
            ),
            from,
        ).await
    }

    pub async fn mint(&self, sender: Address, parent_id: u128, field: Field) -> Option<H256> {
        self.call(
            "mintNFT",
            (U256::from(parent_id), sender, field),
            sender,
        ).await
    }

    pub async fn burn(&self, sender: Address, token_id: u128) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "burn",
            (U256::from(token_id),),
            sender,
        ).await
    }

    pub async fn get_metadata(&self, token_id: u128) -> Result<Metadata> {
        let result: web3::contract::Result<Metadata> = self.contract.query(
            "getMetadata",
            (U256::from(token_id),),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn get_children_metadata(&self, parent_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self.contract.query(
            "getChildrenMetadata",
            (U256::from(parent_id),),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn get_ancestry_metadata(&self, token_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self.contract.query(
            "getAncestryMetadata",
            (U256::from(token_id),),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn bid(&self, sender: Address, parent_id: u128, field: Field, amount: f64, minimum_price: f64) -> Option<H256> {
        self.call(
            "bid",
            (
                U256::from(parent_id),
                sender,
                field.clone(),
                U256::from((amount * 10_f64.powi(18)) as u128),
                U256::from((minimum_price * 10_f64.powi(18)) as u128),
            ),
            sender,
        ).await
    }

    pub async fn get_bids(&self, parent_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self.contract.query(
            "getBids",
            (U256::from(parent_id),),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn get_owned_items(&self, owner: Address) -> Result<(Vec<Metadata>, Vec<Metadata>)> {
        let result: web3::contract::Result<(Vec<Metadata>, Vec<Metadata>)> = self.contract.query(
            "getOwnedItems",
            (owner,),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn approve_bid(&self, sender: Address, bid_id: u128) -> Option<H256> {
        self.call(
            "approve",
            (U256::from(bid_id),),
            sender,
        ).await
    }

    pub async fn batch_approve_bids(&self, sender: Address, bid_ids: &[u128]) -> Option<H256> {
        self.call(
            "batchApprove",
            (bid_ids.iter().map(|bid_id| U256::from(*bid_id)).collect::<Vec<U256>>(),),
            sender,
        ).await
    }

    pub async fn delete_bid(&self, sender: Address, bid_id: u128) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "deleteBid",
            (U256::from(bid_id),),
            sender,
        ).await
    }

    pub async fn set_minimum_bid(&self, sender: Address, token_id: u128, minimum_bid: f64) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "setminimumBid", // TODO: fix case typo
            (
                U256::from(token_id),
                U256::from((minimum_bid * 10_f64.powi(18)) as u128),
            ),
            sender,
        ).await

    }
}


#[derive(Clone)]
pub struct Wrapped1155FactoryContract {
    contract: Contract<Either<Eip1193, Http>>,
    handle_error: Arc<dyn Fn(Error)>,
    erc1155_address: Address,
}

#[async_trait]
impl CallWrapper for Wrapped1155FactoryContract {
    fn contract(&self) -> &Contract<Either<Eip1193, Http>> {
        &self.contract
    }

    fn _handle_error(&self, error: Error) {
        (self.handle_error)(error);
    }
}

impl Wrapped1155FactoryContract {
    pub fn new(web3: &Web3<Either<Eip1193, Http>>, erc1155_address: Address, handle_error: Arc<dyn Fn(Error)>) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("WRAPPER_FACTORY_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/Wrapped1155Factory.json"),
            ).unwrap(),
            handle_error,
            erc1155_address,
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn unwrap(&self, recipient: Address, amount: f64) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "unwrap",
            (
                self.erc1155_address,
                FUEL,
                U256::from((amount * 10_f64.powi(18)) as u128),
                recipient,
                CALLDATA.to_vec(),
            ), recipient
        ).await
    }
}


#[derive(Clone)]
pub struct ERC20Contract {
    contract: Contract<Either<Eip1193, Http>>,
}

impl ERC20Contract {
    pub fn new(web3: &Web3<Either<Eip1193, Http>>) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("ERC20_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/Wrapped1155.json"),
            ).unwrap(),
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn get_balance(&self, address: Address) -> Result<f64> {
        let result: web3::contract::Result<U256> = self.contract.query(
            "balanceOf",
            (address,),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?.as_u128() as f64 / 10_f64.powi(18))
    }
}
