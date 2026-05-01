use hibana_wasi_guest::net::Datagram;
use std::io::Write;

const MAX_PAYLOAD_PROOF: [u8; Datagram::MAX_PAYLOAD] = [b'x'; Datagram::MAX_PAYLOAD];

fn main() -> hibana_wasi_guest::Result<()> {
    let net = Datagram::ping_pong()?;

    net.send(&MAX_PAYLOAD_PROOF)?;

    let mut recv = [0u8; Datagram::MAX_PAYLOAD];
    let n = net.recv(&mut recv)?;
    assert_eq!(&recv[..n], b"pong", "datagram payload");

    net.shutdown()?;

    let mut stdout = std::io::stdout();
    stdout
        .write_all(b"hibana network datagram ping pong\n")
        .expect("write datagram marker");
    Ok(())
}
