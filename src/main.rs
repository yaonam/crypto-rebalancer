use dotenv::dotenv;
use rebalancer::account::{Portfolio, Signer};
use rebalancer::task;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let signer = Arc::new(Mutex::new(
        Signer::new(
            std::env::var("KRAKEN_KEY").expect("KRAKEN_KEY not set"),
            std::env::var("KRAKEN_SECRET").expect("KRAKEN_SECRET not set"),
        )
        .await,
    ));

    // {
    //     let signer = signer.lock().unwrap();
    //     let token = signer.get_ws_token().await;
    //     println!("{}", token);

    //     let balances = signer.get_account_balances().await;
    //     println!("{:?}", balances);
    // }

    let portfolio = Arc::new(Mutex::new(Portfolio::new(signer.clone()).await));

    let eth = "ETH/USD".to_string();
    let btc = "XBT/USD".to_string(); // bitcoin
    let sol = "SOL/USD".to_string(); // solana
    let arb = "ARB/USD".to_string(); // arbitrum

    // Wait a bit for the portfolio to be initialized.
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    let task_eth = task::spawn(eth, portfolio.clone(), signer.clone()).await;
    let task_btc = task::spawn(btc, portfolio.clone(), signer.clone()).await;
    let task_sol = task::spawn(sol, portfolio.clone(), signer.clone()).await;
    let task_arb = task::spawn(arb, portfolio.clone(), signer.clone()).await;

    let _ = tokio::select! {
        _ = task_eth => (),
        _ = task_btc => (),
        _ = task_sol => (),
        _ = task_arb => (),
        _ = ctrl_c() => (), // Graceful shutdown
    };
    println!("Exiting...");
    // let _ = tokio::join!(task1, task2);
}
