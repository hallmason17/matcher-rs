use crate::{limit::Limit, Commands, Events, Order, Side};
use std::{borrow::BorrowMut, cmp::Ordering, time::Instant};

#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    pub bids: Vec<Limit>,
    pub asks: Vec<Limit>,
    events: Vec<Events>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
            events: Vec::new(),
        }
    }

    pub fn process_command(&mut self, command: Commands) {
        match command {
            Commands::NewOrder {
                order_type,
                side,
                price,
                qty,
            } => {
                let order = Order::new(order_type, side, price, qty);
                self.events.push(Events::OrderPlaced {
                    order_id: order.order_id.clone(),
                    order_side: order.order_side,
                    order_type: order.order_type,
                    timestamp: order.created_at,
                });
                self.place_order(order.to_owned());
            }
            _ => {
                todo!()
            }
        }
    }

    pub fn place_order(&mut self, mut order: Order) {
        if order.order_init_qty == 0 {
            return;
        }
        let order_to_match = self.find_order_to_match(&order);

        if let Some(order_try_match) = order_to_match {
            let can_match = match order.order_side {
                Side::Buy => order.order_price >= order_try_match.order_price,
                Side::Sell => order.order_price <= order_try_match.order_price,
            };
            if can_match {
                if self.try_match_order(&mut order) {
                } else {
                    let ord = Order {
                        order_id: order.order_id.clone(),
                        order_side: order.order_side,
                        order_rem_qty: order.order_rem_qty - order_try_match.order_rem_qty,
                        order_price: order.order_price,
                        order_type: order.order_type,
                        order_init_qty: order.order_init_qty,
                        created_at: order.created_at,
                        updated_at: Instant::now(),
                    };
                    self.place_order(ord);
                }
            }
        } else {
            let queue = match order.order_side {
                Side::Buy => &mut self.bids,
                Side::Sell => &mut self.asks,
            };
            if let Some(lim_pos) = queue.iter().position(|lim| lim.price == order.order_price) {
                let lim = queue[lim_pos].borrow_mut();
                lim.orders.push_back(order);
            } else {
                let mut new_lim = Limit::new(order.order_price);
                new_lim.orders.push_back(order.clone());
                queue.push(new_lim.to_owned());
            }
        }
    }
    fn find_order_to_match(&mut self, order: &Order) -> Option<Order> {
        let order_to_match: Option<Order> = {
            let queue_to_match = match order.order_side {
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

    fn try_match_order(&mut self, order: &mut Order) -> bool {
        let order_to_match = self.find_order_to_match(&order);
        let timestamp = Instant::now();
        if let Some(order_try_match) = order_to_match {
            let can_match = match order.order_side {
                Side::Buy => order.order_price >= order_try_match.order_price,
                Side::Sell => order.order_price <= order_try_match.order_price,
            };
            if can_match {
                match order.order_rem_qty.cmp(&order_try_match.order_rem_qty) {
                    Ordering::Greater => {
                        let lim_vec = match order.order_side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.order_price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let opp_ord = lim.orders.pop_front().unwrap().to_owned();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
                            }
                            self.events.push(Events::OrderPartiallyFilled {
                                order_id: order.order_id.clone(),
                                qty: opp_ord.order_rem_qty,
                                timestamp,
                            });
                            self.events.push(Events::OrderFilled {
                                order_id: opp_ord.order_id,
                                timestamp,
                            });
                            return false;
                        }
                        false
                    }
                    Ordering::Less => {
                        let lim_vec = match order.order_side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.order_price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let mut opp_ord = lim.orders.front().unwrap().to_owned();
                            let _ = opp_ord.fill(order.order_rem_qty);
                            self.events.push(Events::OrderPartiallyFilled {
                                order_id: opp_ord.order_id,
                                qty: order.order_rem_qty,
                                timestamp,
                            });
                            self.events.push(Events::OrderFilled {
                                order_id: order.order_id.clone(),
                                timestamp,
                            });
                            return true;
                        };
                        true
                    }
                    _ => {
                        let lim_vec = match order.order_side {
                            Side::Buy => &mut self.asks,
                            Side::Sell => &mut self.bids,
                        };
                        if let Some(lim_pos) = lim_vec
                            .iter()
                            .position(|lim| lim.price == order_try_match.order_price)
                        {
                            let lim = lim_vec[lim_pos].borrow_mut();
                            let opp_ord = lim.orders.pop_front();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
                            }
                            self.events.push(Events::OrderFilled {
                                order_id: opp_ord.unwrap().order_id,
                                timestamp,
                            });
                            self.events.push(Events::OrderFilled {
                                order_id: order.order_id.clone(),
                                timestamp,
                            });
                            return true;
                        }
                        true
                    }
                };
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {

    use crate::order_book::OrderBook;
    use crate::{Commands, Order, OrderType, Side};

    #[test]
    fn test_match_multiple_orders() {
        let order_price = 122;
        let mut order_book = OrderBook::new();
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
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
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: 123,
            qty: 1,
        };
        let order1 = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Buy,
            price: 124,
            qty: 1,
        };
        let order2 = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: 122,
            qty: 1,
        };
        let order3 = Commands::NewOrder {
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
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        let order = Commands::NewOrder {
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
        let order = Commands::NewOrder {
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
        let order = Commands::NewOrder {
            order_type: OrderType::GoodTilCancel,
            side: Side::Sell,
            price: order_price,
            qty: 1,
        };
        order_book.process_command(order);
        assert_eq!(order_book.asks.len(), 1);
    }
}
