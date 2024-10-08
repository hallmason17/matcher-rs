// Copyright 2024 Mason Hall. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::Order;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Limit {
    pub price: i32,
    pub orders: VecDeque<Order>,
}

impl Limit {
    pub fn new(price: i32) -> Self {
        Limit {
            price,
            orders: VecDeque::new(),
        }
    }

    pub fn find_by_id(&self, id: usize) -> Option<usize> {
        if let Some(pos) = self.orders.iter().position(|x| x.id == id) {
            Some(pos)
        } else {
            None
        }
    }

    pub fn remove_order_by_id(&mut self, id: usize) -> bool {
        if let Some(order_pos) = self.find_by_id(id) {
            self.orders.remove(order_pos).is_some()
        } else {
            false
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::Order;

    use super::Limit;

    #[test]
    fn test_remove_by_id() {
        let mut limit = Limit::new(10);
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order1 = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order1.clone());

        let removed = limit.remove_order_by_id(order1.id);
        assert_eq!(removed, true)
    }

    #[test]
    fn test_find_by_id() {
        let mut limit = Limit::new(10);
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());
        let order = Order::new(crate::OrderType::GoodTilCancel, crate::Side::Buy, 10, 1);
        limit.orders.push_back(order.clone());

        let pos = limit.find_by_id(order.id);
        assert_eq!(pos, Some(3usize))
    }
}
