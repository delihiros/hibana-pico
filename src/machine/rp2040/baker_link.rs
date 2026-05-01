use crate::{
    choreography::protocol::{GpioSet, LABEL_GPIO_SET},
    kernel::{
        choreofs::{
            CHOREOFS_ROUTE_OBJECT, ChoreoFsError, ChoreoFsStore, pico_rights_from_wasip1_base,
        },
        fd_object::GpioFdWriteRoute,
        guest_ledger::{GuestFd, GuestLedger},
        wasi::{ChoreoResourceKind, PicoFdRights, PicoFdRoute, PicoFdView, PicoFdViewEntry},
    },
};

pub const BAKER_LINK_LED_FD: u8 = 3;
pub const BAKER_LINK_CHOREOFS_PREOPEN_FD: u8 = 9;
pub const BAKER_LINK_LED_PIN: u8 = 22;
pub const BAKER_LINK_LED_FDS: [u8; 3] = [3, 4, 5];
pub const BAKER_LINK_LED_PINS: [u8; 3] = [22, 21, 20];
pub const BAKER_LINK_LED_RESOURCE_PATHS: [&[u8]; 3] =
    [b"device/led/green", b"device/led/orange", b"device/led/red"];
pub const BAKER_LINK_WRONG_OBJECT_PATH: &[u8] = b"device/not-gpio";
pub const BAKER_LINK_LED_ACTIVE_HIGH: bool = true;
pub const BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS: usize = 7;
pub const BAKER_LINK_TRAFFIC_GREEN_DELAY_TICKS: u32 = 250;
pub const BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS: u32 = 50;
pub const BAKER_LINK_TRAFFIC_RED_DELAY_TICKS: u32 = 250;
pub const BAKER_LINK_LED_LANE: u8 = 3;
pub const BAKER_LINK_LED_ROUTE_LABEL: u8 = LABEL_GPIO_SET;
pub const BAKER_LINK_LED_TARGET_NODE: u8 = 0;
pub const BAKER_LINK_LED_TARGET_ROLE: u16 = 0;
pub const BAKER_LINK_LED_SESSION_GENERATION: u16 = 0;
pub const BAKER_LINK_LED_POLICY_SLOT: u8 = 0;

pub type BakerLinkLedResourceStore = ChoreoFsStore<4, 24, 0>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BakerLinkTrafficStep {
    fd: u8,
    high: bool,
    delay_ticks: u32,
}

impl BakerLinkTrafficStep {
    pub const fn new(fd: u8, high: bool, delay_ticks: u32) -> Self {
        Self {
            fd,
            high,
            delay_ticks,
        }
    }

    pub const fn fd(self) -> u8 {
        self.fd
    }

    pub const fn high(self) -> bool {
        self.high
    }

    pub const fn delay_ticks(self) -> u32 {
        self.delay_ticks
    }
}

pub const BAKER_LINK_TRAFFIC_LIGHT_PATTERN: [BakerLinkTrafficStep;
    BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS] = [
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[0],
        true,
        BAKER_LINK_TRAFFIC_GREEN_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[1],
        true,
        BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[1],
        false,
        BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[1],
        true,
        BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[1],
        false,
        BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[1],
        true,
        BAKER_LINK_TRAFFIC_ORANGE_DELAY_TICKS,
    ),
    BakerLinkTrafficStep::new(
        BAKER_LINK_LED_FDS[2],
        true,
        BAKER_LINK_TRAFFIC_RED_DELAY_TICKS,
    ),
];

pub const fn baker_link_traffic_light_step(step: usize) -> BakerLinkTrafficStep {
    BAKER_LINK_TRAFFIC_LIGHT_PATTERN[step % BAKER_LINK_TRAFFIC_LIGHT_PATTERN_STEPS]
}

pub const fn baker_link_led_route() -> PicoFdRoute {
    PicoFdRoute::new(
        BAKER_LINK_LED_TARGET_NODE,
        BAKER_LINK_LED_TARGET_ROLE,
        BAKER_LINK_LED_LANE,
        BAKER_LINK_LED_ROUTE_LABEL,
        BAKER_LINK_LED_SESSION_GENERATION,
        BAKER_LINK_LED_POLICY_SLOT,
    )
}

pub const fn baker_link_led_fd_write_route() -> GpioFdWriteRoute {
    GpioFdWriteRoute::new(
        &BAKER_LINK_LED_FDS,
        &BAKER_LINK_LED_PINS,
        BAKER_LINK_LED_ACTIVE_HIGH,
        baker_link_led_route(),
    )
}

