const PREOPEN_FD: u32 = 9;
const FD_WRITE_RIGHT: u64 = 1 << 6;
const ERRNO_SUCCESS: u16 = 0;

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
}

fn main() {
    let path = b"not/allowed";
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
    assert_eq!(
        errno,
        ERRNO_SUCCESS,
        "bad guest expects forbidden path success; kernel must fail closed before this point"
    );
}
