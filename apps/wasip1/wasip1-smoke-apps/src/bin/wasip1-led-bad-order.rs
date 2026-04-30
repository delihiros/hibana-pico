#![no_main]

#[repr(C)]
struct Ciovec {
    buf: *const u8,
    len: usize,
}

unsafe impl Sync for Ciovec {}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    fn fd_write(fd: u32, iovs: *const Ciovec, iovs_len: usize, nwritten: *mut usize) -> u16;
    fn poll_oneoff(
        in_: *const u8,
        out: *mut u8,
        nsubscriptions: usize,
        nevents: *mut usize,
    ) -> u16;
}

static ON: [u8; 1] = *b"1";
static ON_IOV: Ciovec = Ciovec {
    buf: ON.as_ptr(),
    len: ON.len(),
};
static WAIT: [u8; 8] = 50u64.to_le_bytes();

static mut WRITTEN: usize = 0;
static mut EVENTS: [u8; 8] = [0; 8];
static mut NEVENTS: usize = 0;

#[unsafe(export_name = "__main_void")]
pub extern "C" fn main_void() {
    unsafe {
        poll_oneoff(
            WAIT.as_ptr(),
            (&raw mut EVENTS).cast(),
            1,
            &raw mut NEVENTS,
        );
        fd_write(3, &ON_IOV, 1, &raw mut WRITTEN);
    }
}