pub fn baker_link_led_resource_store() -> Result<BakerLinkLedResourceStore, ChoreoFsError> {
    let mut store = ChoreoFsStore::new();
    for path in BAKER_LINK_LED_RESOURCE_PATHS {
        store.install_gpio_device(path)?;
    }
    store.install_config_cell(BAKER_LINK_WRONG_OBJECT_PATH, &[])?;
    Ok(store)
}

pub fn baker_link_choreofs_ledger<const FDS: usize, const LEASES: usize, const PENDING: usize>(
    store: &BakerLinkLedResourceStore,
    memory_len: u32,
    memory_epoch: u32,
) -> Result<GuestLedger<FDS, LEASES, PENDING>, ChoreoFsError> {
    let mut ledger = GuestLedger::pico_min(memory_len, memory_epoch);
    store.grant_preopen_root(&mut ledger, BAKER_LINK_CHOREOFS_PREOPEN_FD)?;
    Ok(ledger)
}

pub fn open_baker_link_choreofs_path<
    const FDS: usize,
    const LEASES: usize,
    const PENDING: usize,
>(
    store: &BakerLinkLedResourceStore,
    ledger: &mut GuestLedger<FDS, LEASES, PENDING>,
    path: &[u8],
    rights_base: u64,
) -> Result<GuestFd, ChoreoFsError> {
    let selector = baker_link_choreofs_selector(path);
    let new_fd = baker_link_choreofs_fd_for_selector(selector)?;
    ledger.resolve_fd(
        BAKER_LINK_CHOREOFS_PREOPEN_FD,
        PicoFdRights::Read,
        ChoreoResourceKind::PreopenRoot,
    )?;
    let rights = pico_rights_from_wasip1_base(rights_base);
    let opened = store.open(selector, rights)?;
    let (lane, route_label, target_role, policy_slot) = match opened.resource() {
        ChoreoResourceKind::Gpio => (
            BAKER_LINK_LED_LANE,
            BAKER_LINK_LED_ROUTE_LABEL,
            BAKER_LINK_LED_TARGET_ROLE,
            BAKER_LINK_LED_POLICY_SLOT,
        ),
        _ => (8, CHOREOFS_ROUTE_OBJECT, opened.object_id(), 0),
    };
    Ok(ledger.apply_fd_cap_mint(
        new_fd,
        rights,
        opened.resource(),
        lane,
        route_label,
        opened.object_id(),
        BAKER_LINK_LED_TARGET_NODE,
        target_role,
        BAKER_LINK_LED_SESSION_GENERATION,
        opened.generation(),
        policy_slot,
    )?)
}

fn baker_link_choreofs_selector(path: &[u8]) -> &[u8] {
    match path.split_first() {
        Some((b'/', rest)) => rest,
        _ => path,
    }
}

fn baker_link_choreofs_fd_for_selector(path: &[u8]) -> Result<u8, ChoreoFsError> {
    for (index, candidate) in BAKER_LINK_LED_RESOURCE_PATHS.iter().enumerate() {
        if *candidate == path {
            return Ok(BAKER_LINK_LED_FDS[index]);
        }
    }
    if path == BAKER_LINK_WRONG_OBJECT_PATH {
        return Ok(BAKER_LINK_LED_FD);
    }
    Err(ChoreoFsError::NotFound)
}

pub fn grant_baker_link_led_fd<const N: usize>(
    table: &mut PicoFdView<N>,
) -> Result<PicoFdViewEntry, ChoreoFsError> {
    let store = baker_link_led_resource_store()?;
    mint_baker_link_led_fd_for(table, &store, BAKER_LINK_LED_FD)
}

pub fn grant_baker_link_led_sequence_fds<const N: usize>(
    table: &mut PicoFdView<N>,
) -> Result<(), ChoreoFsError> {
    let store = baker_link_led_resource_store()?;
    for fd in BAKER_LINK_LED_FDS {
        mint_baker_link_led_fd_for(table, &store, fd)?;
    }
    Ok(())
}

fn mint_baker_link_led_fd_for<const N: usize>(
    table: &mut PicoFdView<N>,
    store: &BakerLinkLedResourceStore,
    fd: u8,
) -> Result<PicoFdViewEntry, ChoreoFsError> {
    let index = baker_link_led_index_for_fd(fd).ok_or(ChoreoFsError::NotFound)?;
    let opened = store.open(BAKER_LINK_LED_RESOURCE_PATHS[index], PicoFdRights::Write)?;
    Ok(table.apply_cap_mint(
        fd,
        PicoFdRights::Write,
        opened.resource(),
        BAKER_LINK_LED_LANE,
        BAKER_LINK_LED_ROUTE_LABEL,
        opened.object_id(),
        BAKER_LINK_LED_TARGET_NODE,
        BAKER_LINK_LED_TARGET_ROLE,
        BAKER_LINK_LED_SESSION_GENERATION,
        opened.generation(),
        BAKER_LINK_LED_POLICY_SLOT,
    )?)
}

