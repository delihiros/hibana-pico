use std::io::{self, Read};

fn main() {
    let mut command = [0u8; 8];
    let _ = io::stdin().read(&mut command);
    println!("hibana swarm actuator wasip1 app ack={}", command[0]);
}
