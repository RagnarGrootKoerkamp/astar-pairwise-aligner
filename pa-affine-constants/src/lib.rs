use pa_types::Cost;
use std::cell::RefCell;

thread_local!(pub static INDEL_COST: RefCell<Cost> = RefCell::new(3));
thread_local!(pub static SUB_COST: RefCell<Cost> = RefCell::new(2));
thread_local!(pub static R: RefCell<Cost> = INDEL_COST.with(|indel_cost| {
    SUB_COST.with(|sub_cost| {
        RefCell::new(std::cmp::min(*indel_cost.borrow(), *sub_cost.borrow()))
    })
}));
