use polygon_io_client::ws::{ForexQuoteTickMessage, StockQuoteTickMessage};
use rust_extensions::date_time::DateTimeAsMicroseconds;

use crate::BidAsk;

impl Into<BidAsk> for ForexQuoteTickMessage {
    fn into(self) -> BidAsk {
        BidAsk {
            date_time: crate::BidAskDateTime::Source(
                DateTimeAsMicroseconds::from_str(&(self.timestamp).to_string()).unwrap(),
            ),
            id: self.symbol,
            bid: self.bid,
            ask: self.ask,
            source: "polygon".to_string(),
        }
    }
}

impl Into<BidAsk> for StockQuoteTickMessage {
    fn into(self) -> BidAsk {
        BidAsk {
            date_time: crate::BidAskDateTime::Source(
                DateTimeAsMicroseconds::from_str(&(self.timestamp).to_string()).unwrap(),
            ),
            id: self.symbol,
            bid: self.bid,
            ask: self.ask,
            source: "polygon".to_string(),
        }
    }
}
