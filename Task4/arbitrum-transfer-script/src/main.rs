use anyhow::{anyhow, Context, Result};
use dotenv::dotenv;
use ethers::prelude::*;
use ethers::providers::Middleware;
use ethers::signers::Signer;
use ethers::types::{BlockNumber, H256};
use ethers::types::transaction::eip1559::Eip1559TransactionRequest;
use ethers::types::transaction::eip2718::TypedTransaction;
use ethers::utils::{format_ether, parse_units};
use std::{cmp::max, env, str::FromStr, sync::Arc, time::Duration};

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // ===== 1) 读取环境变量 =====
    let private_key = env::var("PRIVATE_KEY").context("请在 .env 文件中设置 PRIVATE_KEY")?;
    let recipient = env::var("RECIPIENT_ADDRESS").context("请在 .env 文件中设置 RECIPIENT_ADDRESS")?;
    let rpc_url = env::var("ARBITRUM_RPC_URL").unwrap_or_else(|_| "https://sepolia-rollup.arbitrum.io/rpc".to_string());
    let amount_eth_str = env::var("AMOUNT_ETH").unwrap_or_else(|_| "0.001".to_string());

    println!("📡 RPC: {}", rpc_url);
    println!("💰 转账金额(ETH): {}", amount_eth_str);

    // ===== 2) 解析地址/钱包 =====
    let recipient_address: Address = Address::from_str(&recipient)
        .with_context(|| format!("RECIPIENT_ADDRESS 不是合法地址: {recipient}"))?;

    let wallet: LocalWallet = private_key
        .trim()
        .parse::<LocalWallet>()
        .context("PRIVATE_KEY 私钥格式错误（hex，可带 0x）")?;

    let provider = Provider::<Http>::try_from(rpc_url.as_str())
        .with_context(|| format!("RPC URL 无法初始化 Provider: {rpc_url}"))?
        .interval(Duration::from_millis(250));

    // ===== 3) chain_id：必须写入 signer，否则会报 invalid chain id =====
    let chain_id_u256 = provider.get_chainid().await.context("获取 chain_id 失败")?;
    let chain_id = chain_id_u256.as_u64();
    let wallet = wallet.with_chain_id(chain_id);

    let sender_address = wallet.address();
    println!("👤 发送方: {:#x}", sender_address);
    println!("👤 接收方: {:#x}", recipient_address);
    println!("⛓️  Chain ID: {}", chain_id);

    let client = Arc::new(SignerMiddleware::new(provider.clone(), wallet));

    // ===== 4) 金额：字符串 -> wei（避免浮点误差）=====
    let amount_wei: U256 = parse_units(&amount_eth_str, 18)
        .context("AMOUNT_ETH 解析失败（建议类似 0.001）")?
        .into();
    println!("🔢 转账金额(Wei): {}", amount_wei);

    // ===== 5) 余额检查 =====
    let sender_balance = provider.get_balance(sender_address, None).await.context("获取发送方余额失败")?;
    let recipient_balance = provider.get_balance(recipient_address, None).await.context("获取接收方余额失败")?;
    println!("💵 发送方余额: {} ETH", format_ether(sender_balance));
    println!("💵 接收方余额: {} ETH", format_ether(recipient_balance));

    if sender_balance < amount_wei {
        return Err(anyhow!(
            "余额不足：需要 {} ETH，当前仅 {} ETH",
            amount_eth_str,
            format_ether(sender_balance)
        ));
    }

    // ===== 6) 读取 baseFee + 估算 EIP-1559 fee（防 maxFee < baseFee）=====
    let latest_block = provider
        .get_block(BlockNumber::Latest)
        .await
        .context("获取最新区块失败")?
        .ok_or_else(|| anyhow!("拿不到最新区块"))?;

    let base_fee = latest_block
        .base_fee_per_gas
        .ok_or_else(|| anyhow!("最新区块没有 base_fee_per_gas（RPC 可能不支持）"))?;

    let (suggest_max_fee, suggest_tip) = provider
        .estimate_eip1559_fees(None)
        .await
        .context("estimate_eip1559_fees 失败")?;

    // 硬规则：max_fee >= base_fee + tip，再 +20% buffer
    let min_need = base_fee + suggest_tip;
    let final_max_fee = max(suggest_max_fee, min_need) * 12 / 10;

    println!("⛽ baseFee: {} wei", base_fee);
    println!("⛽ tip(suggest): {} wei", suggest_tip);
    println!("⛽ maxFee(suggest): {} wei", suggest_max_fee);
    println!("⛽ maxFee(final +20%): {} wei", final_max_fee);

    // ===== 7) 构造 EIP-1559 交易（先不写 gas，先 estimate_gas）=====
    let mut tx1559 = Eip1559TransactionRequest {
        from: Some(sender_address),
        to: Some(NameOrAddress::Address(recipient_address)),
        value: Some(amount_wei),
        max_fee_per_gas: Some(final_max_fee),
        max_priority_fee_per_gas: Some(suggest_tip),
        gas: None,
        data: None, // 纯转账
        ..Default::default()
    };

    // ===== 8) estimate_gas + buffer（修 intrinsic gas too low）=====
    let typed_for_estimate: TypedTransaction = tx1559.clone().into();
    let gas_est = provider
        .estimate_gas(&typed_for_estimate, None)
        .await
        .context("estimate_gas 失败")?;
    let gas_limit = gas_est * 120 / 100; // +20%
    tx1559.gas = Some(gas_limit);

    println!("⛽ gas_est: {}", gas_est);
    println!("⛽ gas_limit(+20%): {}", gas_limit);

    // ===== 9) 再次余额检查（含 gas 上限）=====
    let gas_fee_upper = gas_limit * final_max_fee;
    let total_upper = amount_wei + gas_fee_upper;

    println!("💸 Gas 上限费用: {} ETH", format_ether(gas_fee_upper));
    println!("💸 总费用上限: {} ETH", format_ether(total_upper));

    if sender_balance < total_upper {
        return Err(anyhow!(
            "余额不足（含 Gas 上限）：需要 {} ETH，当前仅 {} ETH",
            format_ether(total_upper),
            format_ether(sender_balance)
        ));
    }

    // ===== 10) 发送交易 =====
    println!("⏳ 发送交易...");
    let pending_tx = client
        .send_transaction(tx1559, None)
        .await
        .context("交易发送失败")?;

    let tx_hash: H256 = *pending_tx;
    println!("✅ 已广播！");
    println!("📄 交易哈希: {:#x}", tx_hash);
    println!("🔍 浏览器: https://sepolia.arbiscan.io/tx/{:#x}", tx_hash);

    // ===== 11) 等待回执（可选）=====
    println!("⏰ 等待确认...");
    match pending_tx.await {
        Ok(Some(receipt)) => {
            println!("✅ 交易已确认！");
            println!("📦 区块号: {:?}", receipt.block_number);
            println!("⛽ 实际 Gas 使用: {:?}", receipt.gas_used);
            println!("🏷️  状态: {:?}", receipt.status);
        }
        Ok(None) => {
            println!("⚠️  已发送但暂未返回回执（可能还在 pending），请用 hash 在浏览器查看");
        }
        Err(e) => return Err(anyhow!("等待回执失败: {}", e)),
    }

    Ok(())
}
