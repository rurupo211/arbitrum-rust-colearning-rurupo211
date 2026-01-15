use ethers::{
    abi::Abi,
    contract::Contract,
    providers::{Http, Provider},
    types::Address,
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 开始查询合约...");
    
    // 1. 连接到 Arbitrum Sepolia 测试网
    let rpc_url = "https://sepolia-rollup.arbitrum.io/rpc";
    let provider = Provider::<Http>::try_from(rpc_url)?;
    
    // 测试连接
    let block_number = provider.get_block_number().await?;
    println!("✅ 连接成功！当前区块号: {}", block_number);
    
    // 2. 设置要查询的合约地址（USDC 测试合约）
    let contract_address = "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d";
    println!("📋 查询合约地址: {}", contract_address);
    
    // 3. 将字符串地址转换为 Address 类型
    let address = Address::from_str(contract_address)?;
    
    // 4. 定义合约的 ABI（我们只需要查询函数）
    // ERC20 合约的标准查询函数：name(), symbol(), decimals()
    let abi_json = r#"[
        {
            "inputs": [],
            "name": "name",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "symbol",
            "outputs": [{"internalType": "string", "name": "", "type": "string"}],
            "stateMutability": "view",
            "type": "function"
        },
        {
            "inputs": [],
            "name": "decimals",
            "outputs": [{"internalType": "uint8", "name": "", "type": "uint8"}],
            "stateMutability": "view",
            "type": "function"
        }
    ]"#;
    
    // 5. 解析 ABI
    let abi: Abi = serde_json::from_str(abi_json)?;
    println!("✅ ABI 解析成功");
    
    // 6. 创建合约实例
    let contract = Contract::new(address, abi, provider);
    
    // 7. 查询合约信息
    println!("");
    println!("📊 开始查询合约信息...");
    
    // 7.1 查询合约名称
    println!("🔍 查询 name()...");
    match contract.method::<_, String>("name", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(name) => println!("   ✅ 合约名称: {}", name),
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    // 7.2 查询代币符号
    println!("🔍 查询 symbol()...");
    match contract.method::<_, String>("symbol", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(symbol) => println!("   ✅ 代币符号: {}", symbol),
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    // 7.3 查询小数位数
    println!("🔍 查询 decimals()...");
    match contract.method::<_, u8>("decimals", ()) {
        Ok(method) => {
            match method.call().await {
                Ok(decimals) => println!("   ✅ 小数位数: {}", decimals),
                Err(e) => println!("   ❌ 查询失败: {}", e),
            }
        }
        Err(e) => println!("   ❌ 构建查询失败: {}", e),
    }
    
    println!("");
    println!("🎉 合约查询完成！");
    
    Ok(())
}