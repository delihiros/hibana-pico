#[cfg(any(
    all(
        feature = "baker-bad-order-demo",
        any(
            feature = "baker-chaser-demo",
            feature = "baker-ordinary-std-demo",
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-chaser-demo",
        any(
            feature = "baker-ordinary-std-demo",
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-ordinary-std-demo",
        any(
            feature = "baker-invalid-fd-demo",
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-invalid-fd-demo",
        any(
            feature = "baker-bad-payload-demo",
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-bad-payload-demo",
        any(
            feature = "baker-choreofs-demo",
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-demo",
        any(
            feature = "baker-choreofs-bad-path-demo",
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-bad-path-demo",
        any(
            feature = "baker-choreofs-bad-payload-demo",
            feature = "baker-choreofs-wrong-object-demo"
        )
    ),
    all(
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )
))]
compile_error!("select at most one Baker WASI guest pattern");

#[cfg(all(target_arch = "arm", target_os = "none"))]
use hibana_pico::kernel::features::Wasip1HandlerSet;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(not(any(
    feature = "baker-bad-order-demo",
    feature = "baker-chaser-demo",
    feature = "baker-ordinary-std-demo",
    feature = "baker-choreofs-demo",
    feature = "baker-choreofs-bad-path-demo",
    feature = "baker-choreofs-bad-payload-demo",
    feature = "baker-choreofs-wrong-object-demo",
    feature = "baker-invalid-fd-demo",
    feature = "baker-bad-payload-demo"
)))]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-blink.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-bad-order-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-bad-order.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-chaser-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-chaser.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-invalid-fd-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-invalid-fd.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-bad-payload-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-bad-payload.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-ordinary-std-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-ordinary-std-chaser.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-open.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-bad-path-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-bad-path.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-bad-payload-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-bad-payload.wasm"
));
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[cfg(feature = "baker-choreofs-wrong-object-demo")]
pub static WASIP1_LED_GUEST: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/target/wasip1-apps/wasm32-wasip1/release/wasip1-led-choreofs-wrong-object.wasm"
));

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub const fn baker_wasip1_handler_set() -> Wasip1HandlerSet {
    if cfg!(any(
        feature = "baker-choreofs-demo",
        feature = "baker-choreofs-bad-path-demo",
        feature = "baker-choreofs-bad-payload-demo",
        feature = "baker-choreofs-wrong-object-demo"
    )) {
        Wasip1HandlerSet::PICO_STD_CHOREOFS
    } else if cfg!(feature = "baker-ordinary-std-demo") {
        Wasip1HandlerSet::PICO_STD_START
    } else {
        Wasip1HandlerSet::PICO_MIN
    }
}
