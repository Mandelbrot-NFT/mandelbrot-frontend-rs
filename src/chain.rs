use leptos_ethereum_provider::{base_currency, Chain};


pub fn ethereum() -> Chain {
    Chain {
        chain_id: "0x1".into(),
        chain_name: "Ethereum Mainnet".into(),
        rpc_urls: [String::from("https://mainnet.infura.io/v3")],
        native_currency: base_currency::eth(),
        block_explorer_urls: Some([String::from("https://etherscan.io")]),
    }
}

pub fn sepolia_testnet() -> Chain {
    Chain {
        chain_name: String::from("Sepolia test network"),
        chain_id: String::from("0xAA36A7"),
        rpc_urls: [String::from("https://rpc.notadegen.com/sepolia")],
        native_currency: base_currency::eth(),
        block_explorer_urls: Some([String::from("https://sepolia.etherscan.io")]),
    }
}
