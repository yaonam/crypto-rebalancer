use dotenv::dotenv;
use rebalancer::account::{Portfolio, Signer};
use rebalancer::task;
use std::sync::{Arc, Mutex};
use tokio::signal::ctrl_c;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let signer = Signer::new(
        std::env::var("KRAKEN_KEY").expect("KRAKEN_KEY not set"),
        std::env::var("KRAKEN_SECRET").expect("KRAKEN_SECRET not set"),
    );
    let result = signer.sign("".to_string(), "".to_string());
    println!("{}", result);

    // let pair1 = "ETH/USD".to_string();
    // let pair2 = "STORJ/USD".to_string();

    // let portfolio = Arc::new(Mutex::new(Portfolio::new()));

    // let task1 = tokio::spawn(task::start(pair1, portfolio.clone()));
    // let task2 = tokio::spawn(task::start(pair2, portfolio.clone()));

    // let _ = tokio::select! {
    //     _ = task1 => (),
    //     _ = task2 => (),
    //     _ = ctrl_c() => (), // Graceful shutdown
    // };
}
