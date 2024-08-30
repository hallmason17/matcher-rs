use uuid7::Uuid;

mod order_book;

fn main() {
    print!("Hello World!")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Order {
    order_id: Uuid,
    order_type: OrderType,
    order_side: Side,
    order_price: i32,
    order_init_qty: u32,
    order_rem_qty: u32,
}

#[allow(dead_code)]
impl Order {
    fn new(order_type: OrderType, order_side: Side, order_price: i32, order_qty: u32) -> Order {
        Order {
            order_id: uuid7::uuid7(),
            order_type,
            order_side,
            order_price,
            order_init_qty: order_qty,
            order_rem_qty: order_qty,
        }
    }

    pub fn fill(&mut self, qty: u32) {
        if qty > self.order_rem_qty {
            return;
        }
        self.order_rem_qty -= qty;
    }

    pub fn is_filled(&self) -> bool {
        return self.order_rem_qty == 0;
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct OrderBookLevelsInfo {
    bids: Vec<LevelInfo>,
    asks: Vec<LevelInfo>,
}

#[allow(dead_code)]
impl OrderBookLevelsInfo {
    pub fn new() -> Self {
        OrderBookLevelsInfo {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
struct LevelInfo {
    price: i32,
    quantity: u32,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OrderType {
    FillAndKill,
    GoodTilCancel,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Buy,
    Sell,
}
