use uuid7::Uuid;

use crate::{Order, OrderType, Side};
use std::{borrow::BorrowMut, cmp::min, collections::HashMap};
#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    bids: HashMap<i32, Vec<Order>>,
    pub asks: HashMap<i32, Vec<Order>>,
}

#[allow(dead_code)]
impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: HashMap::new(),
            asks: HashMap::new(),
        }
    }

    pub fn get_bids(&self) -> &HashMap<i32, Vec<Order>> {
        return &self.bids;
    }

    pub fn get_asks(&self) -> &HashMap<i32, Vec<Order>> {
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

    fn add_bid(&mut self, order: Order) {
        let mut bids = Vec::new();
        bids.push(order);
        self.bids
            .entry(order.order_price)
            .and_modify(|orders| orders.push(order))
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
        let mut asks = Vec::new();
        asks.push(order);
        self.asks
            .entry(order.order_price)
            .and_modify(|orders| orders.push(order))
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

    fn match_orders(&self) {
        loop {
            if self.bids.is_empty() || self.asks.is_empty() {
                break;
            }
            let bid_price = &self.get_best_bid().unwrap();
            let ask_price = &self.get_best_ask().unwrap();

            if bid_price < ask_price {
                break;
            }

            let bids = self.bids.get(bid_price).to_owned().unwrap();
            let mut bid = bids.first().unwrap().to_owned();

            let asks = self.asks.get(ask_price).to_owned().unwrap();
            let mut ask = asks.first().unwrap().to_owned();

            let qty = min(bid.order_rem_qty, ask.order_rem_qty);

            bid.fill(qty);
            ask.fill(qty);

            if bid.is_filled() {
                bids.to_owned().remove(0);
            }
        }
    }

    fn remove_order(&mut self, order_id: Uuid) {}
}

#[cfg(test)]
mod tests {

    use crate::order_book::OrderBook;
    use crate::{Order, OrderType, Side};

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
