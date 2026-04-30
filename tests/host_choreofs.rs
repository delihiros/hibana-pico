use hibana_pico::{
    choreography::protocol::{FdRead, FdWrite, MemBorrow, MemRelease},
    kernel::{
        choreofs::{
            ChoreoFsError, ChoreoFsObjectKind, ChoreoFsStore, WASIP1_RIGHT_FD_READ,
            WASIP1_RIGHT_FD_READDIR, WASIP1_RIGHT_FD_WRITE,
        },
        guest_ledger::{
            GuestFdKind, GuestLedger, GuestLedgerError, GuestQuotaLimits, WasiErrnoMap, WasiProfile,
        },
        wasi::PicoFdError,
        wasi::{ChoreoResourceKind, PicoFdRights, PicoFdViewSource},
    },
};

type TestLedger = GuestLedger<8, 4, 4>;
type TestStore = ChoreoFsStore<8, 64, 64>;

fn ledger() -> TestLedger {
    GuestLedger::new(
        WasiProfile::HostFull,
        4096,
        1,
        GuestQuotaLimits::new(8, 4),
        WasiErrnoMap::new(),
    )
}

#[test]
fn choreofs_path_open_mints_object_fd_and_uses_lease_backed_read() {
    let mut store = TestStore::new();
    store
        .install_directory(b"app")
        .expect("install app directory");
    store
        .install_static_blob(b"app/config", b"mode=demo")
        .expect("install config object");

    let mut ledger = ledger();
    let preopen = store
        .grant_preopen_root(&mut ledger, 3)
        .expect("grant preopen root");
    assert_eq!(preopen.fd(), 3);
    assert_eq!(preopen.kind(), GuestFdKind::PreopenRoot);
    assert_eq!(preopen.source(), PicoFdViewSource::Grant);

    let object_fd = store
        .open_wasip1_path_with_ledger(&mut ledger, 3, 4, b"app/config", WASIP1_RIGHT_FD_READ)
        .expect("open config object through manifest");
    assert_eq!(object_fd.kind(), GuestFdKind::ChoreoObject);
    assert_eq!(object_fd.source(), PicoFdViewSource::Mint);
    let stat = store.stat_fd(object_fd).expect("stat opened object");
    assert_eq!(stat.kind(), ChoreoFsObjectKind::StaticBlob);
    assert_eq!(stat.size(), b"mode=demo".len());

    let grant = ledger
        .grant_write_lease(MemBorrow::new(128, 16, 1))
        .expect("grant write lease for fd_read destination");
    let read = FdRead::new_with_lease(object_fd.fd(), grant.lease_id(), 16).expect("fd_read");
    let token = ledger
        .begin_choreofs_read(&read, grant)
        .expect("begin ChoreoFS read pending token");

    let mut out = [0u8; 16];
    let len = store
        .read(object_fd, 0, &mut out)
        .expect("read static config object");
    assert_eq!(&out[..len], b"mode=demo");

    ledger
        .complete_choreofs_read(token, object_fd.fd(), grant.lease_id(), len as u16)
        .expect("complete ChoreoFS read pending token");
    ledger
        .release_lease(MemRelease::new(grant.lease_id()))
        .expect("release read destination lease");
}

