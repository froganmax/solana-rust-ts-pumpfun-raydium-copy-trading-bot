use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{bs58, commitment_config::CommitmentConfig, signature::Signature};
use solana_transaction_status::{option_serializer::OptionSerializer, UiTransactionEncoding};
use std::env;
use std::str::FromStr;

use crate::common::utils::ParseTx;

pub async fn tx_parse(rpc_client: RpcClient, signature: &str) -> Result<ParseTx, String> {
    let sig = Signature::from_str(signature).expect("Invalid signature");
    let config = RpcTransactionConfig {
        encoding: Some(UiTransactionEncoding::Json),
        commitment: Some(CommitmentConfig::confirmed()),
        max_supported_transaction_version: Some(0),
    };
    match rpc_client.get_transaction_with_config(&sig, config).await {
        Ok(transaction) => {
            let parsed = transaction.transaction.meta.clone();
            let unwanted_key = env::var("JUP_PUBKEY").expect("JUP_PUBKEY not set");
            let needed_key = env::var("RAY_PUBKEY").expect("RAY_PUBKEY not set");
            let sol_address = env::var("SOL_PUBKEY").expect("SOL_PUBKEY not set");
            let target = env::var("TARGET_PUBKEY").expect("TARGET_PUBKEY not set");
            let raydium_authority_v4 =
                env::var("RAY_AUTHORITY_V4").expect("RAY_AUTHORITY_V4 not set");
            match parsed {
                Some(tx) => {
                    let msg = tx.log_messages.clone();
                    match msg.clone() {
                        OptionSerializer::Some(msg_log) => {
                            if msg_log
                                .iter()
                                .any(|logs| logs.clone().contains(&unwanted_key))
                            {
                                // Handle the case where an unwanted key is found
                                return Ok(ParseTx {
                                    type_tx: "".to_string(),
                                    direction: None,
                                    amount_in: 0_f64,
                                    amount_out: 0_f64,
                                    mint: "".to_string(),
                                });
                            } else {
                                if msg_log
                                    .iter()
                                    .any(|logs| logs.clone().contains(&needed_key))
                                {
                                    let ixs =
                                        match tx.inner_instructions.clone() {
                                            OptionSerializer::Some(ixs) => ixs,
                                            OptionSerializer::None => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                            OptionSerializer::Skip => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                        };
                                    let mut mint = "";
                                    let mut token_amount_post = 0.0;
                                    let mut token_amount_pre = 0.0;
                                    let mut sol_amount_pre = 0.0;
                                    let mut sol_amount_post = 0.0;
                                    let post_token_balances =
                                        match tx.post_token_balances.clone() {
                                            OptionSerializer::Some(token_balance) => token_balance,
                                            OptionSerializer::None => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                            OptionSerializer::Skip => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                        };
                                    for post_token_balance in post_token_balances.iter() {
                                        let mut owner = match post_token_balance.owner.clone() {
                                            OptionSerializer::Some(str_owner) => str_owner,
                                            OptionSerializer::None => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                            OptionSerializer::Skip => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                        };
                                        if owner == target {
                                            mint = &post_token_balance.mint;
                                            token_amount_post = match post_token_balance
                                                .ui_token_amount
                                                .ui_amount
                                                .clone()
                                            {
                                                Some(amount) => amount,
                                                None => 0.0,
                                            };
                                        }
                                        if owner == raydium_authority_v4
                                            && post_token_balance.mint == sol_address
                                        {
                                            sol_amount_post = match post_token_balance
                                                .ui_token_amount
                                                .ui_amount
                                                .clone()
                                            {
                                                Some(amount) => amount,
                                                None => 0.0,
                                            };
                                        }
                                    }
                                    let pre_token_balances =
                                        match tx.pre_token_balances.clone() {
                                            OptionSerializer::Some(token_balance) => token_balance,
                                            OptionSerializer::None => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                            OptionSerializer::Skip => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                        };
                                    for pre_token_balance in pre_token_balances.iter() {
                                        let owner = match pre_token_balance.owner.clone() {
                                            OptionSerializer::Some(str_owner) => str_owner,
                                            OptionSerializer::None => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                            OptionSerializer::Skip => return Err(
                                                "No log messages found in transaction metadata."
                                                    .to_string(),
                                            ),
                                        };
                                        if owner == target {
                                            token_amount_pre = match pre_token_balance
                                                .ui_token_amount
                                                .ui_amount
                                                .clone()
                                            {
                                                Some(amount) => amount,
                                                None => 0.0,
                                            };
                                        }
                                        if owner == raydium_authority_v4
                                            && pre_token_balance.mint == sol_address
                                        {
                                            sol_amount_pre = match pre_token_balance
                                                .ui_token_amount
                                                .ui_amount
                                                .clone()
                                            {
                                                Some(amount) => amount,
                                                None => 0.0,
                                            };
                                        }
                                    }
                                    if token_amount_post - token_amount_pre > 0.0 {
                                        println!(
                                        "Swap In_mint: {:#?} In_amount: {:#?} Out_mint: {:#?}  Out_amount: {:#?}",
                                        sol_address,
                                        sol_amount_post - sol_amount_pre,
                                        mint,
                                        token_amount_post - token_amount_pre,
                                    );
                                        return Ok(ParseTx {
                                            // Placeholder return
                                            type_tx: "Swap".to_string(),
                                            direction: Some("buy".to_string()),
                                            amount_in: sol_amount_post - sol_amount_pre,
                                            amount_out: token_amount_post - token_amount_pre,
                                            mint: mint.to_string(),
                                        });
                                    } else {
                                        println!(
                                        "Swap In_mint: {:#?} In_amount: {:#?} Out_mint: {:#?} Out_amount: {:#?}",
                                        mint,
                                        token_amount_pre - token_amount_post,
                                        sol_address,
                                        sol_amount_pre - sol_amount_post,
                                    );
                                        return Ok(ParseTx {
                                            // Placeholder return
                                            type_tx: "Swap".to_string(),
                                            direction: Some("sell".to_string()),
                                            amount_in: token_amount_pre - token_amount_post,
                                            amount_out: sol_amount_pre - sol_amount_post,
                                            mint: mint.to_string(),
                                        });
                                    }
                                }
                            }
                            return Ok(ParseTx {
                                // Placeholder return
                                type_tx: "".to_string(),
                                direction: None,
                                amount_in: 0.0,
                                amount_out: 0.0,
                                mint: "".to_string(),
                            });
                        }
                        OptionSerializer::None => {
                            return Err("No log messages found in transaction metadata.".to_string())
                        }
                        OptionSerializer::Skip => {
                            return Err("No log messages found in transaction metadata.".to_string())
                        }
                    };
                }
                None => {
                    return Err("Not able to parse transaction metadata!".to_string());
                }
            };
        }
        Err(e) => {
            eprintln!("Failed to fetch transaction: {}", e);
            return Err(format!("Failed to fetch transaction: {}", e));
        }
    }
}
