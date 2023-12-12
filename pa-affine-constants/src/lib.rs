use pa_types::Cost;
use std::cell::RefCell;

thread_local!(pub static INDEL_COST: RefCell<Cost> = RefCell::new(3));
thread_local!(pub static SUB_COST: RefCell<Cost> = RefCell::new(2));