#[test]
fn choreofs_config_and_append_log_are_bounded_minted_objects() {
    let mut store = TestStore::new();
    store.install_directory(b"app").expect("install app dir");
    store
        .install_config_cell(b"app/state", b"v1")
        .expect("install config cell");
    store
        .install_append_log(b"app/events")
        .expect("install append log");

    let mut ledger = ledger();
    store
        .grant_preopen_root(&mut ledger, 3)
        .expect("grant preopen root");
    let state_fd = store
        .open_wasip1_path_with_ledger(
            &mut ledger,
            3,
            4,
            b"app/state",
            WASIP1_RIGHT_FD_READ | WASIP1_RIGHT_FD_WRITE,
        )
        .expect("open config cell");
    let log_fd = store
        .open_wasip1_path_with_ledger(
            &mut ledger,
            3,
            5,
            b"app/events",
            WASIP1_RIGHT_FD_READ | WASIP1_RIGHT_FD_WRITE,
        )
        .expect("open append log");

    let grant = ledger
        .grant_read_lease(MemBorrow::new(64, 2, 1))
        .expect("grant source lease");
    let write = FdWrite::new_with_lease(state_fd.fd(), grant.lease_id(), b"v2").expect("fd_write");
    let token = ledger
        .begin_choreofs_write(&write, grant)
        .expect("begin config write");
    let written = store.write(state_fd, 0, b"v2").expect("write config cell");
    ledger
        .complete_choreofs_write(token, state_fd.fd(), grant.lease_id(), written as u16)
        .expect("complete config write");
    ledger
        .release_lease(MemRelease::new(grant.lease_id()))
        .expect("release source lease");

    let mut out = [0u8; 8];
    let len = store.read(state_fd, 0, &mut out).expect("read new config");
    assert_eq!(&out[..len], b"v2");

    assert_eq!(
        store.write(log_fd, 1, b"bad"),
        Err(ChoreoFsError::BadOffset),
        "append log must not allow random writes"
    );
    assert_eq!(store.write(log_fd, 0, b"a"), Ok(1));
    assert_eq!(store.write(log_fd, 1, b"b"), Ok(1));
    let len = store.read(log_fd, 0, &mut out).expect("read append log");
    assert_eq!(&out[..len], b"ab");
}

#[test]
fn choreofs_gpio_device_object_mints_gpio_fd_without_data_path() {
    let mut store = TestStore::new();
    store
        .install_gpio_device(b"device/led/green")
        .expect("install GPIO device object");

    let mut ledger = ledger();
    store
        .grant_preopen_root(&mut ledger, 3)
        .expect("grant preopen root");
    let fd = store
        .open_wasip1_path_with_ledger(
            &mut ledger,
            3,
            4,
            b"device/led/green",
            WASIP1_RIGHT_FD_WRITE,
        )
        .expect("open GPIO device object");

    assert_eq!(fd.kind(), GuestFdKind::Gpio);
    assert_eq!(fd.source(), PicoFdViewSource::Mint);
    assert_eq!(fd.choreo_object_generation(), 1);
    assert_eq!(
        ledger.resolve_fd(4, PicoFdRights::Write, ChoreoResourceKind::Gpio),
        Ok(fd)
    );

    let mut out = [0u8; 4];
    assert_eq!(store.read(fd, 0, &mut out), Err(ChoreoFsError::WrongFdKind));
    assert_eq!(store.write(fd, 0, b"1"), Err(ChoreoFsError::WrongFdKind));
}

#[test]
fn choreofs_directory_view_and_path_normalization_fail_closed() {
    let mut store = TestStore::new();
    store.install_directory(b"app").expect("install app dir");
    store
        .install_static_blob(b"app/config", b"cfg")
        .expect("install config");
    store
        .install_static_blob(b"app/state", b"state")
        .expect("install state");
    store
        .install_static_blob(b"other/root", b"ignored")
        .expect("install non-child object");

    let mut ledger = ledger();
    store
        .grant_preopen_root(&mut ledger, 3)
        .expect("grant preopen root");
    let dir_fd = store
        .open_wasip1_path_with_ledger(&mut ledger, 3, 4, b"app", WASIP1_RIGHT_FD_READDIR)
        .expect("open directory view");
    assert_eq!(dir_fd.kind(), GuestFdKind::DirectoryView);
    assert_eq!(
        store.stat_path(b"app/config").expect("stat path").size(),
        b"cfg".len()
    );

    let mut out = [0u8; 32];
    let read = store
        .read_directory(dir_fd, 0, &mut out)
        .expect("read manifest directory view");
    assert!(read.done());
    assert_eq!(&out[..read.written()], b"config\nstate\n");

    assert_eq!(
        store.open(b"../secret", PicoFdRights::Read),
        Err(ChoreoFsError::InvalidComponent)
    );
    assert_eq!(
        store.open(b"/app/config", PicoFdRights::Read),
        Err(ChoreoFsError::AbsolutePath)
    );
    assert_eq!(
        store.open_with_ledger(&mut ledger, 3, 6, b"missing", PicoFdRights::Read),
        Err(ChoreoFsError::NotFound)
    );
    assert_eq!(
        ledger.resolve_fd(4, PicoFdRights::Read, ChoreoResourceKind::ChoreoObject),
        Err(GuestLedgerError::Fd(PicoFdError::WrongResource)),
        "directory fd must not be usable as an object fd"
    );
}
