use hibana_wasi_guest::baker::{Led, sleep_ms};

fn main() {
    let green = Led::open("/device/led/green").unwrap();
    let orange = Led::open("/device/led/orange").unwrap();
    let red = Led::open("/device/led/red").unwrap();

    green.set(true).unwrap();
    sleep_ms(180).unwrap();

    orange.set(true).unwrap();
    sleep_ms(40).unwrap();
    orange.set(false).unwrap();
    sleep_ms(40).unwrap();
    orange.set(true).unwrap();
    sleep_ms(40).unwrap();
    orange.set(false).unwrap();
    sleep_ms(40).unwrap();
    orange.set(true).unwrap();
    sleep_ms(40).unwrap();

    red.set(true).unwrap();
    sleep_ms(180).unwrap();
}
