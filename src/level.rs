// Copyright 2024 Mason Hall. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::Order;
use std::collections::VecDeque;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct Level {
    pub price: i32,
    pub orders: VecDeque<Order>,
}

impl Level {
    pub fn new(price: i32) -> Self {
        Level {
            price,
            orders: VecDeque::new(),
        }
    }
}
