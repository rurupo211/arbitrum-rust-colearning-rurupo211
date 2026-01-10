use anyhow::{anyhow, Context, Result};
use ethers::prelude::*;
use ethers::utils::format_ether;
use std::env;
use std::str::FromStr;


async fn query_arb_sepolia_eth_balance(address: Address, rpc_url: &str) -> Result<(U256, String)> {

    let provider = Provider::<Http>::try_from(rpc_url)
        .with_context(|| format!("RPC URL 无法解析或初始化 Provider: {rpc_url}"))?;


    let wei: U256 = provider
        .get_balance(address, None)
        .await
        .context("RPC 调用失败：get_balance")?;


    let eth_readable = format_ether(wei);

    Ok((wei, eth_readable))
}

#[tokio::main]
async fn main() -> Result<()> {

    let mut args = env::args().skip(1);

    let address_str = args
        .next()
        .ok_or_else(|| anyhow!("缺少参数：地址。\n用法: balance <ADDRESS> [RPC_URL]"))?;


    let default_rpc = "https://sepolia-rollup.arbitrum.io/rpc";
    let rpc_url = args.next().unwrap_or_else(|| default_rpc.to_string());

    let address =
        Address::from_str(&address_str).with_context(|| format!("地址格式不正确: {address_str}"))?;

    let (wei, eth) = query_arb_sepolia_eth_balance(address, &rpc_url).await?;

    println!("✅ addr={:?} | eth={} | wei={} | convert={wei}->{eth}", address, eth, wei);



    Ok(())
}
