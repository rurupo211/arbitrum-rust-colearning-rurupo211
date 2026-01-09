use alloy::providers::{Provider, ProviderBuilder};
use alloy::primitives::Address;
use alloy::sol;
use std::error::Error;


sol! {
    #[sol(rpc)]
    contract HelloWeb3 {
        function hello_web3() pure public returns (string memory);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let p = ProviderBuilder::new()
        .connect_http("https://arbitrum-sepolia-rpc.publicnode.com".parse()?);

    println!("Latest block number: {}", p.get_block_number().await?);

    println!(
        "合约返回: {}",
        HelloWeb3::new(
            "0x3f1f78ED98Cd180794f1346F5bD379D5Ec47DE90".parse::<Address>()?,
            p
        )
        .hello_web3()
        .call()
        .await?
    );

    Ok(())
}
