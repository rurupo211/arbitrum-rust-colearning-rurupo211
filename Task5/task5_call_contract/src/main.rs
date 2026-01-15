use ethers::prelude::*;
use eyre::Result;
use std::sync::Arc;

abigen!(
    MinimalErc20,
    r#"[
        function name() view returns (string)
        function symbol() view returns (string)
        function decimals() view returns (uint8)
    ]"#,
);

#[tokio::main]
async fn main() -> Result<()> {
    // 1) 自动加载项目根目录的 .env
    //    注意：.env 里应该写 ARB_SEPOLIA_RPC=...，不要写 export
    dotenvy::dotenv().ok();

    // 2) 读取 RPC
    let rpc = std::env::var("ARB_SEPOLIA_RPC").unwrap_or_else(|_| {
        eprintln!("缺少 ARB_SEPOLIA_RPC。请在项目根目录的 .env 文件里写：");
        eprintln!("ARB_SEPOLIA_RPC=https://sepolia-rollup.arbitrum.io/rpc");
        std::process::exit(1);
    });

    // 3) Provider（只读调用不需要私钥）
    let provider = Provider::<Http>::try_from(rpc)?;
    let provider = Arc::new(provider);

    // 4) 目标合约地址（Arbitrum Sepolia 上的公开 MockERC20）
    let token_addr: Address = "0xb7da0b8e3e5221ed32bd7cbc1f4af4e61020f365".parse()?;

    // 5) 绑定合约
    let token = MinimalErc20::new(token_addr, provider);

    // 6) 调用只读方法
    let name = token.name().call().await?;
    let symbol = token.symbol().call().await?;
    let decimals = token.decimals().call().await?;

    println!("contract: {:?}", token_addr);
    println!("name: {}", name);
    println!("symbol: {}", symbol);
    println!("decimals: {}", decimals);

    Ok(())
}
