// https://doc.rust-lang.org/book/ch03-00-common-programming-concepts.html
#![allow(unused_variables)]
#![allow(dead_code)]
#![allow(unused)]

use std::collections::HashMap;
use std::error::Error;

fn main() {
    match hello_rust::read_stock_data("AAL_data.csv") {
        Ok(_) => (),
        Err(e) => println!("failed on: {}", e)
    }
}