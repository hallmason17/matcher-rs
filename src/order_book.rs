use std::{borrow::BorrowMut, cmp::Ordering, time::SystemTime};

use crate::{limit::Limit, Order, Side};

#[derive(Debug, PartialEq, Eq)]
pub struct OrderBook {
    pub bids: Vec<Limit>,
    pub asks: Vec<Limit>,
}

impl OrderBook {
    pub fn new() -> OrderBook {
        OrderBook {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    pub fn place_order(&mut self, mut order: Order) {
        if order.order_init_qty == 0 {
            return;
        }
        let order_to_match: Option<Order> = {
            let queue_to_match = match order.order_side {
                Side::Buy => &mut self.asks,
                Side::Sell => &mut self.bids,
            };
            if !queue_to_match.is_empty() {
                queue_to_match.sort();
                queue_to_match.first().unwrap().orders.front().cloned()
            } else {
                None
            }
        };

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
                        updated_at: SystemTime::now(),
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


    fn try_match_order(&mut self, order: &mut Order) -> bool {
        let order_to_match: Option<Order> = {
            let queue_to_match = match order.order_side {
                Side::Buy => &mut self.asks,
                Side::Sell => &mut self.bids,
            };
            if !queue_to_match.is_empty() {
                queue_to_match.sort();
                queue_to_match.first().unwrap().orders.front().cloned()
            } else {
                None
            }
        };

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
                            lim.orders.pop_front();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
                            }
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
                            lim.orders.pop_front();
                            if lim.orders.is_empty() {
                                lim_vec.remove(lim_pos);
                            }
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

    use std::collections::VecDeque;

    use crate::order_book::OrderBook;
    use crate::{Order, OrderPub, OrderType, Side};

    #[test]
    fn test_match_multiple_orders() {
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
        let order1 = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 5);
        order_book.place_order(order1);

        dbg!(&order_book);
        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn match_orders_diff_prices() {
        let mut order_book = OrderBook::new();
        let buyOrder = OrderPub::new(OrderType::GoodTilCancel, Side::Buy, 123, 1);
        let sellOrder = OrderPub::new(OrderType::GoodTilCancel, Side::Sell, 122, 1);
        order_book.place_order(sellOrder.convert_to_order());
        order_book.place_order(buyOrder.convert_to_order());
        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn test_match_orders() {
        let order_price = 122;
        let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        let order1 = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
        order_book.place_order(order1);

        dbg!(&order_book);
        assert_eq!(order_book.bids.len(), 0);
        assert_eq!(order_book.asks.len(), 0);
    }

    #[test]
    fn add_bid_order() {
        let order_price = 122;
        let order = Order::new(OrderType::GoodTilCancel, Side::Sell, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        assert_eq!(order_book.asks.len(), 1);
    }

    #[test]
    fn add_ask_order() {
        let order_price = 123;
        let order = Order::new(OrderType::GoodTilCancel, Side::Buy, order_price, 1);
        let mut order_book = OrderBook::new();
        order_book.place_order(order);
        assert_eq!(order_book.bids.len(), 1);
    }

}
