use dotenv::dotenv;
use rebalancer::account::{Portfolio, Signer};
use rebalancer::broker::backtest::{BacktestPortfolio, BacktestStatic};
use rebalancer::broker::broker_trait::Broker;
use rebalancer::broker::{Backtest, Kraken, KrakenStatic};
use rebalancer::strategy::ANSMM;
use std::sync::Arc;
use tokio::signal::ctrl_c;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Backtest stuff...
    // let mut broker_portoflio = BacktestPortfolio::new().await;
    // tokio::spawn(async move {
    //     broker_portoflio.start().await;
    // });
    // let mut broker = Backtest::new().await;
    // println!("Broker initialized");
    // ANSMM::new::<_, BacktestStatic>(vec!["ETH/USD"], &mut broker).await;

    // println!("Strat initialized");
    // let task = tokio::spawn(async move {
    //     broker.start().await;
    // });
    // println!("Task spawned");
    // let _ = tokio::select! {
    //     _ = task => (),
    //     _ = ctrl_c() => (), // Graceful shutdown
    // };
    // println!("Exited somehow?");

    // Kraken stuff...
    let mut broker = Kraken::<ANSMM>::new(
        std::env::var("KRAKEN_KEY").expect("KRAKEN_KEY not set"),
        std::env::var("KRAKEN_SECRET").expect("KRAKEN_SECRET not set"),
    )
    .await;
    println!("Broker initialized");
    ANSMM::new::<_, KrakenStatic>(vec!["ETH/USD", "STORJ/USD"], &mut broker).await;

    let signer = Arc::new(Mutex::new(
        Signer::new(
            std::env::var("KRAKEN_KEY").expect("KRAKEN_KEY not set"),
            std::env::var("KRAKEN_SECRET").expect("KRAKEN_SECRET not set"),
        )
        .await,
    ));

    {
        let signer = signer.lock().await;
        let token = signer.get_ws_token().await;
        println!("{}", token);

        let balances = signer.get_account_balances().await;
        println!("{:?}", balances);
    }

    // let portfolio = Arc::new(Mutex::new(Portfolio::new(signer.clone()).await));

    // let pair1 = "ETH/USD".to_string();
    // let pair2 = "STORJ/USD".to_string();

    // let task1 = task::spawn(pair1, portfolio.clone(), signer.clone()).await;
    // let task2 = task::spawn(pair2, portfolio.clone(), signer.clone()).await;

    // let _ = tokio::select! {
    //     _ = task1 => (),
    //     _ = task2 => (),
    //     _ = ctrl_c() => (), // Graceful shutdown
    // };
    // let _ = tokio::join!(task1, task2);
}