pub fn grant_baker_link_led_sequence_ledger<
    const FDS: usize,
    const LEASES: usize,
    const PENDING: usize,
>(
    ledger: &mut GuestLedger<FDS, LEASES, PENDING>,
) -> Result<(), ChoreoFsError> {
    let store = baker_link_led_resource_store()?;
    for (index, fd) in BAKER_LINK_LED_FDS.into_iter().enumerate() {
        let opened = store.open(BAKER_LINK_LED_RESOURCE_PATHS[index], PicoFdRights::Write)?;
        ledger.apply_fd_cap_mint(
            fd,
            PicoFdRights::Write,
            opened.resource(),
            BAKER_LINK_LED_LANE,
            BAKER_LINK_LED_ROUTE_LABEL,
            opened.object_id(),
            BAKER_LINK_LED_TARGET_NODE,
            BAKER_LINK_LED_TARGET_ROLE,
            BAKER_LINK_LED_SESSION_GENERATION,
            opened.generation(),
            BAKER_LINK_LED_POLICY_SLOT,
        )?;
    }
    Ok(())
}

pub fn baker_link_pico_min_ledger<const LEASES: usize, const PENDING: usize>(
    memory_len: u32,
    memory_epoch: u32,
) -> Result<GuestLedger<3, LEASES, PENDING>, ChoreoFsError> {
    let mut ledger = GuestLedger::pico_min(memory_len, memory_epoch);
    grant_baker_link_led_sequence_ledger(&mut ledger)?;
    Ok(ledger)
}

fn baker_link_led_index_for_fd(fd: u8) -> Option<usize> {
    BAKER_LINK_LED_FDS
        .iter()
        .enumerate()
        .find_map(|(index, candidate)| (*candidate == fd).then_some(index))
}

pub fn baker_link_led_pin_for_fd(fd: u8) -> Option<u8> {
    baker_link_led_fd_write_route().pin_for_fd(fd)
}

pub fn apply_baker_link_led_bank_set(mut write_pin: impl FnMut(u8, bool), set: GpioSet) {
    if set.high() == BAKER_LINK_LED_ACTIVE_HIGH && BAKER_LINK_LED_PINS.contains(&set.pin()) {
        for pin in BAKER_LINK_LED_PINS {
            write_pin(pin, !BAKER_LINK_LED_ACTIVE_HIGH);
        }
    }
    write_pin(set.pin(), set.high());
}

#[cfg(test)]
mod tests {
    use super::{
        BAKER_LINK_LED_ACTIVE_HIGH, BAKER_LINK_LED_FD, BAKER_LINK_LED_PIN, BAKER_LINK_LED_PINS,
        BAKER_LINK_LED_RESOURCE_PATHS, BAKER_LINK_LED_SESSION_GENERATION,
        apply_baker_link_led_bank_set, baker_link_led_fd_write_route,
        baker_link_led_resource_store, baker_link_pico_min_ledger, grant_baker_link_led_fd,
        grant_baker_link_led_sequence_fds,
    };
    use crate::{
        choreography::protocol::FdWrite,
        kernel::{
            choreofs::ChoreoFsObjectKind,
            fd_object::{GpioFdWriteError, check_gpio_object_fd_write},
            wasi::{ChoreoResourceKind, PicoFdRights, PicoFdView, PicoFdViewSource},
        },
    };

    #[test]
    fn digit_one_selects_baker_link_led_active_level() {
        let mut fds: PicoFdView<1> = PicoFdView::new();
        grant_baker_link_led_fd(&mut fds).expect("grant led fd");
        let write = FdWrite::new(BAKER_LINK_LED_FD, b"1").expect("fd_write payload");

        let set = check_gpio_object_fd_write(&fds, &write, baker_link_led_fd_write_route())
            .expect("resolve led write");
        assert_eq!(set.pin(), BAKER_LINK_LED_PIN);
        assert_eq!(set.high(), BAKER_LINK_LED_ACTIVE_HIGH);
    }

    #[test]
    fn digit_zero_sets_baker_link_led_inactive_level() {
        let mut fds: PicoFdView<1> = PicoFdView::new();
        grant_baker_link_led_fd(&mut fds).expect("grant led fd");
        let write = FdWrite::new(BAKER_LINK_LED_FD, b"0").expect("fd_write payload");

        let set = check_gpio_object_fd_write(&fds, &write, baker_link_led_fd_write_route())
            .expect("resolve led write");
        assert_eq!(set.pin(), BAKER_LINK_LED_PIN);
        assert_eq!(set.high(), !BAKER_LINK_LED_ACTIVE_HIGH);
    }

