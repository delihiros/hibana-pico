#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    #[link_name = "sock_accept"]
    fn wasi_sock_accept(fd: u32, flags: u32, accepted_fd: *mut u32) -> u16;
}

fn main() {
    let mut accepted = 0u32;
    let errno = unsafe { wasi_sock_accept(31, 0, &mut accepted) };
    assert_eq!(errno, 0, "sock_accept must fail closed");
}
