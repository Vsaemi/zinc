//!
//! The zkSync account public key changer.
//!

use colored::Colorize;

use zksync_eth_signer::PrivateKeySigner;

static TOKEN_SYMBOL: &str = "ETH";
const FEE: u64 = 100_000_000_000_000_000;

static ETH_ADDRESS: &str = "36615Cf349d7F6344891B1e7CA7C72883F5dc049";
static ETH_PRIVATE_KEY: &str = "7726827caac94a7f9e1b160f7ea819f172f7b6f9d2a97f992c38edeab82d4110";

const NETWORK: zksync::Network = zksync::Network::Localhost;

///
/// The utility entry point.
///
#[actix_rt::main]
async fn main() {
    let provider = zksync::Provider::new(NETWORK);
    let wallet_credentials = zksync::WalletCredentials::from_eth_signer(
        ETH_ADDRESS.parse().expect("ETH address parsing"),
        ETH_PRIVATE_KEY
            .parse()
            .map(PrivateKeySigner::new)
            .expect("ETH private key parsing"),
        NETWORK,
    )
    .await
    .expect("Wallet credentials");
    let wallet = zksync::Wallet::new(provider, wallet_credentials)
        .await
        .expect("Wallet initialization");

    let tx_info = wallet
        .start_change_pubkey()
        .fee(FEE)
        .fee_token(TOKEN_SYMBOL)
        .expect("Fee token resolving")
        .send()
        .await
        .expect("Transaction sending")
        .wait_for_commit()
        .await
        .expect("Transaction waiting");
    if !tx_info.success.unwrap_or_default() {
        panic!(tx_info
            .fail_reason
            .unwrap_or_else(|| "Unknown error".to_owned()),);
    }

    println!("{}", "OK".bright_green());
}