    #[test]
    fn sequence_fds_select_each_baker_link_led_pin() {
        let mut fds: PicoFdView<3> = PicoFdView::new();
        grant_baker_link_led_sequence_fds(&mut fds).expect("grant led sequence fds");

        for (fd, pin) in [(3, 22), (4, 21), (5, 20)] {
            let view = fds
                .resolve_current(fd, PicoFdRights::Write, ChoreoResourceKind::Gpio)
                .expect("resolve minted led fd");
            assert_eq!(view.source(), PicoFdViewSource::Mint);

            let write = FdWrite::new(fd, b"1").expect("fd_write payload");
            let set = check_gpio_object_fd_write(&fds, &write, baker_link_led_fd_write_route())
                .expect("resolve led write");
            assert_eq!(set.pin(), pin);
            assert_eq!(set.high(), BAKER_LINK_LED_ACTIVE_HIGH);
        }
    }

    #[test]
    fn baker_led_fds_are_minted_from_choreofs_device_objects() {
        let store = baker_link_led_resource_store().expect("create Baker LED resource store");
        for path in BAKER_LINK_LED_RESOURCE_PATHS {
            let stat = store.stat_path(path).expect("stat LED ChoreoFS object");
            assert_eq!(stat.kind(), ChoreoFsObjectKind::GpioDevice);
            let opened = store
                .open(path, PicoFdRights::Write)
                .expect("open LED ChoreoFS object");
            assert_eq!(opened.resource(), ChoreoResourceKind::Gpio);
        }

        let ledger = baker_link_pico_min_ledger::<1, 1>(4096, 1).expect("create Baker ledger");
        for fd in [3, 4, 5] {
            let view = ledger
                .resolve_fd(fd, PicoFdRights::Write, ChoreoResourceKind::Gpio)
                .expect("resolve Baker LED fd");
            assert_eq!(view.source(), PicoFdViewSource::Mint);
            assert_eq!(view.wait_or_subscription_id(), u16::from(fd - 3));
            assert_eq!(
                view.route().session_generation(),
                BAKER_LINK_LED_SESSION_GENERATION
            );
            assert_ne!(view.choreo_object_generation(), 0);
        }
    }

    #[test]
    fn bank_set_keeps_led_sequence_one_hot() {
        let mut levels = [!BAKER_LINK_LED_ACTIVE_HIGH; 32];
        let mut fds: PicoFdView<3> = PicoFdView::new();
        grant_baker_link_led_sequence_fds(&mut fds).expect("grant led sequence fds");

        for fd in [3, 4, 5] {
            let set = check_gpio_object_fd_write(
                &fds,
                &FdWrite::new(fd, b"1").expect("fd_write payload"),
                baker_link_led_fd_write_route(),
            )
            .expect("resolve led write");
            apply_baker_link_led_bank_set(|pin, high| levels[pin as usize] = high, set);
            assert_eq!(
                BAKER_LINK_LED_PINS
                    .iter()
                    .filter(|pin| levels[**pin as usize] == BAKER_LINK_LED_ACTIVE_HIGH)
                    .count(),
                1,
                "only one Baker LED should remain active after selecting fd {fd}"
            );
            assert_eq!(levels[set.pin() as usize], BAKER_LINK_LED_ACTIVE_HIGH);
        }
    }

    #[test]
    fn non_digit_payload_fails_closed() {
        let mut fds: PicoFdView<1> = PicoFdView::new();
        grant_baker_link_led_fd(&mut fds).expect("grant led fd");
        let write = FdWrite::new(BAKER_LINK_LED_FD, b"on").expect("fd_write payload");

        assert_eq!(
            check_gpio_object_fd_write(&fds, &write, baker_link_led_fd_write_route()),
            Err(GpioFdWriteError::BadPayload)
        );
    }

    #[test]
    fn stdout_fd_does_not_route_to_led() {
        let mut fds: PicoFdView<1> = PicoFdView::new();
        fds.apply_local_cap_grant(1, PicoFdRights::Write, ChoreoResourceKind::Stdout, 1, 0, 0)
            .expect("grant stdout");
        let write = FdWrite::new(1, b"1").expect("fd_write payload");

        assert_eq!(
            check_gpio_object_fd_write(&fds, &write, baker_link_led_fd_write_route()),
            Err(GpioFdWriteError::BadFd)
        );
    }
}
