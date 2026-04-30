use std::io::Write;

#[repr(C)]
struct Ciovec {
    buf: *const u8,
    buf_len: usize,
}

#[repr(C)]
struct Iovec {
    buf: *mut u8,
    buf_len: usize,
}

#[link(wasm_import_module = "wasi_snapshot_preview1")]
unsafe extern "C" {
    #[link_name = "sock_send"]
    fn wasi_sock_send(
        fd: u32,
        si_data: *const Ciovec,
        si_data_len: usize,
        si_flags: u32,
        nwritten: *mut usize,
    ) -> u16;

    #[link_name = "sock_recv"]
    fn wasi_sock_recv(
        fd: u32,
        ri_data: *mut Iovec,
        ri_data_len: usize,
        ri_flags: u32,
        nread: *mut usize,
        ro_flags: *mut u32,
    ) -> u16;

    #[link_name = "sock_shutdown"]
    fn wasi_sock_shutdown(fd: u32, how: u32) -> u16;
}

const NETWORK_FD: u32 = 30;

fn main() {
    let payload = b"ping";
    let send_iov = Ciovec {
        buf: payload.as_ptr(),
        buf_len: payload.len(),
    };
    let mut written = 0usize;
    let errno = unsafe { wasi_sock_send(NETWORK_FD, &send_iov, 1, 0, &mut written) };
    assert_eq!(errno, 0, "sock_send errno");
    assert_eq!(written, payload.len(), "sock_send length");

    let mut recv = [0u8; 16];
    let mut recv_iov = Iovec {
        buf: recv.as_mut_ptr(),
        buf_len: recv.len(),
    };
    let mut nread = 0usize;
    let mut ro_flags = 0u32;
    let errno = unsafe { wasi_sock_recv(NETWORK_FD, &mut recv_iov, 1, 0, &mut nread, &mut ro_flags) };
    assert_eq!(errno, 0, "sock_recv errno");
    assert_eq!(&recv[..nread], b"pong", "sock_recv payload");
    assert_eq!(ro_flags, 0, "sock_recv flags");

    let errno = unsafe { wasi_sock_shutdown(NETWORK_FD, 0) };
    assert_eq!(errno, 0, "sock_shutdown errno");

    let mut stdout = std::io::stdout();
    stdout
        .write_all(b"hibana sock fd ping pong\n")
        .expect("write sock marker");
}
