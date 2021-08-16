mod clocom;

use std::collections::HashMap;

fn main() {
    match clocom::read_stock_data("AAL_data.csv") {
        Ok(_) => (),
        Err(e) => println!("failed on: {}", e)
    }
    // print!("{}siema",termion::cursor::Goto(20,20));
}