use hibana_wasi_guest::net::Listener;

const FAIL_MARKER: &str = "sock_accept must fail closed";

fn main() -> hibana_wasi_guest::Result<()> {
    let listener = Listener::control()?;
    let _stream = listener.accept_stream().expect(FAIL_MARKER);
    Ok(())
}
