use std::{
    fs::File,
    hint::black_box,
    io::Write,
    os::fd::{FromRawFd, IntoRawFd},
    thread,
    time::Duration,
};

const STEP_BYTES: usize = 6;
const STEP_COUNT: usize = 7;

const TRAFFIC_PLAN: &[u8] = b"\x03\x31\xfa\x00\x00\x00\
\x04\x31\x32\x00\x00\x00\
\x05\x31\x32\x00\x00\x00\
\x04\x31\x32\x00\x00\x00\
\x03\x31\x32\x00\x00\x00\
\x04\x31\x32\x00\x00\x00\
\x05\x31\xfa\x00\x00\x00";

fn main() {
    black_box(TRAFFIC_PLAN);

    for index in 0..STEP_COUNT {
        let offset = index * STEP_BYTES;
        let fd = TRAFFIC_PLAN[offset] as u32;
        let payload = TRAFFIC_PLAN[offset + 1];
        let delay = u32::from_le_bytes([
            TRAFFIC_PLAN[offset + 2],
            TRAFFIC_PLAN[offset + 3],
            TRAFFIC_PLAN[offset + 4],
            TRAFFIC_PLAN[offset + 5],
        ]) as u64;
        write_led(fd, payload);
        sleep_ms(delay);
    }
}

fn write_led(fd: u32, payload: u8) {
    let mut file = unsafe { File::from_raw_fd(fd as i32) };
    file.write_all(&[payload]).expect("write Baker LED fd");
    let _ = file.into_raw_fd();
}

fn sleep_ms(ms: u64) {
    thread::sleep(Duration::from_millis(ms));
}
