use std::sync::Arc;

use eyre::Result;
use web3::{
    Web3,
    contract::{Contract, Options, tokens::Tokenize},
    transports::{Either, Http, eip_1193::Eip1193},
    types::{Address, H256, TransactionReceipt, U256},
};

use super::types::{Field, Metadata};

pub enum Error {
    TokenNotFound,
    NoRightsToBurn, // Only the token owner can burn it
    TokenNotEmpty,  // Cannot burn token if it has children
    BidNotFound,
    BidTooLow,            // Bid must exceed or equal minimum bid price
    MinimumBidTooLow,     // Child's minimum bid has to be at least as much as parent's
    TooManyChildTokens,   // A maximum of MAX_CHILDREN child tokens can be minted
    NoRightsToApproveBid, // Only the owner of parent token can approve the bid
    NoRightsToDeleteBid,  // Only the bid creator can delete it
    FieldOutside,         // Token has to be within the field of its parent
    FieldsOverlap,        // Sibling fields cannot overlap
    FieldTooLarge,        // Token's field cannot exceed MAXIMUM_FIELD_PORTION % of its parent's
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

trait CallWrapper {
    fn contract(&self) -> &Contract<Either<Eip1193, Http>>;

    fn _handle_error(&self, error: Error);

    fn process_error(&self, error: web3::contract::Error) {
        if let web3::contract::Error::Api(web3::error::Error::Rpc(rpc_error)) = &error {
            if let Some(jsonrpc_core::types::Value::String(code)) = &rpc_error.data {
                self._handle_error(Error::from_code(code, &rpc_error.message));
            }
        }
    }

    async fn call<T: Clone + Tokenize + std::marker::Send>(
        &self,
        method: &str,
        params: T,
        sender: Address,
    ) -> Option<H256> {
        match self
            .contract()
            .estimate_gas(method, params.clone(), sender, Options::default())
            .await
        {
            Ok(gas) => {
                log::info!("{} GAS: {:?}", method, gas);
            }
            Err(error) => {
                self.process_error(error);
                return None;
            }
        }

        match self.contract().call(method, params, sender, Options::default()).await {
            Ok(tx_hash) => Some(tx_hash),
            Err(error) => {
                self.process_error(error);
                return None;
            }
        }
    }

    async fn call_with_confirmations<T: Clone + Tokenize + std::marker::Send>(
        &self,
        method: &str,
        params: T,
        sender: Address,
    ) -> Option<TransactionReceipt> {
        match self
            .contract()
            .estimate_gas(method, params.clone(), sender, Options::default())
            .await
        {
            Ok(gas) => {
                log::info!("{} GAS: {:?}", method, gas);
            }
            Err(error) => {
                self.process_error(error);
                return None;
            }
        }

        match self
            .contract()
            .call_with_confirmations(method, params, sender, Options::default(), 1)
            .await
        {
            Ok(receipt) => Some(receipt),
            Err(error) => {
                self.process_error(error.into());
                return None;
            }
        }
    }
}

#[derive(Clone)]
pub struct MandelbrotNFTContract {
    contract: Contract<Either<Eip1193, Http>>,
    handle_error: Arc<dyn Fn(Error)>,
}

impl CallWrapper for MandelbrotNFTContract {
    fn contract(&self) -> &Contract<Either<Eip1193, Http>> {
        &self.contract
    }

    fn _handle_error(&self, error: Error) {
        (self.handle_error)(error);
    }
}

impl MandelbrotNFTContract {
    pub fn new(web3: &Web3<Either<Eip1193, Http>>, handle_error: Arc<dyn Fn(Error)>) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/MandelbrotNFT.json"),
            )
            .unwrap(),
            handle_error,
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn get_token_balance(&self, address: Address) -> Result<f64> {
        let result: web3::contract::Result<U256> = self
            .contract
            .query("balanceOf", address, None, Options::default(), None)
            .await;
        Ok(result?.as_u128() as f64 / 10_f64.powi(18))
    }

    pub async fn _mint(&self, sender: Address, parent_id: u128, field: Field) -> Option<H256> {
        self.call("mintNFT", (U256::from(parent_id), sender, field), sender)
            .await
    }

    pub async fn burn(&self, sender: Address, token_id: u128) -> Option<TransactionReceipt> {
        self.call_with_confirmations("burn", (U256::from(token_id),), sender)
            .await
    }

    pub async fn _get_metadata(&self, token_id: u128) -> Result<Metadata> {
        let result: web3::contract::Result<Metadata> = self
            .contract
            .query("getMetadata", (U256::from(token_id),), None, Options::default(), None)
            .await;
        Ok(result?)
    }

    pub async fn get_children_metadata(&self, parent_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self
            .contract
            .query(
                "getChildrenMetadata",
                (U256::from(parent_id),),
                None,
                Options::default(),
                None,
            )
            .await;
        Ok(result?)
    }

    pub async fn get_ancestry_metadata(&self, token_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self
            .contract
            .query(
                "getAncestryMetadata",
                (U256::from(token_id),),
                None,
                Options::default(),
                None,
            )
            .await;
        Ok(result?)
    }

    pub async fn bid(
        &self,
        sender: Address,
        parent_id: u128,
        field: Field,
        amount: f64,
        minimum_price: f64,
    ) -> Option<H256> {
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
        )
        .await
    }

    pub async fn get_bids(&self, parent_id: u128) -> Result<Vec<Metadata>> {
        let result: web3::contract::Result<Vec<Metadata>> = self
            .contract
            .query("getBids", (U256::from(parent_id),), None, Options::default(), None)
            .await;
        Ok(result?)
    }

    pub async fn get_owned_items(&self, owner: Address) -> Result<(Vec<Metadata>, Vec<Metadata>)> {
        let result: web3::contract::Result<(Vec<Metadata>, Vec<Metadata>)> = self
            .contract
            .query("getOwnedItems", (owner,), None, Options::default(), None)
            .await;
        Ok(result?)
    }

    pub async fn _approve_bid(&self, sender: Address, bid_id: u128) -> Option<H256> {
        self.call("approve", (U256::from(bid_id),), sender).await
    }

    pub async fn batch_approve_bids(&self, sender: Address, bid_ids: &[u128]) -> Option<H256> {
        self.call(
            "batchApprove",
            (bid_ids.iter().map(|bid_id| U256::from(*bid_id)).collect::<Vec<U256>>(),),
            sender,
        )
        .await
    }

    pub async fn delete_bid(&self, sender: Address, bid_id: u128) -> Option<TransactionReceipt> {
        self.call_with_confirmations("deleteBid", (U256::from(bid_id),), sender)
            .await
    }

    pub async fn set_minimum_bid(
        &self,
        sender: Address,
        token_id: u128,
        minimum_bid: f64,
    ) -> Option<TransactionReceipt> {
        self.call_with_confirmations(
            "setMinimumBid",
            (
                U256::from(token_id),
                U256::from((minimum_bid * 10_f64.powi(18)) as u128),
            ),
            sender,
        )
        .await
    }
}
