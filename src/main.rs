use crate::order_book::OrderBook;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod limit;
mod order_book;

fn get_id() -> usize {
    static COUNTER: AtomicUsize = AtomicUsize::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    tracing::info!("Starting up matcher-rs");
    let mut order_book = OrderBook::new();
    let i = 20_000;
    let now = Instant::now();
    for _ in 0..i {
        order_book.process_command(OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: 122,
            qty: 1,
        });
        order_book.process_command(OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: 122,
            qty: 1,
        });
    }
    tracing::info!("Time to place {:?} orders: {:?}", i * 2, now.elapsed());
    tracing::info!("Avg time per order: {:?}", now.elapsed() / i * 2);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OrderCommand {
    New {
        order_type: OrderType,
        side: Side,
        price: i32,
        qty: u32,
    },
    Modify,
    Cancel,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OrderEvent {
    Placed {
        id: usize,
        side: Side,
        order_type: OrderType,
        price: i32,
        timestamp: Instant,
    },
    Modified,
    Canceled,
    PartiallyFilled {
        id: usize,
        price: i32,
        qty: u32,
        timestamp: Instant,
    },
    Filled {
        id: usize,
        price: i32,
        timestamp: Instant,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Order {
    id: usize,
    order_type: OrderType,
    side: Side,
    price: i32,
    initial_qty: u32,
    remaining_qty: u32,
    created_at: Instant,
    updated_at: Instant,
}

impl Order {
    fn new(order_type: OrderType, side: Side, price: i32, qty: u32) -> Order {
        let now = Instant::now();
        Order {
            id: get_id(),
            order_type,
            side,
            price,
            initial_qty: qty,
            remaining_qty: qty,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn fill(&mut self, qty: u32) -> Result<Order, ()> {
        if qty > self.remaining_qty {
            return Err(());
        }
        let new_rem_qty = self.remaining_qty - qty;
        Ok(Order {
            id: self.id.clone(),
            order_type: self.order_type,
            side: self.side,
            price: self.price,
            initial_qty: self.initial_qty,
            remaining_qty: new_rem_qty,
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
