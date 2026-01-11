use std::env;

fn main() {
    dotenvy::dotenv().ok();

    // Read CONTRACT_ADDRESS
    if let Ok(contract_addr) = env::var("CONTRACT_ADDRESS") {
        println!("cargo:rustc-env=CONTRACT_ADDRESS={}", contract_addr);
    }

    // Rebuild if .env changes
    println!("cargo:rerun-if-changed=.env");
}