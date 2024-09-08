use crate::{limit::Limit, Order, OrderCommand, OrderEvent, Side};
use std::{borrow::BorrowMut, cmp::Ordering, time::Instant};

#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    pub bids: Vec<Limit>,
    pub asks: Vec<Limit>,
    events: Vec<OrderEvent>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn process_command(&mut self, command: OrderCommand) {
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
            _ => {
                todo!()
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
                match self.try_match_order(&mut order) {
                    MatchStatus::Done => {}
                    MatchStatus::Pending => {
                        let ord = Order {
                            id: order.id,
                            side: order.side,
                            remaining_qty: order.remaining_qty - order_try_match.remaining_qty,
                            price: order.price,
                            order_type: order.order_type,
                            initial_qty: order.initial_qty,
                            created_at: order.created_at,
                            updated_at: Instant::now(),
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
            if let Some(lim_pos) = queue.iter().position(|lim| lim.price == order.price) {
                let lim = queue[lim_pos].borrow_mut();
                lim.orders.push_back(order);
            } else {
                let mut new_lim = Limit::new(order.price);
                new_lim.orders.push_back(order);
                queue.push(new_lim);
            }
        }
    }

    fn find_order_to_match(&mut self, order: &Order) -> Option<Order> {
        let order_to_match: Option<Order> = {
            let queue_to_match = match order.side {
                Side::Buy => {
                    self.asks.sort();
                    &mut self.asks
                }
                Side::Sell => {
                    self.bids
                        .sort_by(|a, b| b.price.partial_cmp(&a.price).unwrap());
                    &mut self.bids
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

    fn try_match_order(&mut self, order: &mut Order) -> MatchStatus {
        let order_to_match = self.find_order_to_match(order);
        let timestamp = Instant::now();
        if let Some(order_try_match) = order_to_match {
            let can_match = match order.side {
                Side::Buy => order.price >= order_try_match.price,
                Side::Sell => order.price <= order_try_match.price,
            };
            if can_match {
                match order.remaining_qty.cmp(&order_try_match.remaining_qty) {
                    Ordering::Greater => {
                        let lim_vec = match order.side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let opp_ord = lim.orders.pop_front().unwrap();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
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
                        MatchStatus::Pending
                    }
                    Ordering::Less => {
                        let lim_vec = match order.side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let mut opp_ord = lim.orders.front().unwrap().to_owned();
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
                        MatchStatus::Done
                    }
                    _ => {
                        let lim_vec = match order.side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let opp_ord = lim.orders.pop_front().unwrap();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
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
                        MatchStatus::Done
                    }
                };
            }
        }
        MatchStatus::Pending
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
