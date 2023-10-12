pub enum Data {
    OHLC(OHLCData),
    Trade(TradeData),
    OrderBook(OrderBookData),
}

pub struct OHLCData {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub time: String,
}

pub struct TradeData {
    pub price: f64,
    pub volume: f64,
    pub time: String,
}

pub struct OrderData {
    pub price: f64,
    pub volume: f64,
}

pub struct OrderBookData {
    pub asks: Vec<OrderData>,
    pub bids: Vec<OrderData>,
}

pub struct LimitOrder {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub time: String,
}

pub struct OrderFilled {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub status: String,
    pub side: String,
    pub time: String,
}

pub struct OrderCancelled {
    pub id: String,
    pub asset: String,
    pub amount: f64,
    pub price: f64,
    pub status: String,
    pub side: String,
    pub time: String,
}
