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
}
