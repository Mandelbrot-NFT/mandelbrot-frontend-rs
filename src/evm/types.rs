use ethabi::token::Token;
use num_bigfloat::BigFloat;
use web3::{
    contract::tokens::Tokenizable,
    types::{Address, U256},
};


#[derive(Clone, Debug)]
pub struct Field {
    pub x_min: BigFloat,
    pub y_min: BigFloat,
    pub x_max: BigFloat,
    pub y_max: BigFloat,
}

impl Tokenizable for Field {
    fn from_token(token: Token) -> Result<Self, web3::contract::Error> {
        match token {
            Token::Tuple(tokens) => {
                Ok(Self {
                    // TODO: parse token to bigfloat
                    x_min: BigFloat::from(U256::from_token(tokens[0].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.1),
                    y_min: BigFloat::from(U256::from_token(tokens[1].clone())?.as_u128() as f64 / 10_f64.powi(18) - 1.5),
                    x_max: BigFloat::from(U256::from_token(tokens[2].clone())?.as_u128() as f64 / 10_f64.powi(18) - 2.1),
                    y_max: BigFloat::from(U256::from_token(tokens[3].clone())?.as_u128() as f64 / 10_f64.powi(18) - 1.5),
                })
            }
            _ => Err(web3::contract::Error::Abi(ethabi::Error::InvalidData)),
        }
    }

    fn into_token(self) -> Token {
        Token::Tuple(vec![
            U256::from(((self.x_min + BigFloat::from(2.1)) * BigFloat::from(10_f64.powi(18))).to_u128().unwrap()).into_token(),
            U256::from(((self.y_min + BigFloat::from(1.5)) * BigFloat::from(10_f64.powi(18))).to_u128().unwrap()).into_token(),
            U256::from(((self.x_max + BigFloat::from(2.1)) * BigFloat::from(10_f64.powi(18))).to_u128().unwrap()).into_token(),
            U256::from(((self.y_max + BigFloat::from(1.5)) * BigFloat::from(10_f64.powi(18))).to_u128().unwrap()).into_token(),
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
    pub locked_fuel: f64,
    pub minimum_price: f64,
    pub layer: u128,
    pub owned: bool,
    pub selected: bool,
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
                    locked_fuel: U256::from_token(tokens[4].clone())?.as_u128() as f64 / 10_f64.powi(18),
                    minimum_price: U256::from_token(tokens[5].clone())?.as_u128() as f64 / 10_f64.powi(18),
                    layer: U256::from_token(tokens[6].clone())?.as_u128(),
                    owned: false,
                    selected: false,
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
            U256::from(((self.locked_fuel) * 10_f64.powi(18)) as u128).into_token(),
            U256::from(((self.minimum_price) * 10_f64.powi(18)) as u128).into_token(),
            self.layer.into_token(),
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
            color: if self.selected {
                mandelbrot_explorer::FrameColor::Green
            } else {
                if self.owned {
                    match color {
                        mandelbrot_explorer::FrameColor::Red => mandelbrot_explorer::FrameColor::Pink,
                        mandelbrot_explorer::FrameColor::Yellow => mandelbrot_explorer::FrameColor::Lemon,
                        _ => mandelbrot_explorer::FrameColor::LightBlue
                    }
                } else {
                    color
                }
            },
        }
    }
}
