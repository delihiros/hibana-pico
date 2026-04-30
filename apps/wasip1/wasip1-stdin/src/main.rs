use std::io::{self, Read};

fn main() {
    let mut buf = [0u8; 24];
    let _ = io::stdin().read(&mut buf);
}
