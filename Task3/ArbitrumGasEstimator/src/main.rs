use ethers::prelude::*;
use std::convert::TryFrom;

// Arbitrum Sepolia 测试网公共 RPC (仅供测试，生产环境建议使用 Alchemy/Infura)
const RPC_URL: &str = "https://sepolia-rollup.arbitrum.io/rpc";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let provider = Provider::<Http>::try_from(RPC_URL)?;
    println!("✅ 已连接到 Arbitrum Sepolia 测试网");

    //  获取实时 Gas 价格 (单位: Wei)
    // 核心指引：不要硬编码，使用 provider.get_gas_price()
    let gas_price = provider.get_gas_price().await?;
    
    // 定义基础转账 Gas 限额 (Standard Transfer Gas Limit)
    // 行业通用值：普通 ETH 转账通常固定消耗 21,000 Gas
    let gas_limit = U256::from(21000);

    // 计算预估 Gas 费
    // 计算公式：Gas 费 = Gas 价格 × Gas 限额
    let estimated_fee = gas_price * gas_limit;


    println!("------------------------------------------------");
    println!("🔥 实时 Gas Price: {} Wei ({:.2} Gwei)", 
        gas_price, 
        ethers::utils::format_units(gas_price, "gwei")?.parse::<f64>()?
    );
    println!("⛽ 基础转账 Gas Limit: {}", gas_limit);
    println!("💰 预估转账手续费: {} Wei", estimated_fee);
    
    // 转换为 ETH 单位方便阅读
    let fee_in_eth = ethers::utils::format_units(estimated_fee, "ether")?;
    println!("📉 约合 ETH: {} ETH", fee_in_eth);
    println!("------------------------------------------------");

    Ok(())
}