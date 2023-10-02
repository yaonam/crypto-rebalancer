use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;

mod public;
pub use public::*;
mod private;
pub use private::*;
mod open_orders;
pub use open_orders::*;
mod misc;
pub use misc::*;

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum WSPayload {
    PublicMessage(PublicMessage),
    OpenOrders(OpenOrders),
    // OwnTrades(OwnTradesData),
    SystemStatus(SystemStatus),
    SubscriptionStatus(SubscriptionStatus),
    Heartbeat(Heartbeat),
}
