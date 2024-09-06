use crate::order_book::OrderBook;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;

use std::io::Write;

use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod limit;
mod order_book;

fn main() {
    tracing_subscriber::registry().with(fmt::layer()).init();
    tracing::info!("Starting up matcher-rs");
    let mut order_book = OrderBook::new();
    let buy_order = OrderPub::new(OrderType::GoodTilCancel, Side::Buy, 122, 1);
    let sell_order = OrderPub::new(OrderType::GoodTilCancel, Side::Sell, 122, 1);
    order_book.place_order(buy_order.convert_to_order());
    order_book.place_order(sell_order.convert_to_order());
    let mut order_book = OrderBook::new();
    let mut file = std::fs::File::create("order.json").unwrap();

    let now = Instant::now();
    let mut order_vec = Vec::new();
    order_vec.push(OrderPub::new(OrderType::GoodTilCancel, Side::Sell, 125, 1));
    for _ in 0..500_000 {
        order_vec.push(OrderPub::new(OrderType::GoodTilCancel, Side::Sell, 122, 1));
    }
    order_vec.push(OrderPub::new(OrderType::GoodTilCancel, Side::Sell, 125, 1));
    for _ in 0..500_000 {
        order_vec.push(OrderPub::new(OrderType::GoodTilCancel, Side::Buy, 122, 1));
    }
    let json = serde_json::to_string(&order_vec).unwrap();
    file.write_all(json.as_bytes()).unwrap();
    tracing::info!("Time taken to write to file: {:?}", now.elapsed());

    let now = Instant::now();
    let file = std::fs::File::open("order.json").unwrap();
    let reader = std::io::BufReader::new(file);
    let _orders: Vec<OrderPub> = serde_json::from_reader(reader).unwrap();
    tracing::info!("Time taken to read from file: {:?}", now.elapsed());

    let mut orders = Vec::new();
    for order_pub in order_vec {
        orders.push(order_pub.convert_to_order());
    }

    let mut times = Vec::new();
    for order in orders {
        let now = Instant::now();
        order_book.place_order(order);
        times.push(now.elapsed());
    }

    let sum: Duration = times.iter().sum::<Duration>();
    tracing::info!("Total time to place orders: {:?} ", sum);
    let avg = sum / times.len().try_into().unwrap();
    tracing::info!("Avg time to place order: {:?}", avg);
    dbg!(&order_book.bids.len());
    dbg!(&order_book.asks.len());
    dbg!(&order_book);
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct OrderPub {
    order_type: OrderType,
    order_side: Side,
    order_price: i32,
    order_qty: u32,
}

impl OrderPub {
    pub fn new(
        order_type: OrderType,
        order_side: Side,
        order_price: i32,
        order_qty: u32,
    ) -> OrderPub {
        OrderPub {
            order_type,
            order_side,
            order_price,
            order_qty,
        }
    }

    pub fn convert_to_order(self) -> Order {
        Order::new(
            self.order_type,
            self.order_side,
            self.order_price,
            self.order_qty,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
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
