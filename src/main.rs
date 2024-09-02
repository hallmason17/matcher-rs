use crate::order_book::OrderBook;
use std::time::SystemTime;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod order_book;

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    tracing::info!("Starting up matcher-rs");
    let order_price = 122;
    let mut order_book = OrderBook::new();
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    order_book.place_order(order);
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    order_book.place_order(order);
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    order_book.place_order(order);
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    order_book.place_order(order);
    let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
    order_book.place_order(order);
    let order1 = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price + 1, 5);
    order_book.place_order(order1);

    assert_eq!(order_book.get_bids().len(), 0);
    assert_eq!(order_book.get_asks().len(), 0);
    dbg!(order_book.get_trades());
}

enum Events {
    NewOrder {
        id: String,
        order_type: OrderType,
        order_side: Side,
        created_at: SystemTime,
    },
    OrderFilled {
        id: String,
    },
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Trade {
    trade_id: String,
    buy_ord_id: String,
    sell_ord_id: String,
    quantity: u32,
    price: i32,
}

impl Trade {
    pub fn new(buyer_id: &str, seller_id: &str, quantity: &u32, price: &i32) -> Trade {
        Trade {
            trade_id: uuid7::uuid7().to_string(),
            buy_ord_id: buyer_id.to_string(),
            sell_ord_id: seller_id.to_string(),
            quantity: *quantity,
            price: *price,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Order {
    order_id: String,
    order_type: OrderType,
    order_side: Side,
    order_price: i32,
    order_init_qty: u32,
    order_rem_qty: u32,
    created_at: SystemTime,
    updated_at: SystemTime,
}

impl Order {
    fn new(order_type: OrderType, order_side: Side, order_price: i32, order_qty: u32) -> Order {
        let now = SystemTime::now();
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
            updated_at: SystemTime::now(),
        })
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
