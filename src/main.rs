use rebalancer::task;

#[tokio::main]
async fn main() {
    let pair = "ETH/USD".to_string();
    let task1 = tokio::spawn(task::start(pair));
    let pair = "STORJ/USD".to_string();
    let task2 = tokio::spawn(task::start(pair));
    let _ = tokio::join!(task1, task2);
}
