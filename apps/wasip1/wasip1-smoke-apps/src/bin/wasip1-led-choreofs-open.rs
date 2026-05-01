use std::hint::black_box;

const PREOPEN_FD: u32 = 9;
const FD_WRITE_RIGHT: u64 = 1 << 6;
const ERRNO_SUCCESS: u16 = 0;
const EVENTTYPE_CLOCK: u8 = 0;
const SUBSCRIPTION_EVENTTYPE_OFFSET: usize = 8;
const SUBSCRIPTION_CLOCK_TIMEOUT_OFFSET: usize = 24;

#[repr(C)]
struct Ciovec {
    buf: *const u8,
    buf_len: usize,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    fn path_open(
        fd: u32,
        dirflags: u32,
        path: *const u8,
        path_len: usize,
        oflags: u32,
        fs_rights_base: u64,
        fs_rights_inheriting: u64,
        fdflags: u32,
        opened_fd: *mut u32,
    ) -> u16;
    fn fd_write(fd: u32, iovs: *const Ciovec, iovs_len: usize, nwritten: *mut usize) -> u16;
    fn poll_oneoff(
        input: *const u8,
        output: *mut u8,
        nsubscriptions: usize,
        nevents: *mut usize,
    ) -> u16;
}

fn main() {
    let green = open_led(b"device/led/green");
    let orange = open_led(b"device/led/orange");
    let red = open_led(b"device/led/red");

    let plan = [
        (green, b'1', 180),
        (orange, b'1', 40),
        (orange, b'0', 40),
        (orange, b'1', 40),
        (orange, b'0', 40),
        (orange, b'1', 40),
        (red, b'1', 180),
    ];
    black_box(plan);

    for (fd, payload, delay_ms) in plan {
        write_led(fd, payload);
        sleep_ms(delay_ms);
    }
}

fn open_led(path: &[u8]) -> u32 {
    let mut fd = 0u32;
    let errno = unsafe {
        path_open(
            PREOPEN_FD,
            0,
            path.as_ptr(),
            path.len(),
            0,
            FD_WRITE_RIGHT,
            0,
            0,
            &mut fd,
        )
    };
    assert_eq!(errno, ERRNO_SUCCESS);
    fd
}

fn write_led(fd: u32, payload: u8) {
    let byte = [payload];
    let iov = [Ciovec {
        buf: byte.as_ptr(),
        buf_len: byte.len(),
    }];
    let mut written = 0usize;
    let errno = unsafe { fd_write(fd, iov.as_ptr(), iov.len(), &mut written) };
    assert_eq!(errno, ERRNO_SUCCESS);
    assert_eq!(written, byte.len());
}

fn sleep_ms(ms: u32) {
    let mut subscription = [0u8; 48];
    let mut event = [0u8; 32];
    let mut ready = 0usize;
    subscription[SUBSCRIPTION_EVENTTYPE_OFFSET] = EVENTTYPE_CLOCK;
    subscription[SUBSCRIPTION_CLOCK_TIMEOUT_OFFSET..SUBSCRIPTION_CLOCK_TIMEOUT_OFFSET + 8]
        .copy_from_slice(&(ms as u64 * 1_000_000).to_le_bytes());

    let errno = unsafe { poll_oneoff(subscription.as_ptr(), event.as_mut_ptr(), 1, &mut ready) };
    assert_eq!(errno, ERRNO_SUCCESS);
    assert_eq!(ready, 1);
    black_box(event);
}
