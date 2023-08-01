use ethabi::token::Token;
use web3::{
    contract::tokens::Tokenizable,
    types::{Address, U256},
};


#[derive(Clone, Debug)]
pub struct Field {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
}

impl Tokenizable for Field {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    x_min: U256::from_token(tokens[0].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    y_min: U256::from_token(tokens[1].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    x_max: U256::from_token(tokens[2].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0,
                    y_max: U256::from_token(tokens[3].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.0
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            U256::from(((self.x_min + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_min + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.x_max + 2.0) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.y_max + 2.0) * 10_f64.powi(18)) as u128).into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Field {}


#[derive(Clone, Debug)]
pub struct Metadata {
    pub token_id: u128,
    pub owner: Address,
    parent_id: u128,
    pub field: Field,
    pub minimum_price: f64,
}

impl Tokenizable for Metadata {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    token_id: U256::from_token(tokens[0].clone())?.as_u128(),
                    owner: Address::from_token(tokens[1].clone())?,
                    parent_id: U256::from_token(tokens[2].clone())?.as_u128(),
                    field: Field::from_token(tokens[3].clone())?,
                    minimum_price: U256::from_token(tokens[4].clone())?.as_u128() as f64 / 10_f64.powi(18),
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            self.token_id.into_token(),
            self.owner.into_token(),
            self.parent_id.into_token(),
            self.field.into_token(),
            U256::from(((self.minimum_price) * 10_f64.powi(18)) as u128).into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Metadata {}

impl Metadata {
    pub fn to_frame(&self, color: mandelbrot_explorer::FrameColor) -> mandelbrot_explorer::Frame {
        mandelbrot_explorer::Frame {
            id: self.token_id,
            x_min: self.field.x_min,
            x_max: self.field.x_max,
            y_min: self.field.y_min,
            y_max: self.field.y_max,
            color,
        }
    } 
}


#[derive(Debug)]
pub struct Bid {
    pub bid_id: u128,
    parent_id: u128,
    pub field: Field,
    pub recipient: Address,
    pub amount: f64,
    pub selected: bool,
}

impl Tokenizable for Bid {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self { 
                    bid_id: U256::from_token(tokens[0].clone())?.as_u128(),
                    parent_id: U256::from_token(tokens[1].clone())?.as_u128(),
                    field: Field::from_token(tokens[2].clone())?,
                    recipient: Address::from_token(tokens[3].clone())?,
                    amount: U256::from_token(tokens[4].clone())?.as_u128() as f64 / 10_f64.powi(18),
                    selected: false,
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            self.bid_id.into_token(),
            self.parent_id.into_token(),
            self.field.into_token(),
            self.recipient.into_token(),
            U256::from(((self.amount) * 10_f64.powi(18)) as u128).into_token(),
        ])
    }
}

impl web3::contract::tokens::TokenizableItem for Bid {}

impl Bid {
    pub fn to_frame(&self) -> mandelbrot_explorer::Frame {
        mandelbrot_explorer::Frame {
            id: self.bid_id,
            x_min: self.field.x_min,
            x_max: self.field.x_max,
            y_min: self.field.y_min,
            y_max: self.field.y_max,
            color: if self.selected {mandelbrot_explorer::FrameColor::Green} else {mandelbrot_explorer::FrameColor::Yellow},
        }
    } 
}
