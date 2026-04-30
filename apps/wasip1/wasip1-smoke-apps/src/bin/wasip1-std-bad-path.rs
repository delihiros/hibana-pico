use std::fs;

fn main() {
    let _ = fs::read_to_string("forbidden.txt").expect("forbidden path must fail closed");
}
