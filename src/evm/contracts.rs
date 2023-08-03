use async_trait::async_trait;
use eyre::Result;
use web3::{
    contract::{tokens::Tokenize, Contract, Options},
    types::{Address, H256, U256, TransactionReceipt},
    transports::eip_1193::Eip1193,
    Web3
};

use super::types::{Bid, Field, Metadata};


const FUEL: U256 = U256([0, 0, 0, 0]);
const CALLDATA: &[u8] = &[87, 114, 97, 112, 112, 101, 100, 32, 77, 97, 110, 100, 101, 108, 98, 114, 111, 116, 32, 70, 85, 69, 76, 0, 0, 0, 0, 0, 0, 0, 0, 46, 119, 70, 85, 69, 76, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10, 18];


#[async_trait(?Send)]
trait CallWrapper {
    fn contract(&self) -> &Contract<Eip1193>;

    async fn call<T: Clone + Tokenize + std::marker::Send>(&self, method: &str, params: T, sender: Address) -> Result<H256> {
        let gas = self.contract().estimate_gas(method, params.clone(), sender, Options::default()).await?;
        log::info!("{} GAS: {:?}", method, gas);
        Ok(self.contract().call(method, params, sender, Options::default()).await?)
    }

    async fn call_with_confirmations<T: Clone + Tokenize + std::marker::Send>(&self, method: &str, params: T, sender: Address) -> Result<TransactionReceipt> {
        let gas = self.contract().estimate_gas(method, params.clone(), sender, Options::default()).await?;
        log::info!("{} GAS: {:?}", method, gas);
        Ok(self.contract().call_with_confirmations(method, params, sender, Options::default(), 1).await?)
    }
}


#[derive(Clone)]
pub struct ERC1155Contract {
    contract: Contract<Eip1193>
}

#[async_trait]
impl CallWrapper for ERC1155Contract {
    fn contract(&self) -> &Contract<Eip1193> {
        &self.contract
    }
}

impl ERC1155Contract {
    pub fn new(web3: &Web3<Eip1193>) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("ERC1155_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/MandelbrotNFT.json"),
            ).unwrap()
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

    pub async fn transfer_fuel(&self, from: Address, to: Address, amount: f64) -> Result<TransactionReceipt> {
        Ok(self.call_with_confirmations(
            "safeTransferFrom", (
                from,
                to,
                FUEL,
                U256::from((amount * 10_f64.powi(18)) as u128),
                CALLDATA.to_vec(),
            ),
            from,
        ).await?)
    }

    pub async fn mint(&self, sender: Address, parent_id: u128, field: Field) -> Result<H256> {
        Ok(self.call(
            "mintNFT",
            (U256::from(parent_id), sender, field),
            sender,
        ).await?)
    }

    pub async fn burn(&self, sender: Address, token_id: u128) -> Result<H256> {
        Ok(self.call(
            "burn",
            (U256::from(token_id),),
            sender,
        ).await?)
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

    pub async fn bid(&self, sender: Address, parent_id: u128, field: Field, amount: f64, minimum_price: f64) -> Result<H256> {
        Ok(self.call(
            "bid",
            (
                U256::from(parent_id),
                sender,
                field.clone(),
                U256::from((amount * 10_f64.powi(18)) as u128),
                U256::from((minimum_price * 10_f64.powi(18)) as u128),
            ),
            sender,
        ).await?)
    }

    pub async fn get_bids(&self, parent_id: u128) -> Result<Vec<Bid>> {
        let result: web3::contract::Result<Vec<Bid>> = self.contract.query(
            "getBids",
            (U256::from(parent_id),),
            None,
            Options::default(),
            None
        ).await;
        Ok(result?)
    }

    pub async fn approve_bid(&self, sender: Address, bid_id: u128) -> Result<H256> {
        Ok(self.call(
            "approve",
            (U256::from(bid_id),),
            sender,
        ).await?)
    }

    pub async fn batch_approve_bids(&self, sender: Address, bid_ids: &[u128]) -> Result<H256> {
        Ok(self.call(
            "batchApprove",
            (bid_ids.iter().map(|bid_id| U256::from(*bid_id)).collect::<Vec<U256>>(),),
            sender,
        ).await?)
    }

    pub async fn delete_bid(&self, sender: Address, bid_id: u128) -> Result<H256> {
        Ok(self.call(
            "deleteBid",
            (U256::from(bid_id),),
            sender,
        ).await?)
    }
}


#[derive(Clone)]
pub struct Wrapped1155FactoryContract {
    contract: Contract<Eip1193>,
    erc1155_address: Address,
}

#[async_trait]
impl CallWrapper for Wrapped1155FactoryContract {
    fn contract(&self) -> &Contract<Eip1193> {
        &self.contract
    }
}

impl Wrapped1155FactoryContract {
    pub fn new(web3: &Web3<Eip1193>, erc1155_address: Address) -> Self {
        Self {
            contract: Contract::from_json(
                web3.eth(),
                env!("WRAPPER_FACTORY_CONTRACT_ADDRESS").trim_start_matches("0x").parse().unwrap(),
                include_bytes!("../../resources/Wrapped1155Factory.json"),
            ).unwrap(),
            erc1155_address,
        }
    }

    pub fn address(&self) -> Address {
        self.contract.address()
    }

    pub async fn unwrap(&self, recipient: Address, amount: f64) -> Result<TransactionReceipt>{
        Ok(self.call_with_confirmations(
            "unwrap", (
                self.erc1155_address,
                FUEL,
                U256::from((amount * 10_f64.powi(18)) as u128),
                recipient,
                CALLDATA.to_vec(),
            ), recipient
        ).await?)
    }
}


#[derive(Clone)]
pub struct ERC20Contract {
    contract: Contract<Eip1193>,
}

impl ERC20Contract {
    pub fn new(web3: &Web3<Eip1193>) -> Self {
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
