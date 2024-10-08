// Copyright 2024 Mason Hall. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::{level::Level, Order, OrderCommand, OrderEvent, Side};
use std::{borrow::BorrowMut, cmp::Ordering, time::Instant};

#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    pub bids: Vec<Level>,
    pub asks: Vec<Level>,
    commands: Vec<OrderCommand>,
    events: Vec<OrderEvent>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            commands: Vec::with_capacity(200_000),
            events: Vec::with_capacity(200_000),
        }
    }

    pub fn process_command(&mut self, command: OrderCommand) {
        self.commands.push(command.clone());
        match command {
            OrderCommand::New {
                order_type,
                side,
                price,
                qty,
            } => {
                let order = Order::new(order_type, side, price, qty);
                self.events.push(OrderEvent::Placed {
                    id: order.id,
                    side: order.side,
                    order_type: order.order_type,
                    price,
                    timestamp: order.created_at,
                });
                self.place_order(order);
            }
            OrderCommand::Cancel { id, side, price } => self.remove_order(id, price, side),
            OrderCommand::Modify {
                id,
                side,
                price,
                qty,
                order_type,
            } => {
                let queue = match side {
                    Side::Buy => &mut self.bids,
                    Side::Sell => &mut self.asks,
                };
                if let Some(lev_pos) = queue.iter().position(|lev| lev.price == price) {
                    let level = queue[lev_pos].borrow_mut();
                    if let Some(order_pos) = level.find_by_id(id) {
                        let order = level.orders[order_pos].clone();
                        self.process_command(OrderCommand::Cancel { id, side, price });
                        self.process_command(OrderCommand::New {
                            order_type,
                            side: order.side,
                            price,
                            qty,
                        })
                    }
                }
            }
        }
    }

    fn remove_order(&mut self, id: usize, price: i32, side: Side) {
        let queue = match side {
            Side::Buy => &mut self.bids,
            Side::Sell => &mut self.asks,
        };
        if let Some(lev_pos) = queue.iter().position(|lev| lev.price == price) {
            let lev = queue[lev_pos].borrow_mut();
            if lev.remove_order_by_id(id) == true {
                self.events.push(OrderEvent::Canceled { id })
            }
        }
    }

    pub fn place_order(&mut self, mut order: Order) {
        if order.initial_qty == 0 {
            return;
        }
        let order_to_match = self.find_order_to_match(&order);

        if let Some(order_try_match) = order_to_match {
            let can_match = match order.side {
                Side::Buy => order.price >= order_try_match.price,
                Side::Sell => order.price <= order_try_match.price,
            };
            if can_match {
                match self.try_match_order(&mut order, &order_try_match) {
                    MatchStatus::Done => {}
                    MatchStatus::Pending => {
                        let ord = Order {
                            remaining_qty: order.remaining_qty - order_try_match.remaining_qty,
                            updated_at: Instant::now(),
                            ..order
                        };
                        self.place_order(ord);
                    }
                }
            }
        } else {
            let queue = match order.side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            if let Some(lev_pos) = queue.iter().position(|lev| lev.price == order.price) {
                let lev = queue[lev_pos].borrow_mut();
                lev.orders.push_back(order);
            } else {
                let mut new_lev = Level::new(order.price);
                new_lev.orders.push_back(order);
                queue.push(new_lev);
            }
        }
    }

    fn find_order_to_match(&mut self, order: &Order) -> Option<Order> {
        let order_to_match: Option<Order> = {
            let queue_to_match = match order.side {
                Side::Buy => {
                    self.asks.sort();
                    &self.asks
                }
                Side::Sell => {
                    self.bids
                        .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                    &self.bids
                }
            };
            if !queue_to_match.is_empty() {
                queue_to_match.first().unwrap().orders.front().cloned()
            } else {
                None
            }
        };
        order_to_match
    }

    fn try_match_order(&mut self, order: &mut Order, match_order: &Order) -> MatchStatus {
        let timestamp = Instant::now();
        match order.remaining_qty.cmp(&match_order.remaining_qty) {
            Ordering::Greater => {
                let lev_vec = match order.side {
                    Side::Buy => &mut self.asks,
                    Side::Sell => &mut self.bids,
                };
                if let Some(lev_pos) = lev_vec
                    .iter()
                    .position(|lev| lev.price == match_order.price)
                {
                    let lev = lev_vec[lev_pos].borrow_mut();
                    let opp_ord = lev.orders.pop_front().unwrap();
                    if lev.orders.is_empty() {
                        lev_vec.remove(lev_pos);
                    }
                    self.events.push(OrderEvent::PartiallyFilled {
                        id: order.id,
                        price: order.price,
                        qty: opp_ord.remaining_qty,
                        timestamp,
                    });
                    self.events.push(OrderEvent::Filled {
                        id: opp_ord.id,
                        price: order.price,
                        timestamp,
                    });
                    return MatchStatus::Pending;
                }
                return MatchStatus::Pending;
            }
            Ordering::Less => {
                let lev_vec = match order.side {
                    Side::Buy => &mut self.asks,
                    Side::Sell => &mut self.bids,
                };
                if let Some(lev_pos) = lev_vec
                    .iter()
                    .position(|lev| lev.price == match_order.price)
                {
                    let lev = lev_vec[lev_pos].borrow_mut();
                    let mut opp_ord = lev.orders.front().unwrap().to_owned();
                    let _ = opp_ord.fill(order.remaining_qty);
                    self.events.push(OrderEvent::PartiallyFilled {
                        id: opp_ord.id,
                        price: order.price,
                        qty: order.remaining_qty,
                        timestamp,
                    });
                    self.events.push(OrderEvent::Filled {
                        id: order.id,
                        price: order.price,
                        timestamp,
                    });
                    return MatchStatus::Done;
                };
                return MatchStatus::Done;
            }
            _ => {
                let lev_vec = match order.side {
                    Side::Buy => &mut self.asks,
                    Side::Sell => &mut self.bids,
                };
                if let Some(lev_pos) = lev_vec
                    .iter()
                    .position(|lev| lev.price == match_order.price)
                {
                    let lev = lev_vec[lev_pos].borrow_mut();
                    let opp_ord = lev.orders.pop_front().unwrap();
                    if lev.orders.is_empty() {
                        lev_vec.remove(lev_pos);
                    }
                    self.events.push(OrderEvent::Filled {
                        id: opp_ord.id,
                        price: order.price,
                        timestamp,
                    });
                    self.events.push(OrderEvent::Filled {
                        id: order.id,
                        price: order.price,
                        timestamp,
                    });
                    return MatchStatus::Done;
                }
                return MatchStatus::Done;
            }
        }
    }
}

enum MatchStatus {
    Pending,
    Done,
}

#[cfg(test)]
mod tests {

    use crate::order_book::OrderBook;
    use crate::{OrderCommand, OrderType, Side};

    #[test]
    fn test_match_multiple_orders() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: order_price,
            qty: 5,
        };
        order_book.process_command(order);

        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn match_orders_diff_prices() {
        let mut order_book = OrderBook::new();
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: 123,
            qty: 1,
        };
        let order1 = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: 124,
            qty: 1,
        };
        let order2 = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: 122,
            qty: 1,
        };
        let order3 = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: 122,
            qty: 1,
        };
        order_book.process_command(order);
        order_book.process_command(order1);
        order_book.process_command(order2);
        order_book.process_command(order3);
        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn test_match_orders() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);

        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn add_bid_order() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        assert_eq!(order_book.bids.len(), 1);
    }

    #[test]
    fn add_ask_order() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = OrderCommand::New {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        assert_eq!(order_book.asks.len(), 1);
    }
}
