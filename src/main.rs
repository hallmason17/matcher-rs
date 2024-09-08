use crate::order_book::OrderBook;
use serde::{Deserialize, Serialize};
use std::time::Instant;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod limit;
mod order_book;

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    tracing::info!("Starting up matcher-rs");
    let mut order_book = OrderBook::new();
    let buy_order = Commands::NewOrder {
        order_type: OrderType::GoodTilCancel,
        side: Side::Buy,
        price: 122,
        qty: 1,
    };
    let sell_order = Commands::NewOrder {
        order_type: OrderType::GoodTilCancel,
        side: Side::Sell,
        price: 122,
        qty: 1,
    };
    order_book.process_command(buy_order);
    order_book.process_command(sell_order);

    dbg!(&order_book.bids.len());
    dbg!(&order_book.asks.len());
    dbg!(&order_book);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Commands {
    NewOrder {
        order_type: OrderType,
        side: Side,
        price: i32,
        qty: u32,
    },
    ModifyOrder,
    CancelOrder,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Events {
    OrderPlaced {
        order_id: String,
        order_side: Side,
        order_type: OrderType,
        timestamp: Instant,
    },
    OrderModified,
    OrderCanceled,
    OrderPartiallyFilled {
        order_id: String,
        qty: u32,
        timestamp: Instant,
    },
    OrderFilled {
        order_id: String,
        timestamp: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Order {
    order_id: String,
    order_type: OrderType,
    order_side: Side,
    order_price: i32,
    order_init_qty: u32,
    order_rem_qty: u32,
    created_at: Instant,
    updated_at: Instant,
}

impl Order {
    fn new(order_type: OrderType, order_side: Side, order_price: i32, order_qty: u32) -> Order {
        let now = Instant::now();
        Order {
            order_id: uuid7::uuid7().to_string(),
            order_type,
            order_side,
            order_price,
            order_init_qty: order_qty,
            order_rem_qty: order_qty,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn fill(&mut self, qty: u32) -> Result<Order, ()> {
        if qty > self.order_rem_qty {
            return Err(());
        }
        let new_rem_qty = self.order_rem_qty - qty;
        Ok(Order {
            order_id: self.order_id.clone(),
            order_type: self.order_type,
            order_side: self.order_side,
            order_price: self.order_price,
            order_init_qty: self.order_init_qty,
            order_rem_qty: new_rem_qty,
            created_at: self.created_at,
            updated_at: Instant::now(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
enum OrderType {
    FillAndKill,
    GoodTilCancel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Ord, PartialOrd)]
enum Side {
    Buy,
    Sell,
}
