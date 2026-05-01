const PREOPEN_FD: u32 = 9;
const FD_WRITE_RIGHT: u64 = 1 << 6;
const ERRNO_SUCCESS: u16 = 0;

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
}

fn main() {
    let fd = open_path(b"/device/led/green");
    let _orange = open_path(b"/device/led/orange");
    let _red = open_path(b"/device/led/red");
    let bad = b"on";
    let iov = [Ciovec {
        buf: bad.as_ptr(),
        buf_len: bad.len(),
    }];
    let mut written = 0usize;
    let errno = unsafe { fd_write(fd, iov.as_ptr(), iov.len(), &mut written) };
    assert_eq!(errno, ERRNO_SUCCESS, "bad LED payload must fail closed");
}

fn open_path(path: &[u8]) -> u32 {
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
