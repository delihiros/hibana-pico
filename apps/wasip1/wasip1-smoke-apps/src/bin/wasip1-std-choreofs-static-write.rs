use std::{fs::OpenOptions, io::Write};

fn main() {
    let mut file = OpenOptions::new()
        .write(true)
        .open("readonly.txt")
        .expect("readonly static write must fail closed");
    file.write_all(b"bad")
        .expect("readonly static write must not succeed");
}
