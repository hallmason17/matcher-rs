use tracing;

use crate::{Order, OrderType, Side};
use std::{
    cmp::min,
    collections::{HashMap, VecDeque},
};
#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    bids: HashMap<i32, VecDeque<Order>>,
    asks: HashMap<i32, VecDeque<Order>>,
}

#[allow(dead_code)]
impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    pub fn get_bids(&self) -> &HashMap<i32, VecDeque<Order>> {
        return &self.bids;
    }

    pub fn get_asks(&self) -> &HashMap<i32, VecDeque<Order>> {
        return &self.asks;
    }

    pub fn place_order(&mut self, order: Order) {
        if order.order_init_qty == 0 {
            return;
        }

        if order.order_type == OrderType::FillAndKill
            && !self.can_match_order(order.order_side, order.order_price)
        {
            return;
        }

        match order.order_side {
            Side::Buy => self.add_bid(order),
            Side::Sell => self.add_ask(order),
        }
    }

    pub fn match_orders(&mut self) {
        loop {
            if self.bids.is_empty() || self.asks.is_empty() {
                tracing::info!("Bids or asks empty, nothing to match");
                break;
            }
            if let Some(bid_price) = &self.get_best_bid() {
                if let Some(ask_price) = &self.get_best_ask() {
                    if bid_price < ask_price {
                        break;
                    }
                    let mut new_bids = self.bids.get(bid_price).unwrap().clone();
                    let mut new_asks = self.asks.get(ask_price).unwrap().clone();
                    if let Some(bid) = new_bids.front() {
                        if let Some(ask) = new_asks.front() {
                            let qty = min(bid.order_rem_qty, ask.order_rem_qty);
                            let new_bid = bid.to_owned();
                            let new_ask = ask.to_owned();
                            self.fill_order(new_bid, qty);

                            tracing::info!(
                                "Filled order {:?} for {:?} quantity, {:?} remaining to fill",
                                new_bid.order_id,
                                qty,
                                new_bid.order_rem_qty
                            );

                            self.fill_order(new_ask, qty);
                            tracing::info!(
                                "Filled order {:?} for {:?} quantity, {:?} remaining to fill",
                                new_ask.order_id,
                                qty,
                                new_ask.order_rem_qty
                            );

                            if new_bid.is_filled() {
                                tracing::info!(
                                    "Order {:?} filled, removing from queue",
                                    new_bid.order_id
                                );
                                new_bids.pop_front();
                                self.set_bids_queue(bid_price, new_bids);
                            }

                            if new_ask.is_filled() {
                                tracing::info!(
                                    "Order {:?} filled, removing from queue",
                                    new_ask.order_id
                                );
                                new_asks.pop_front();
                                self.set_asks_queue(ask_price, new_asks);
                            }
                        }
                    }
                }
            }
        }
    }

    fn remove_order(&mut self, order: Order) {
        match order.order_side {
            Side::Buy => {
                if let Some(q) = self.bids.get(&order.order_price) {
                    let mut queue = q.clone();
                    queue.retain(|x| x.order_id != order.order_id);
                    self.set_bids_queue(&order.order_price, queue);
                }
            }
            Side::Sell => {
                if let Some(q) = self.asks.get(&order.order_price) {
                    let mut queue = q.clone();
                    queue.retain(|x| x.order_id != order.order_id);
                    self.set_asks_queue(&order.order_price, queue);
                }
            }
        }
    }

    fn fill_order(&mut self, mut order: Order, qty: u32) {
        order.fill(qty);
        match order.order_side {
            Side::Buy => {
                if let Some(q) = self.bids.get(&order.order_price) {
                    let mut queue = q.clone();
                    queue.pop_front();
                    queue.push_front(order);
                    self.set_bids_queue(&order.order_price, queue);
                }
            }
            Side::Sell => {
                if let Some(q) = self.asks.get(&order.order_price) {
                    let mut queue = q.clone();
                    queue.pop_front();
                    queue.push_front(order);
                    self.set_asks_queue(&order.order_price, queue);
                }
            }
        }
    }

    fn set_bids_queue(&mut self, price: &i32, new_bids: VecDeque<Order>) {
        if new_bids.len() == 0 {
            self.bids.remove(&price);
        }
        if let Some(q) = self.bids.get_mut(&price) {
            *q = new_bids;
        }
    }

    fn set_asks_queue(&mut self, price: &i32, new_asks: VecDeque<Order>) {
        if new_asks.len() == 0 {
            self.asks.remove(&price);
        }
        if let Some(q) = self.asks.get_mut(&price) {
            *q = new_asks;
        }
    }

    fn add_bid(&mut self, order: Order) {
        let mut bids = VecDeque::new();
        bids.push_back(order);
        self.bids
            .entry(order.order_price)
            .and_modify(|orders| orders.push_back(order))
            .or_insert(bids);
    }

    fn get_best_ask(&self) -> Option<i32> {
        if self.asks.is_empty() {
            return None;
        }
        let mut best_ask: &i32 = &i32::MAX;
        for (k, _) in self.asks.iter() {
            if k <= best_ask {
                best_ask = k
            }
        }
        return Some(*best_ask);
    }

    fn get_best_bid(&self) -> Option<i32> {
        if self.bids.is_empty() {
            return None;
        }
        let mut best_bid: &i32 = &i32::MIN;
        for (k, _) in self.bids.iter() {
            if k >= best_bid {
                best_bid = k
            }
        }
        return Some(*best_bid);
    }

    fn add_ask(&mut self, order: Order) {
        let mut asks = VecDeque::new();
        asks.push_back(order);
        self.asks
            .entry(order.order_price)
            .and_modify(|orders| orders.push_back(order))
            .or_insert(asks);
    }

    fn can_match_order(&self, side: Side, price: i32) -> bool {
        match side {
            Side::Buy => {
                if self.asks.is_empty() {
                    return false;
                }
                match self.get_best_ask() {
                    Some(best_ask) => {
                        println!("{:?} {:?}", price, best_ask);
                        return price >= best_ask;
                    }
                    None => return false,
                }
            }
            Side::Sell => {
                if self.bids.is_empty() {
                    return false;
                }
                match self.get_best_bid() {
                    Some(best_bid) => {
                        println!("{:?} {:?}", price, best_bid);
                        return price <= best_bid;
                    }
                    None => return false,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use std::collections::VecDeque;

    use crate::order_book::OrderBook;
    use crate::{Order, OrderType, Side};

    #[test]
    fn remove_ask_order() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
        order_book.place_order(order);
        assert_eq!(order_book.get_asks().len(), 1);
        if let Some(q) = order_book.get_asks().get(&order_price) {
            let mut vecd = VecDeque::new();
            vecd.push_back(order);
            assert_eq!(q, &vecd);
        }
        order_book.remove_order(order);
        assert_eq!(order_book.get_asks().len(), 0);
    }

    #[test]
    fn remove_bid_order() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
        order_book.place_order(order);
        assert_eq!(order_book.get_bids().len(), 1);
        if let Some(q) = order_book.get_bids().get(&order_price) {
            let mut vecd = VecDeque::new();
            vecd.push_back(order);
            assert_eq!(q, &vecd);
        }
        order_book.remove_order(order);
        assert_eq!(order_book.get_bids().len(), 0);
    }

    #[test]
    fn match_orders() {
        let order_price = 122;
        let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        let order1 = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
        order_book.place_order(order1);
        assert_eq!(order_book.get_bids().len(), 1);
        assert_eq!(order_book.get_asks().len(), 1);

        order_book.match_orders();
        assert_eq!(order_book.get_bids().len(), 0);
        assert_eq!(order_book.get_asks().len(), 0);
    }

    #[test]
    fn add_bid_order() {
        let order_price = 122;
        let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        assert_eq!(order_book.get_asks().len(), 1);
        assert_eq!(order_book.get_best_ask(), Some(order_price));
    }

    #[test]
    fn add_ask_order() {
        let order_price = 123;
        let order = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        assert_eq!(order_book.get_bids().len(), 1);
        assert_eq!(order_book.get_best_bid(), Some(order_price));
    }

    #[test]
    fn can_match_sell_order() {
        let buy_order = Order::new(OrderType::GoodTilCancel, Side::Buy, 123, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(buy_order);

        assert_eq!(order_book.can_match_order(Side::Sell, 122), true);
    }

    #[test]
    fn cannot_match_sell_order() {
        let buy_order = Order::new(OrderType::GoodTilCancel, Side::Buy, 120, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(buy_order);

        assert_eq!(order_book.can_match_order(Side::Sell, 122), false);
    }

    #[test]
    fn can_match_buy_order() {
        let sell_order = Order::new(OrderType::GoodTilCancel, Side::Sell, 118, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(sell_order);

        assert_eq!(order_book.can_match_order(Side::Buy, 120), true);
    }

    #[test]
    fn cannot_match_buy_order() {
        let sell_order = Order::new(OrderType::GoodTilCancel, Side::Sell, 123, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(sell_order);

        assert_eq!(order_book.can_match_order(Side::Buy, 120), false);
    }
}
