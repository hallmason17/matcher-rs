use crate::order_book::OrderBook;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid7::Uuid;

mod order_book;

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    tracing::info!("Starting up matcher-rs");
    let order_price = 122;
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    let mut order_book = OrderBook::new();
    order_book.place_order(order);
    order_book.match_orders();
    let order1 = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
    order_book.place_order(order1);
    order_book.match_orders();

    assert_eq!(order_book.get_bids().len(), 0);
    assert_eq!(order_book.get_asks().len(), 0);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Order {
    order_id: Uuid,
    order_type: OrderType,
    order_side: Side,
    order_price: i32,
    order_init_qty: u32,
    order_rem_qty: u32,
    created_at: SystemTime,
    updated_at: SystemTime,
}

#[allow(dead_code)]
impl Order {
    fn new(order_type: OrderType, order_side: Side, order_price: i32, order_qty: u32) -> Order {
        let now = SystemTime::now();
        Order {
            order_id: uuid7::uuid7(),
            order_type,
            order_side,
            order_price,
            order_init_qty: order_qty,
            order_rem_qty: order_qty,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn fill(&mut self, qty: u32) {
        if qty > self.order_rem_qty {
            return;
        }
        self.order_rem_qty -= qty;
        self.updated_at = SystemTime::now();
    }

    pub fn is_filled(&self) -> bool {
        self.order_rem_qty == 0
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct OrderBookLevelsInfo {
    bids: Vec<LevelInfo>,
    asks: Vec<LevelInfo>,
}

#[allow(dead_code)]
impl OrderBookLevelsInfo {
    pub fn new() -> Self {
        OrderBookLevelsInfo {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct LevelInfo {
    price: i32,
    quantity: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderType {
    FillAndKill,
    GoodTilCancel,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Buy,
    Sell,
}
