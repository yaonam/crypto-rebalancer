pub struct Data {
    pub ask_price: f64,
    pub ask_volume: f64,
    pub bid_price: f64,
    pub bid_volume: f64,
    pub volume: f64,
}

pub struct LimitOrder {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub time: DateTime<Utc>,
}

pub struct OrderFilled {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub status: String,
    pub side: String,
    pub time: DateTime<Utc>,
}

pub struct OrderCancelled {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub status: String,
    pub side: String,
    pub time: DateTime<Utc>,
}
