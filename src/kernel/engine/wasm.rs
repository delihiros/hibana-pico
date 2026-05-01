use crate::{
    choreography::protocol::{
        BudgetExpired, BudgetRun, EngineReq, EngineRet, FdRead, FdRequest, FdWrite, GpioSet,
        TimerSleepUntil, WASIP1_STREAM_CHUNK_CAPACITY,
    },
    kernel::features::{WASIP1_PREVIEW1_MODULE, Wasip1HandlerSet, Wasip1ImportName, Wasip1Syscall},
};

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_TABLE: u8 = 4;
const SECTION_ELEMENT: u8 = 9;
const SECTION_EXPORT: u8 = 7;
const SECTION_CODE: u8 = 10;
const SECTION_DATA: u8 = 11;
const SECTION_CUSTOM: u8 = 0;

const EXTERNAL_KIND_FUNC: u8 = 0;
const FUNC_TYPE_FORM: u8 = 0x60;
const VALTYPE_I32: u8 = 0x7f;
const VALTYPE_I64: u8 = 0x7e;
const VALTYPE_F32: u8 = 0x7d;
const VALTYPE_F64: u8 = 0x7c;
const VALTYPE_FUNCREF: u8 = 0x70;

const OPCODE_UNREACHABLE: u8 = 0x00;
const OPCODE_NOP: u8 = 0x01;
const OPCODE_BLOCK: u8 = 0x02;
const OPCODE_LOOP: u8 = 0x03;
const OPCODE_IF: u8 = 0x04;
const OPCODE_ELSE: u8 = 0x05;
const OPCODE_BR: u8 = 0x0c;
const OPCODE_BR_IF: u8 = 0x0d;
const OPCODE_BR_TABLE: u8 = 0x0e;
const OPCODE_RETURN: u8 = 0x0f;
const OPCODE_CALL: u8 = 0x10;
const OPCODE_CALL_INDIRECT: u8 = 0x11;
const OPCODE_SELECT: u8 = 0x1b;
const OPCODE_DROP: u8 = 0x1a;
const OPCODE_LOCAL_GET: u8 = 0x20;
const OPCODE_LOCAL_SET: u8 = 0x21;
const OPCODE_LOCAL_TEE: u8 = 0x22;
const OPCODE_GLOBAL_GET: u8 = 0x23;
const OPCODE_GLOBAL_SET: u8 = 0x24;
const OPCODE_TABLE_GET: u8 = 0x25;
const OPCODE_TABLE_SET: u8 = 0x26;
const OPCODE_I32_LOAD: u8 = 0x28;
const OPCODE_I64_LOAD: u8 = 0x29;
const OPCODE_F32_LOAD: u8 = 0x2a;
const OPCODE_F64_LOAD: u8 = 0x2b;
const OPCODE_I32_LOAD8_S: u8 = 0x2c;
const OPCODE_I32_LOAD8_U: u8 = 0x2d;
const OPCODE_I32_LOAD16_S: u8 = 0x2e;
const OPCODE_I32_LOAD16_U: u8 = 0x2f;
const OPCODE_I64_LOAD8_S: u8 = 0x30;
const OPCODE_I64_LOAD8_U: u8 = 0x31;
const OPCODE_I64_LOAD16_S: u8 = 0x32;
const OPCODE_I64_LOAD16_U: u8 = 0x33;
const OPCODE_I64_LOAD32_S: u8 = 0x34;
const OPCODE_I64_LOAD32_U: u8 = 0x35;
const OPCODE_I32_STORE: u8 = 0x36;
const OPCODE_I64_STORE: u8 = 0x37;
const OPCODE_F32_STORE: u8 = 0x38;
const OPCODE_F64_STORE: u8 = 0x39;
const OPCODE_I32_STORE8: u8 = 0x3a;
const OPCODE_I32_STORE16: u8 = 0x3b;
const OPCODE_I64_STORE8: u8 = 0x3c;
const OPCODE_I64_STORE16: u8 = 0x3d;
const OPCODE_I64_STORE32: u8 = 0x3e;
const OPCODE_MEMORY_SIZE: u8 = 0x3f;
const OPCODE_MEMORY_GROW: u8 = 0x40;
const OPCODE_I32_CONST: u8 = 0x41;
const OPCODE_I64_CONST: u8 = 0x42;
const OPCODE_F32_CONST: u8 = 0x43;
const OPCODE_F64_CONST: u8 = 0x44;
const OPCODE_I32_EQZ: u8 = 0x45;
const OPCODE_I32_EQ: u8 = 0x46;
const OPCODE_I32_NE: u8 = 0x47;
const OPCODE_I32_LT_S: u8 = 0x48;
const OPCODE_I32_LT_U: u8 = 0x49;
const OPCODE_I32_GT_S: u8 = 0x4a;
const OPCODE_I32_GT_U: u8 = 0x4b;
const OPCODE_I32_LE_S: u8 = 0x4c;
const OPCODE_I32_LE_U: u8 = 0x4d;
const OPCODE_I32_GE_S: u8 = 0x4e;
const OPCODE_I32_GE_U: u8 = 0x4f;
const OPCODE_I64_EQZ: u8 = 0x50;
const OPCODE_I64_EQ: u8 = 0x51;
const OPCODE_I64_NE: u8 = 0x52;
const OPCODE_I64_LT_S: u8 = 0x53;
const OPCODE_I64_LT_U: u8 = 0x54;
const OPCODE_I64_GT_S: u8 = 0x55;
const OPCODE_I64_GT_U: u8 = 0x56;
const OPCODE_I64_LE_S: u8 = 0x57;
const OPCODE_I64_LE_U: u8 = 0x58;
const OPCODE_I64_GE_S: u8 = 0x59;
const OPCODE_I64_GE_U: u8 = 0x5a;
const OPCODE_F32_EQ: u8 = 0x5b;
const OPCODE_F32_NE: u8 = 0x5c;
const OPCODE_F32_LT: u8 = 0x5d;
const OPCODE_F32_GT: u8 = 0x5e;
const OPCODE_F32_LE: u8 = 0x5f;
const OPCODE_F32_GE: u8 = 0x60;
const OPCODE_F64_EQ: u8 = 0x61;
const OPCODE_F64_NE: u8 = 0x62;
const OPCODE_F64_LT: u8 = 0x63;
const OPCODE_F64_GT: u8 = 0x64;
const OPCODE_F64_LE: u8 = 0x65;
const OPCODE_F64_GE: u8 = 0x66;
const OPCODE_I32_CLZ: u8 = 0x67;
const OPCODE_I32_CTZ: u8 = 0x68;
const OPCODE_I32_POPCNT: u8 = 0x69;
const OPCODE_I32_ADD: u8 = 0x6a;
const OPCODE_I32_SUB: u8 = 0x6b;
const OPCODE_I32_MUL: u8 = 0x6c;
const OPCODE_I32_DIV_S: u8 = 0x6d;
const OPCODE_I32_DIV_U: u8 = 0x6e;
const OPCODE_I32_REM_S: u8 = 0x6f;
const OPCODE_I32_REM_U: u8 = 0x70;
const OPCODE_I32_AND: u8 = 0x71;
const OPCODE_I32_OR: u8 = 0x72;
const OPCODE_I32_XOR: u8 = 0x73;
const OPCODE_I32_SHL: u8 = 0x74;
const OPCODE_I32_SHR_S: u8 = 0x75;
const OPCODE_I32_SHR_U: u8 = 0x76;
const OPCODE_I32_ROTL: u8 = 0x77;
const OPCODE_I32_ROTR: u8 = 0x78;
const OPCODE_I64_CLZ: u8 = 0x79;
const OPCODE_I64_CTZ: u8 = 0x7a;
const OPCODE_I64_POPCNT: u8 = 0x7b;
const OPCODE_I64_ADD: u8 = 0x7c;
const OPCODE_I64_SUB: u8 = 0x7d;
const OPCODE_I64_MUL: u8 = 0x7e;
const OPCODE_I64_DIV_S: u8 = 0x7f;
const OPCODE_I64_DIV_U: u8 = 0x80;
const OPCODE_I64_REM_S: u8 = 0x81;
const OPCODE_I64_REM_U: u8 = 0x82;
const OPCODE_I64_AND: u8 = 0x83;
const OPCODE_I64_OR: u8 = 0x84;
const OPCODE_I64_XOR: u8 = 0x85;
const OPCODE_I64_SHL: u8 = 0x86;
const OPCODE_I64_SHR_S: u8 = 0x87;
const OPCODE_I64_SHR_U: u8 = 0x88;
const OPCODE_I64_ROTL: u8 = 0x89;
const OPCODE_I64_ROTR: u8 = 0x8a;
const OPCODE_F32_ABS: u8 = 0x8b;
const OPCODE_F32_NEG: u8 = 0x8c;
const OPCODE_F32_CEIL: u8 = 0x8d;
const OPCODE_F32_FLOOR: u8 = 0x8e;
const OPCODE_F32_TRUNC: u8 = 0x8f;
const OPCODE_F32_NEAREST: u8 = 0x90;
const OPCODE_F32_SQRT: u8 = 0x91;
const OPCODE_F32_ADD: u8 = 0x92;
const OPCODE_F32_SUB: u8 = 0x93;
const OPCODE_F32_MUL: u8 = 0x94;
const OPCODE_F32_DIV: u8 = 0x95;
const OPCODE_F32_MIN: u8 = 0x96;
const OPCODE_F32_MAX: u8 = 0x97;
const OPCODE_F32_COPYSIGN: u8 = 0x98;
const OPCODE_F64_ABS: u8 = 0x99;
const OPCODE_F64_NEG: u8 = 0x9a;
const OPCODE_F64_CEIL: u8 = 0x9b;
const OPCODE_F64_FLOOR: u8 = 0x9c;
const OPCODE_F64_TRUNC: u8 = 0x9d;
const OPCODE_F64_NEAREST: u8 = 0x9e;
const OPCODE_F64_SQRT: u8 = 0x9f;
const OPCODE_F64_ADD: u8 = 0xa0;
const OPCODE_F64_SUB: u8 = 0xa1;
const OPCODE_F64_MUL: u8 = 0xa2;
const OPCODE_F64_DIV: u8 = 0xa3;
const OPCODE_F64_MIN: u8 = 0xa4;
const OPCODE_F64_MAX: u8 = 0xa5;
const OPCODE_F64_COPYSIGN: u8 = 0xa6;
const OPCODE_I32_WRAP_I64: u8 = 0xa7;
const OPCODE_I32_TRUNC_F32_S: u8 = 0xa8;
const OPCODE_I32_TRUNC_F32_U: u8 = 0xa9;
const OPCODE_I32_TRUNC_F64_S: u8 = 0xaa;
const OPCODE_I32_TRUNC_F64_U: u8 = 0xab;
const OPCODE_I64_EXTEND_I32_S: u8 = 0xac;
const OPCODE_I64_EXTEND_I32_U: u8 = 0xad;
const OPCODE_I64_TRUNC_F32_S: u8 = 0xae;
const OPCODE_I64_TRUNC_F32_U: u8 = 0xaf;
const OPCODE_I64_TRUNC_F64_S: u8 = 0xb0;
const OPCODE_I64_TRUNC_F64_U: u8 = 0xb1;
const OPCODE_F32_CONVERT_I32_S: u8 = 0xb2;
const OPCODE_F32_CONVERT_I32_U: u8 = 0xb3;
const OPCODE_F32_CONVERT_I64_S: u8 = 0xb4;
const OPCODE_F32_CONVERT_I64_U: u8 = 0xb5;
const OPCODE_F32_DEMOTE_F64: u8 = 0xb6;
const OPCODE_F64_CONVERT_I32_S: u8 = 0xb7;
const OPCODE_F64_CONVERT_I32_U: u8 = 0xb8;
const OPCODE_F64_CONVERT_I64_S: u8 = 0xb9;
const OPCODE_F64_CONVERT_I64_U: u8 = 0xba;
const OPCODE_F64_PROMOTE_F32: u8 = 0xbb;
const OPCODE_I32_REINTERPRET_F32: u8 = 0xbc;
const OPCODE_I64_REINTERPRET_F64: u8 = 0xbd;
const OPCODE_F32_REINTERPRET_I32: u8 = 0xbe;
const OPCODE_F64_REINTERPRET_I64: u8 = 0xbf;
const OPCODE_I32_EXTEND8_S: u8 = 0xc0;
const OPCODE_I32_EXTEND16_S: u8 = 0xc1;
const OPCODE_I64_EXTEND8_S: u8 = 0xc2;
const OPCODE_I64_EXTEND16_S: u8 = 0xc3;
const OPCODE_I64_EXTEND32_S: u8 = 0xc4;
const OPCODE_REF_NULL: u8 = 0xd0;
const OPCODE_REF_IS_NULL: u8 = 0xd1;
const OPCODE_REF_FUNC: u8 = 0xd2;
const OPCODE_MISC: u8 = 0xfc;
const OPCODE_END: u8 = 0x0b;

const MAX_FUNC_TYPES: usize = 4;
const STACK_CAPACITY: usize = 8;
const DEFAULT_RESUME_FUEL: u32 = 1024;
const LOG_IMPORT_INDEX: u32 = 0;
const YIELD_IMPORT_INDEX: u32 = 1;
const SLEEP_IMPORT_INDEX: u32 = 2;
const MIN_IMPORT_COUNT: u32 = 2;
const MAX_IMPORT_COUNT: u32 = 3;

const SECTION_MEMORY: u8 = 5;
const SECTION_GLOBAL: u8 = 6;
const WASIP1_IMPORT_MODULE: &[u8] = WASIP1_PREVIEW1_MODULE.as_bytes();
const WASIP1_IMPORT_FD_WRITE: &[u8] = Wasip1ImportName::FdWrite.name().as_bytes();
const WASIP1_IMPORT_FD_READ: &[u8] = Wasip1ImportName::FdRead.name().as_bytes();
const WASIP1_IMPORT_FD_FDSTAT_GET: &[u8] = Wasip1ImportName::FdFdstatGet.name().as_bytes();
const WASIP1_IMPORT_FD_CLOSE: &[u8] = Wasip1ImportName::FdClose.name().as_bytes();
const WASIP1_IMPORT_FD_PRESTAT_GET: &[u8] = Wasip1ImportName::FdPrestatGet.name().as_bytes();
const WASIP1_IMPORT_FD_PRESTAT_DIR_NAME: &[u8] =
    Wasip1ImportName::FdPrestatDirName.name().as_bytes();
const WASIP1_IMPORT_FD_FILESTAT_GET: &[u8] = Wasip1ImportName::FdFilestatGet.name().as_bytes();
const WASIP1_IMPORT_FD_READDIR: &[u8] = Wasip1ImportName::FdReaddir.name().as_bytes();
const WASIP1_IMPORT_FD_ADVISE: &[u8] = Wasip1ImportName::FdAdvise.name().as_bytes();
const WASIP1_IMPORT_FD_ALLOCATE: &[u8] = Wasip1ImportName::FdAllocate.name().as_bytes();
const WASIP1_IMPORT_FD_DATASYNC: &[u8] = Wasip1ImportName::FdDatasync.name().as_bytes();
const WASIP1_IMPORT_FD_FDSTAT_SET_FLAGS: &[u8] =
    Wasip1ImportName::FdFdstatSetFlags.name().as_bytes();
const WASIP1_IMPORT_FD_FDSTAT_SET_RIGHTS: &[u8] =
    Wasip1ImportName::FdFdstatSetRights.name().as_bytes();
const WASIP1_IMPORT_FD_FILESTAT_SET_SIZE: &[u8] =
    Wasip1ImportName::FdFilestatSetSize.name().as_bytes();
const WASIP1_IMPORT_FD_FILESTAT_SET_TIMES: &[u8] =
    Wasip1ImportName::FdFilestatSetTimes.name().as_bytes();
const WASIP1_IMPORT_FD_PREAD: &[u8] = Wasip1ImportName::FdPread.name().as_bytes();
const WASIP1_IMPORT_FD_PWRITE: &[u8] = Wasip1ImportName::FdPwrite.name().as_bytes();
const WASIP1_IMPORT_FD_RENUMBER: &[u8] = Wasip1ImportName::FdRenumber.name().as_bytes();
const WASIP1_IMPORT_FD_SEEK: &[u8] = Wasip1ImportName::FdSeek.name().as_bytes();
const WASIP1_IMPORT_FD_SYNC: &[u8] = Wasip1ImportName::FdSync.name().as_bytes();
const WASIP1_IMPORT_FD_TELL: &[u8] = Wasip1ImportName::FdTell.name().as_bytes();
const WASIP1_IMPORT_CLOCK_RES_GET: &[u8] = Wasip1ImportName::ClockResGet.name().as_bytes();
const WASIP1_IMPORT_CLOCK_TIME_GET: &[u8] = Wasip1ImportName::ClockTimeGet.name().as_bytes();
const WASIP1_IMPORT_POLL_ONEOFF: &[u8] = Wasip1ImportName::PollOneoff.name().as_bytes();
const WASIP1_IMPORT_SCHED_YIELD: &[u8] = Wasip1ImportName::SchedYield.name().as_bytes();
const WASIP1_IMPORT_PATH_OPEN: &[u8] = Wasip1ImportName::PathOpen.name().as_bytes();
const WASIP1_IMPORT_PATH_FILESTAT_GET: &[u8] = Wasip1ImportName::PathFilestatGet.name().as_bytes();
const WASIP1_IMPORT_PATH_READLINK: &[u8] = Wasip1ImportName::PathReadlink.name().as_bytes();
const WASIP1_IMPORT_PATH_CREATE_DIRECTORY: &[u8] =
    Wasip1ImportName::PathCreateDirectory.name().as_bytes();
const WASIP1_IMPORT_PATH_REMOVE_DIRECTORY: &[u8] =
    Wasip1ImportName::PathRemoveDirectory.name().as_bytes();
const WASIP1_IMPORT_PATH_UNLINK_FILE: &[u8] = Wasip1ImportName::PathUnlinkFile.name().as_bytes();
const WASIP1_IMPORT_PATH_RENAME: &[u8] = Wasip1ImportName::PathRename.name().as_bytes();
const WASIP1_IMPORT_PATH_FILESTAT_SET_TIMES: &[u8] =
    Wasip1ImportName::PathFilestatSetTimes.name().as_bytes();
const WASIP1_IMPORT_PATH_LINK: &[u8] = Wasip1ImportName::PathLink.name().as_bytes();
const WASIP1_IMPORT_PATH_SYMLINK: &[u8] = Wasip1ImportName::PathSymlink.name().as_bytes();
const WASIP1_IMPORT_ARGS_GET: &[u8] = Wasip1ImportName::ArgsGet.name().as_bytes();
const WASIP1_IMPORT_ARGS_SIZES_GET: &[u8] = Wasip1ImportName::ArgsSizesGet.name().as_bytes();
const WASIP1_IMPORT_ENVIRON_GET: &[u8] = Wasip1ImportName::EnvironGet.name().as_bytes();
const WASIP1_IMPORT_ENVIRON_SIZES_GET: &[u8] = Wasip1ImportName::EnvironSizesGet.name().as_bytes();
const WASIP1_IMPORT_RANDOM_GET: &[u8] = Wasip1ImportName::RandomGet.name().as_bytes();
const WASIP1_IMPORT_PROC_EXIT: &[u8] = Wasip1ImportName::ProcExit.name().as_bytes();
const WASIP1_IMPORT_PROC_RAISE: &[u8] = Wasip1ImportName::ProcRaise.name().as_bytes();
const WASIP1_IMPORT_SOCK_ACCEPT: &[u8] = Wasip1ImportName::SockAccept.name().as_bytes();
const WASIP1_IMPORT_SOCK_RECV: &[u8] = Wasip1ImportName::SockRecv.name().as_bytes();
const WASIP1_IMPORT_SOCK_SEND: &[u8] = Wasip1ImportName::SockSend.name().as_bytes();
const WASIP1_IMPORT_SOCK_SHUTDOWN: &[u8] = Wasip1ImportName::SockShutdown.name().as_bytes();
#[cfg(test)]
const WASIP1_FD_WRITE_IMPORT_INDEX: u32 = 0;
#[cfg(test)]
const WASIP1_POLL_ONEOFF_IMPORT_INDEX: u32 = 1;
#[cfg(test)]
const WASIP1_TRAFFIC_STEP_COUNT: usize = 7;
#[cfg(test)]
const TINY_WASIP1_MEMORY_SIZE: usize = 2048;
const TINY_WASIP1_MAX_DATA_SEGMENTS: usize = 8;
#[cfg(test)]
const RUST_WASIP1_STACK_SLOP: u32 = 512;
#[cfg(test)]
const TINY_WASIP1_VALUE_STACK_CAPACITY: usize = 32;
#[cfg(test)]
const TINY_WASIP1_LOCAL_CAPACITY: usize = 16;
#[cfg(test)]
const TINY_WASIP1_CONTROL_STACK_CAPACITY: usize = 16;
#[cfg(test)]
const TINY_WASIP1_BR_TABLE_CAPACITY: usize = 8;
const WASM_BLOCKTYPE_EMPTY: u8 = 0x40;
const CORE_WASM_MAX_TYPES: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    32
} else if cfg!(any(test, feature = "wasm-engine-wasip1-std-profile")) {
    20
} else {
    16
};
const CORE_WASM_MAX_IMPORTS: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    64
} else if cfg!(feature = "wasm-engine-wasip1-std-profile") {
    16
} else {
    16
};
const CORE_WASM_MAX_FUNCTIONS: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    192
} else if cfg!(any(test, feature = "wasm-engine-wasip1-std-profile")) {
    148
} else {
    32
};
const CORE_WASM_MAX_GLOBALS: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    32
} else if cfg!(any(test, feature = "wasm-engine-wasip1-std-profile")) {
    4
} else {
    16
};
const CORE_WASM_MAX_PARAMS: usize = if cfg!(any(
    test,
    feature = "wasm-engine-wasip1-std-profile",
    feature = "wasm-engine-wasip1-full"
)) {
    12
} else if cfg!(feature = "wasip1-sys-path-minimal") {
    12
} else {
    8
};
const CORE_WASM_MAX_RESULTS: usize = 1;
const CORE_WASM_VALUE_STACK_CAPACITY: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    512
} else {
    64
};
const CORE_WASM_LOCAL_CAPACITY: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    256
} else {
    32
};
const CORE_WASM_CALL_STACK_CAPACITY: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    64
} else {
    8
};
const CORE_WASM_CONTROL_STACK_CAPACITY: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    64
} else {
    16
};
const CORE_WASM_BR_TABLE_CAPACITY: usize = 8;
const CORE_WASIP1_PATH_CAPACITY: usize = 64;
const CORE_WASM_TABLE_CAPACITY: usize = if cfg!(feature = "wasm-engine-wasip1-full") {
    96
} else if cfg!(any(test, feature = "wasm-engine-wasip1-std-profile")) {
    50
} else {
    16
};
const CORE_WASM_MAX_ELEMENT_SEGMENTS: usize = 8;
const CORE_WASM_PAGE_SIZE: usize = 64 * 1024;
const CORE_WASM_MAX_MEMORY_PAGES: u32 = if cfg!(any(test, feature = "wasm-engine-wasip1-full")) {
    32
} else {
    1
};
const CORE_WASM_MEMORY_SIZE: usize = CORE_WASM_PAGE_SIZE * CORE_WASM_MAX_MEMORY_PAGES as usize;
const WASIP1_EVENTTYPE_CLOCK: u8 = 0;
const WASIP1_SUBSCRIPTION_USERDATA_OFFSET: u32 = 0;
const WASIP1_SUBSCRIPTION_EVENTTYPE_OFFSET: u32 = 8;
const WASIP1_SUBSCRIPTION_CLOCK_TIMEOUT_OFFSET: u32 = 24;
const WASIP1_EVENT_ERROR_OFFSET: u32 = 8;
const WASIP1_EVENT_TYPE_OFFSET: u32 = 10;
const WASIP1_EVENT_SIZE: usize = 32;
pub const WASIP1_FILETYPE_UNKNOWN: u8 = 0;
pub const WASIP1_FILETYPE_DIRECTORY: u8 = 3;
pub const WASIP1_FILETYPE_REGULAR_FILE: u8 = 4;
pub const WASIP1_FDSTAT_SIZE: usize = 24;
pub const WASIP1_FDSTAT_FILETYPE_OFFSET: u32 = 0;
pub const WASIP1_FDSTAT_FLAGS_OFFSET: u32 = 2;
pub const WASIP1_FDSTAT_RIGHTS_BASE_OFFSET: u32 = 8;
pub const WASIP1_FDSTAT_RIGHTS_INHERITING_OFFSET: u32 = 16;
pub const WASIP1_PRESTAT_SIZE: usize = 8;
pub const WASIP1_PRESTAT_TAG_DIR: u8 = 0;
pub const WASIP1_PRESTAT_TAG_OFFSET: u32 = 0;
pub const WASIP1_PRESTAT_DIR_NAME_LEN_OFFSET: u32 = 4;
pub const WASIP1_FILESTAT_SIZE: usize = 64;
pub const WASIP1_FILESTAT_FILETYPE_OFFSET: u32 = 16;
pub const WASIP1_FILESTAT_SIZE_OFFSET: u32 = 32;

#[cfg(any(test, feature = "wasm-engine-wasip1-full"))]
type CoreWasmMemory = std::boxed::Box<[u8; CORE_WASM_MEMORY_SIZE]>;
#[cfg(not(any(test, feature = "wasm-engine-wasip1-full")))]
type CoreWasmMemory = [u8; CORE_WASM_MEMORY_SIZE];

pub const DEMO_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x0e, 0x01, 0x0c, 0x00, 0x41, 0xc1,
    0x84, 0xa5, 0xc2, 0x04, 0x10, 0x00, 0x10, 0x01, 0x0b,
];

pub const ROUTE_WASM_NORMAL_VALUE: u32 = 0x0000_0031;
pub const ROUTE_WASM_ALERT_VALUE: u32 = 0x4849_4241;

pub const BAD_ROUTE_EARLY_YIELD_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x06, 0x01, 0x04, 0x00, 0x10, 0x01,
    0x0b,
];

pub const SLEEP_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x3a, 0x03, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61,
    0x0b, 0x73, 0x6c, 0x65, 0x65, 0x70, 0x5f, 0x75, 0x6e, 0x74, 0x69, 0x6c, 0x00, 0x00, 0x03, 0x02,
    0x01, 0x01, 0x07, 0x0a, 0x01, 0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x03, 0x0a, 0x0a,
    0x01, 0x08, 0x00, 0x41, 0x2a, 0x10, 0x02, 0x10, 0x01, 0x0b,
];

pub const GPIO_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x37, 0x03, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61,
    0x08, 0x67, 0x70, 0x69, 0x6f, 0x5f, 0x73, 0x65, 0x74, 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x07,
    0x0a, 0x01, 0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x03, 0x0a, 0x0b, 0x01, 0x09, 0x00,
    0x41, 0x99, 0x02, 0x10, 0x02, 0x10, 0x01, 0x0b,
];

pub const TRAP_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x05, 0x01, 0x03, 0x00, 0x00, 0x0b,
];

pub const FUEL_EXHAUSTION_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x13, 0x01, 0x11, 0x00, 0x01, 0x01,
    0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01,
];

pub const NORMAL_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x0a, 0x01, 0x08, 0x00, 0x41, 0x31,
    0x10, 0x00, 0x10, 0x01, 0x0b,
];

pub const ROUTE_WASM_GUEST: &[u8] = &[
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f, 0x00, 0x60,
    0x00, 0x00, 0x02, 0x25, 0x02, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x07, 0x6c, 0x6f, 0x67,
    0x5f, 0x75, 0x33, 0x32, 0x00, 0x00, 0x06, 0x68, 0x69, 0x62, 0x61, 0x6e, 0x61, 0x09, 0x79, 0x69,
    0x65, 0x6c, 0x64, 0x5f, 0x6e, 0x6f, 0x77, 0x00, 0x01, 0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01,
    0x06, 0x5f, 0x73, 0x74, 0x61, 0x72, 0x74, 0x00, 0x02, 0x0a, 0x12, 0x01, 0x10, 0x00, 0x41, 0x31,
    0x10, 0x00, 0x41, 0xc1, 0x84, 0xa5, 0xc2, 0x04, 0x10, 0x00, 0x10, 0x01, 0x0b,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuestTrap {
    HostCall(EngineReq),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BudgetedGuestTrap {
    Guest(GuestTrap),
    BudgetExpired(BudgetExpired),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WasmError {
    Truncated,
    Invalid(&'static str),
    Unsupported(&'static str),
    UnsupportedOpcode(u8),
    StackOverflow,
    StackUnderflow,
    PendingHostCall,
    ReplyWithoutPending,
    UnexpectedReply,
    Trap,
    FuelExhausted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasmValueKind {
    I32,
    I64,
    F32,
    F64,
    FuncRef,
}

impl CoreWasmValueKind {
    fn decode(byte: u8) -> Result<Self, WasmError> {
        match byte {
            VALTYPE_I32 => Ok(Self::I32),
            VALTYPE_I64 => Ok(Self::I64),
            VALTYPE_F32 => Ok(Self::F32),
            VALTYPE_F64 => Ok(Self::F64),
            VALTYPE_FUNCREF => Ok(Self::FuncRef),
            _ => Err(WasmError::Unsupported("unsupported core value type")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasmValue {
    I32(u32),
    I64(u64),
    F32(u32),
    F64(u64),
    FuncRef(u32),
}

impl CoreWasmValue {
    const fn zero(kind: CoreWasmValueKind) -> Self {
        match kind {
            CoreWasmValueKind::I32 => Self::I32(0),
            CoreWasmValueKind::I64 => Self::I64(0),
            CoreWasmValueKind::F32 => Self::F32(0),
            CoreWasmValueKind::F64 => Self::F64(0),
            CoreWasmValueKind::FuncRef => Self::FuncRef(u32::MAX),
        }
    }

    fn kind(self) -> CoreWasmValueKind {
        match self {
            Self::I32(_) => CoreWasmValueKind::I32,
            Self::I64(_) => CoreWasmValueKind::I64,
            Self::F32(_) => CoreWasmValueKind::F32,
            Self::F64(_) => CoreWasmValueKind::F64,
            Self::FuncRef(_) => CoreWasmValueKind::FuncRef,
        }
    }

    fn as_i32(self) -> Result<u32, WasmError> {
        match self {
            Self::I32(value) => Ok(value),
            _ => Err(WasmError::Invalid("expected i32 core value")),
        }
    }

    fn as_i64(self) -> Result<u64, WasmError> {
        match self {
            Self::I64(value) => Ok(value),
            _ => Err(WasmError::Invalid("expected i64 core value")),
        }
    }

    fn as_f32_bits(self) -> Result<u32, WasmError> {
        match self {
            Self::F32(value) => Ok(value),
            _ => Err(WasmError::Invalid("expected f32 core value")),
        }
    }

    fn as_f64_bits(self) -> Result<u64, WasmError> {
        match self {
            Self::F64(value) => Ok(value),
            _ => Err(WasmError::Invalid("expected f64 core value")),
        }
    }

    fn as_f32(self) -> Result<f32, WasmError> {
        Ok(f32::from_bits(self.as_f32_bits()?))
    }

    fn as_f64(self) -> Result<f64, WasmError> {
        Ok(f64::from_bits(self.as_f64_bits()?))
    }

    fn as_funcref(self) -> Result<u32, WasmError> {
        match self {
            Self::FuncRef(value) => Ok(value),
            _ => Err(WasmError::Invalid("expected funcref core value")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmFuncType {
    params: [CoreWasmValueKind; CORE_WASM_MAX_PARAMS],
    param_count: usize,
    results: [CoreWasmValueKind; CORE_WASM_MAX_RESULTS],
    result_count: usize,
}

impl CoreWasmFuncType {
    const EMPTY: Self = Self {
        params: [CoreWasmValueKind::I32; CORE_WASM_MAX_PARAMS],
        param_count: 0,
        results: [CoreWasmValueKind::I32; CORE_WASM_MAX_RESULTS],
        result_count: 0,
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasmImport<'a> {
    pub function_index: u32,
    pub module: &'a [u8],
    pub name: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasmHostImport<'a> {
    pub import: CoreWasmImport<'a>,
    pub args: [CoreWasmValue; CORE_WASM_MAX_PARAMS],
    pub arg_count: usize,
    pub result_count: usize,
}

impl<'a> CoreWasmHostImport<'a> {
    pub fn args(&self) -> &[CoreWasmValue] {
        &self.args[..self.arg_count]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasmMemoryGrow {
    pub previous_pages: u32,
    pub requested_pages: u32,
    pub new_pages: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasmTrap<'a> {
    HostImport(CoreWasmHostImport<'a>),
    MemoryGrow(CoreWasmMemoryGrow),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasip1Trap {
    FdWrite(TinyWasip1FdWriteCall),
    FdRead(CoreWasip1FdReadCall),
    FdFdstatGet(CoreWasip1FdRequestCall),
    FdClose(CoreWasip1FdRequestCall),
    ClockResGet(CoreWasip1ClockResGetCall),
    ClockTimeGet(CoreWasip1ClockTimeGetCall),
    PollOneoff(TinyWasip1PollOneoffCall),
    RandomGet(CoreWasip1RandomGetCall),
    SchedYield,
    PathMinimal(CoreWasip1PathCall),
    PathFull(CoreWasip1PathCall),
    Socket(CoreWasip1SocketCall),
    ArgsSizesGet(CoreWasip1ArgsSizesGetCall),
    ArgsGet(CoreWasip1ArgsGetCall),
    EnvironSizesGet(CoreWasip1EnvironSizesGetCall),
    EnvironGet(CoreWasip1EnvironGetCall),
    ProcExit(u32),
    ProcRaise(u32),
    MemoryGrow(CoreWasmMemoryGrow),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasip1PathKind {
    FdPrestatGet,
    FdPrestatDirName,
    FdFilestatGet,
    FdReaddir,
    FdAdvise,
    FdAllocate,
    FdDatasync,
    FdFdstatSetFlags,
    FdFdstatSetRights,
    FdFilestatSetSize,
    FdFilestatSetTimes,
    FdPread,
    FdPwrite,
    FdRenumber,
    FdSeek,
    FdSync,
    FdTell,
    PathOpen,
    PathFilestatGet,
    PathReadlink,
    PathCreateDirectory,
    PathRemoveDirectory,
    PathUnlinkFile,
    PathRename,
    PathFilestatSetTimes,
    PathLink,
    PathSymlink,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CoreWasip1SocketKind {
    SockAccept,
    SockRecv,
    SockSend,
    SockShutdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1SocketCall {
    kind: CoreWasip1SocketKind,
    args: [CoreWasmValue; CORE_WASM_MAX_PARAMS],
    arg_count: usize,
}

impl CoreWasip1SocketCall {
    pub const fn kind(self) -> CoreWasip1SocketKind {
        self.kind
    }

    pub fn args(&self) -> &[CoreWasmValue] {
        &self.args[..self.arg_count]
    }

    fn arg_i32(&self, index: usize) -> Result<u32, WasmError> {
        self.args
            .get(index)
            .copied()
            .ok_or(WasmError::Invalid("socket import argument missing"))?
            .as_i32()
    }

    pub fn fd(&self) -> Result<u8, WasmError> {
        let fd = self.arg_i32(0)?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(fd as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1PathCall {
    kind: CoreWasip1PathKind,
    args: [CoreWasmValue; CORE_WASM_MAX_PARAMS],
    arg_count: usize,
}

impl CoreWasip1PathCall {
    pub const fn kind(self) -> CoreWasip1PathKind {
        self.kind
    }

    pub fn args(&self) -> &[CoreWasmValue] {
        &self.args[..self.arg_count]
    }

    pub fn arg_i32(&self, index: usize) -> Result<u32, WasmError> {
        self.args
            .get(index)
            .copied()
            .ok_or(WasmError::Invalid("path import argument missing"))?
            .as_i32()
    }

    pub fn arg_i64(&self, index: usize) -> Result<u64, WasmError> {
        self.args
            .get(index)
            .copied()
            .ok_or(WasmError::Invalid("path import argument missing"))?
            .as_i64()
    }

    pub fn fd(&self) -> Result<u8, WasmError> {
        let fd = self.arg_i32(0)?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(fd as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1PathBytes {
    bytes: [u8; CORE_WASIP1_PATH_CAPACITY],
    len: usize,
}

impl CoreWasip1PathBytes {
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.split_at(self.len).0
    }

    pub const fn len(&self) -> usize {
        self.len
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1FdStat {
    filetype: u8,
    flags: u16,
    rights_base: u64,
    rights_inheriting: u64,
}

impl CoreWasip1FdStat {
    pub const fn new(filetype: u8, flags: u16, rights_base: u64, rights_inheriting: u64) -> Self {
        Self {
            filetype,
            flags,
            rights_base,
            rights_inheriting,
        }
    }

    pub const fn filetype(self) -> u8 {
        self.filetype
    }

    pub const fn flags(self) -> u16 {
        self.flags
    }

    pub const fn rights_base(self) -> u64 {
        self.rights_base
    }

    pub const fn rights_inheriting(self) -> u64 {
        self.rights_inheriting
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1FileStat {
    filetype: u8,
    size: u64,
}

impl CoreWasip1FileStat {
    pub const fn new(filetype: u8, size: u64) -> Self {
        Self { filetype, size }
    }

    pub const fn filetype(self) -> u8 {
        self.filetype
    }

    pub const fn size(self) -> u64 {
        self.size
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1FdReadCall {
    fd: u8,
    iovs: u32,
    iovs_len: u32,
    nread: u32,
}

impl CoreWasip1FdReadCall {
    pub const fn fd(self) -> u8 {
        self.fd
    }

    pub const fn iovs(self) -> u32 {
        self.iovs
    }

    pub const fn iovs_len(self) -> u32 {
        self.iovs_len
    }

    pub const fn nread(self) -> u32 {
        self.nread
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1FdRequestCall {
    fd: u8,
    out_ptr: u32,
}

impl CoreWasip1FdRequestCall {
    pub const fn fd(self) -> u8 {
        self.fd
    }

    pub const fn out_ptr(self) -> u32 {
        self.out_ptr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1ClockResGetCall {
    clock_id: u32,
    resolution_ptr: u32,
}

impl CoreWasip1ClockResGetCall {
    pub const fn clock_id(self) -> u32 {
        self.clock_id
    }

    pub const fn resolution_ptr(self) -> u32 {
        self.resolution_ptr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1ClockTimeGetCall {
    clock_id: u32,
    precision: u64,
    time_ptr: u32,
}

impl CoreWasip1ClockTimeGetCall {
    pub const fn clock_id(self) -> u32 {
        self.clock_id
    }

    pub const fn precision(self) -> u64 {
        self.precision
    }

    pub const fn time_ptr(self) -> u32 {
        self.time_ptr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1RandomGetCall {
    buf: u32,
    buf_len: u32,
}

impl CoreWasip1RandomGetCall {
    pub const fn buf(self) -> u32 {
        self.buf
    }

    pub const fn buf_len(self) -> u32 {
        self.buf_len
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1ArgsSizesGetCall {
    argc_ptr: u32,
    argv_buf_size_ptr: u32,
}

impl CoreWasip1ArgsSizesGetCall {
    pub const fn argc_ptr(self) -> u32 {
        self.argc_ptr
    }

    pub const fn argv_buf_size_ptr(self) -> u32 {
        self.argv_buf_size_ptr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1ArgsGetCall {
    argv: u32,
    argv_buf: u32,
}

impl CoreWasip1ArgsGetCall {
    pub const fn argv(self) -> u32 {
        self.argv
    }

    pub const fn argv_buf(self) -> u32 {
        self.argv_buf
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1EnvironSizesGetCall {
    environ_count_ptr: u32,
    environ_buf_size_ptr: u32,
}

impl CoreWasip1EnvironSizesGetCall {
    pub const fn environ_count_ptr(self) -> u32 {
        self.environ_count_ptr
    }

    pub const fn environ_buf_size_ptr(self) -> u32 {
        self.environ_buf_size_ptr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoreWasip1EnvironGetCall {
    environ: u32,
    environ_buf: u32,
}

impl CoreWasip1EnvironGetCall {
    pub const fn environ(self) -> u32 {
        self.environ
    }

    pub const fn environ_buf(self) -> u32 {
        self.environ_buf
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmCodeBody<'a> {
    code: &'a [u8],
    local_count: usize,
    local_kinds: [CoreWasmValueKind; CORE_WASM_LOCAL_CAPACITY],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmDataSegment<'a> {
    active: bool,
    offset: u32,
    bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmElementSegment {
    functions: [u32; CORE_WASM_TABLE_CAPACITY],
    function_count: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmGlobal {
    kind: CoreWasmValueKind,
    mutable: bool,
    initial: CoreWasmValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CoreWasmFrame<'a> {
    code: &'a [u8],
    pc: usize,
    locals: [CoreWasmValue; CORE_WASM_LOCAL_CAPACITY],
    local_kinds: [CoreWasmValueKind; CORE_WASM_LOCAL_CAPACITY],
    local_count: usize,
    controls: [ControlFrame; CORE_WASM_CONTROL_STACK_CAPACITY],
    control_len: usize,
}

impl<'a> CoreWasmFrame<'a> {
    fn empty() -> Self {
        Self {
            code: &[],
            pc: 0,
            locals: [CoreWasmValue::I32(0); CORE_WASM_LOCAL_CAPACITY],
            local_kinds: [CoreWasmValueKind::I32; CORE_WASM_LOCAL_CAPACITY],
            local_count: 0,
            controls: [ControlFrame {
                kind: ControlKind::Block,
                start_pos: 0,
                else_pos: usize::MAX,
                end_pos: 0,
                result_count: 0,
                result_kind: CoreWasmValueKind::I32,
                stack_height: 0,
            }; CORE_WASM_CONTROL_STACK_CAPACITY],
            control_len: 0,
        }
    }
}

#[cfg(any(test, feature = "wasm-engine-wasip1-full"))]
type CoreWasmFrames<'a> = std::boxed::Box<[CoreWasmFrame<'a>; CORE_WASM_CALL_STACK_CAPACITY]>;
#[cfg(not(any(test, feature = "wasm-engine-wasip1-full")))]
type CoreWasmFrames<'a> = [CoreWasmFrame<'a>; CORE_WASM_CALL_STACK_CAPACITY];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CoreWasmPending<'a> {
    HostImport(CoreWasmHostImport<'a>),
    MemoryGrow(CoreWasmMemoryGrow),
}

#[derive(Clone, Copy)]
pub struct CoreWasmModule<'a> {
    types: [CoreWasmFuncType; CORE_WASM_MAX_TYPES],
    type_count: usize,
    imports: [Option<CoreWasmImport<'a>>; CORE_WASM_MAX_IMPORTS],
    import_type_indices: [u32; CORE_WASM_MAX_IMPORTS],
    import_count: usize,
    function_type_indices: [u32; CORE_WASM_MAX_FUNCTIONS],
    function_count: usize,
    globals: [Option<CoreWasmGlobal>; CORE_WASM_MAX_GLOBALS],
    global_count: usize,
    code_bodies: [Option<CoreWasmCodeBody<'a>>; CORE_WASM_MAX_FUNCTIONS],
    data_segments: [Option<CoreWasmDataSegment<'a>>; TINY_WASIP1_MAX_DATA_SEGMENTS],
    element_segments: [Option<CoreWasmElementSegment>; CORE_WASM_MAX_ELEMENT_SEGMENTS],
    table_functions: [u32; CORE_WASM_TABLE_CAPACITY],
    table_function_count: usize,
    table_min: usize,
    start_function_index: u32,
    memory_min_pages: u32,
    memory_max_pages: u32,
}

pub struct CoreWasmInstance<'a> {
    module: CoreWasmModule<'a>,
    frames: CoreWasmFrames<'a>,
    frame_len: usize,
    values: [CoreWasmValue; CORE_WASM_VALUE_STACK_CAPACITY],
    value_len: usize,
    globals: [CoreWasmValue; CORE_WASM_MAX_GLOBALS],
    global_kinds: [CoreWasmValueKind; CORE_WASM_MAX_GLOBALS],
    global_mutable: [bool; CORE_WASM_MAX_GLOBALS],
    global_count: usize,
    memory: CoreWasmMemory,
    memory_pages: u32,
    data_dropped: [bool; TINY_WASIP1_MAX_DATA_SEGMENTS],
    element_dropped: [bool; CORE_WASM_MAX_ELEMENT_SEGMENTS],
    table_functions: [u32; CORE_WASM_TABLE_CAPACITY],
    table_size: usize,
    pending: Option<CoreWasmPending<'a>>,
    done: bool,
}

pub struct CoreWasip1Instance<'a> {
    core: CoreWasmInstance<'a>,
    handlers: Wasip1HandlerSet,
    done: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
enum FuncSig {
    #[default]
    Unsupported,
    I32ToUnit,
    UnitToUnit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PendingHostCall {
    LogU32(u32),
    Yield,
    SleepUntil(u64),
    GpioSet(GpioSet),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OptionalImport {
    SleepUntil,
    GpioSet,
}

#[derive(Clone, Copy)]
pub struct TinyWasmModule<'a> {
    start_body: &'a [u8],
    optional_import: Option<OptionalImport>,
}

pub struct TinyWasmInstance<'a> {
    code: &'a [u8],
    pc: usize,
    stack: [i32; STACK_CAPACITY],
    stack_len: usize,
    pending: Option<PendingHostCall>,
    done: bool,
    optional_import: Option<OptionalImport>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
#[doc(hidden)]
pub struct TinyWasip1TrafficStep {
    pub fd: u8,
    pub payload: u8,
    pub delay_ticks: u64,
}

#[cfg(test)]
impl TinyWasip1TrafficStep {
    #[doc(hidden)]
    pub const fn new(fd: u8, payload: u8, delay_ticks: u64) -> Self {
        Self {
            fd,
            payload,
            delay_ticks,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TinyWasip1FdWriteCall {
    fd: u8,
    iovs: u32,
    iovs_len: u32,
    nwritten: u32,
}

impl TinyWasip1FdWriteCall {
    pub const fn fd(self) -> u8 {
        self.fd
    }

    pub const fn iovs(self) -> u32 {
        self.iovs
    }

    pub const fn iovs_len(self) -> u32 {
        self.iovs_len
    }

    pub const fn nwritten(self) -> u32 {
        self.nwritten
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TinyWasip1PollOneoffCall {
    in_ptr: u32,
    out_ptr: u32,
    nsubscriptions: u32,
    nevents: u32,
}

impl TinyWasip1PollOneoffCall {
    pub const fn in_ptr(self) -> u32 {
        self.in_ptr
    }

    pub const fn out_ptr(self) -> u32 {
        self.out_ptr
    }

    pub const fn nsubscriptions(self) -> u32 {
        self.nsubscriptions
    }

    pub const fn nevents(self) -> u32 {
        self.nevents
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
#[doc(hidden)]
pub enum TinyWasip1Trap {
    FdWrite(TinyWasip1FdWriteCall),
    PollOneoff(TinyWasip1PollOneoffCall),
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
enum PendingWasip1Return {
    ErrnoI32Stack,
    ErrnoValueStack,
    FdWriteLen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
enum PendingWasip1Call {
    FdWrite(TinyWasip1FdWriteCall, PendingWasip1Return),
    PollOneoff(TinyWasip1PollOneoffCall, PendingWasip1Return),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
enum Wasip1ValueKind {
    I32,
    I64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
enum Wasip1Value {
    I32(u32),
    I64(u64),
}

#[cfg(test)]
impl Wasip1Value {
    const fn zero(kind: Wasip1ValueKind) -> Self {
        match kind {
            Wasip1ValueKind::I32 => Self::I32(0),
            Wasip1ValueKind::I64 => Self::I64(0),
        }
    }

    fn as_i32(self) -> Result<u32, WasmError> {
        match self {
            Self::I32(value) => Ok(value),
            Self::I64(_) => Err(WasmError::Invalid("expected i32 stack value")),
        }
    }

    fn as_i64(self) -> Result<u64, WasmError> {
        match self {
            Self::I64(value) => Ok(value),
            Self::I32(_) => Err(WasmError::Invalid("expected i64 stack value")),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct TinyWasip1CodeBody<'a> {
    code: &'a [u8],
    local_count: usize,
    local_kinds: [Wasip1ValueKind; TINY_WASIP1_LOCAL_CAPACITY],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlKind {
    Block,
    Loop,
    If,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControlFrame {
    kind: ControlKind,
    start_pos: usize,
    else_pos: usize,
    end_pos: usize,
    result_count: usize,
    result_kind: CoreWasmValueKind,
    stack_height: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
enum TinyWasip1Program<'a> {
    Direct {
        code: &'a [u8],
    },
    Rust {
        code: &'a [u8],
        fd_write_index: u32,
        poll_oneoff_index: u32,
    },
    RustStdMain {
        body: TinyWasip1CodeBody<'a>,
        write_index: u32,
        clock_nanosleep_index: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct TinyWasip1DataSegment<'a> {
    offset: u32,
    bytes: &'a [u8],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct ParsedTinyWasip1Module<'a> {
    program: TinyWasip1Program<'a>,
    memory_base: u32,
    traffic_steps: Option<[TinyWasip1TrafficStep; WASIP1_TRAFFIC_STEP_COUNT]>,
    data_segments: [Option<TinyWasip1DataSegment<'a>>; TINY_WASIP1_MAX_DATA_SEGMENTS],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TinyWasip1Payload {
    bytes: [u8; WASIP1_STREAM_CHUNK_CAPACITY],
    len: u8,
}

impl TinyWasip1Payload {
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.split_at(self.len as usize).0
    }
}

#[cfg(test)]
#[doc(hidden)]
pub struct TinyWasip1TrafficLightInstance<'a> {
    program: TinyWasip1Program<'a>,
    pc: usize,
    stack: [i32; STACK_CAPACITY],
    stack_len: usize,
    values: [Wasip1Value; TINY_WASIP1_VALUE_STACK_CAPACITY],
    value_len: usize,
    locals: [Wasip1Value; TINY_WASIP1_LOCAL_CAPACITY],
    local_kinds: [Wasip1ValueKind; TINY_WASIP1_LOCAL_CAPACITY],
    local_count: usize,
    controls: [ControlFrame; TINY_WASIP1_CONTROL_STACK_CAPACITY],
    control_len: usize,
    global0: u32,
    pending: Option<PendingWasip1Call>,
    done: bool,
    memory: [u8; TINY_WASIP1_MEMORY_SIZE],
    memory_base: u32,
    fd_write_count: usize,
    poll_count: usize,
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    fn is_empty(&self) -> bool {
        self.pos == self.bytes.len()
    }

    fn read_u8(&mut self) -> Result<u8, WasmError> {
        let byte = *self.bytes.get(self.pos).ok_or(WasmError::Truncated)?;
        self.pos += 1;
        Ok(byte)
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8], WasmError> {
        let end = self.pos.checked_add(len).ok_or(WasmError::Truncated)?;
        let slice = self.bytes.get(self.pos..end).ok_or(WasmError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    fn read_fixed_u32(&mut self) -> Result<u32, WasmError> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_fixed_u64(&mut self) -> Result<u64, WasmError> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn read_name(&mut self) -> Result<&'a [u8], WasmError> {
        let len = self.read_var_u32()? as usize;
        self.read_bytes(len)
    }

    fn read_var_u32(&mut self) -> Result<u32, WasmError> {
        let mut shift = 0u32;
        let mut value = 0u32;
        loop {
            if shift >= 35 {
                return Err(WasmError::Invalid("u32 leb too wide"));
            }
            let byte = self.read_u8()?;
            value |= ((byte & 0x7f) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
        }
    }

    fn read_var_i32(&mut self) -> Result<i32, WasmError> {
        let mut shift = 0u32;
        let mut value = 0i32;
        let mut byte;
        loop {
            if shift >= 35 {
                return Err(WasmError::Invalid("i32 leb too wide"));
            }
            byte = self.read_u8()?;
            value |= ((byte & 0x7f) as i32) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        if shift < 32 && (byte & 0x40) != 0 {
            value |= (!0i32) << shift;
        }
        Ok(value)
    }

    fn read_var_i64(&mut self) -> Result<i64, WasmError> {
        let mut shift = 0u32;
        let mut value = 0i64;
        let mut byte;
        loop {
            if shift >= 70 {
                return Err(WasmError::Invalid("i64 leb too wide"));
            }
            byte = self.read_u8()?;
            value |= ((byte & 0x7f) as i64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
        }
        if shift < 64 && (byte & 0x40) != 0 {
            value |= (!0i64) << shift;
        }
        Ok(value)
    }
}

impl<'a> CoreWasmModule<'a> {
    const fn empty() -> Self {
        Self {
            types: [CoreWasmFuncType::EMPTY; CORE_WASM_MAX_TYPES],
            type_count: 0,
            imports: [None; CORE_WASM_MAX_IMPORTS],
            import_type_indices: [0; CORE_WASM_MAX_IMPORTS],
            import_count: 0,
            function_type_indices: [0; CORE_WASM_MAX_FUNCTIONS],
            function_count: 0,
            globals: [None; CORE_WASM_MAX_GLOBALS],
            global_count: 0,
            code_bodies: [None; CORE_WASM_MAX_FUNCTIONS],
            data_segments: [None; TINY_WASIP1_MAX_DATA_SEGMENTS],
            element_segments: [None; CORE_WASM_MAX_ELEMENT_SEGMENTS],
            table_functions: [u32::MAX; CORE_WASM_TABLE_CAPACITY],
            table_function_count: 0,
            table_min: 0,
            start_function_index: u32::MAX,
            memory_min_pages: 0,
            memory_max_pages: 0,
        }
    }

    pub fn parse(bytes: &'a [u8]) -> Result<Self, WasmError> {
        let mut module = Self::empty();
        module.parse_from(bytes)?;
        Ok(module)
    }

    pub fn parse_in_place<'slot>(
        bytes: &'a [u8],
        slot: &'slot mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'slot mut Self, WasmError> {
        let ptr = slot.as_mut_ptr();
        unsafe {
            ptr.write(Self::empty());
        }
        let module = unsafe { &mut *ptr };
        module.parse_from(bytes)?;
        Ok(module)
    }

    fn parse_from(&mut self, bytes: &'a [u8]) -> Result<(), WasmError> {
        let mut reader = Reader::new(bytes);
        if reader.read_bytes(4)? != WASM_MAGIC {
            return Err(WasmError::Invalid("invalid wasm magic"));
        }
        if reader.read_bytes(4)? != WASM_VERSION {
            return Err(WasmError::Invalid("unsupported wasm version"));
        }

        let mut saw_export = false;

        while !reader.is_empty() {
            let section_id = reader.read_u8()?;
            let section_len = reader.read_var_u32()? as usize;
            let section_bytes = reader.read_bytes(section_len)?;
            let mut section = Reader::new(section_bytes);
            match section_id {
                SECTION_TYPE => self.parse_core_type_section(&mut section)?,
                SECTION_IMPORT => self.parse_core_import_section(&mut section)?,
                SECTION_FUNCTION => self.parse_core_function_section(&mut section)?,
                SECTION_TABLE => self.parse_core_table_section(&mut section)?,
                SECTION_MEMORY => self.parse_core_memory_section(&mut section)?,
                SECTION_GLOBAL => self.parse_core_global_section(&mut section)?,
                SECTION_EXPORT => {
                    self.parse_core_export_section(&mut section)?;
                    saw_export = true;
                }
                SECTION_ELEMENT => self.parse_core_element_section(&mut section)?,
                SECTION_CODE => self.parse_core_code_section(&mut section)?,
                SECTION_DATA => self.parse_core_data_section(&mut section)?,
                SECTION_CUSTOM => {
                    let _ = section.read_bytes(section.bytes.len().saturating_sub(section.pos))?;
                }
                _ => return Err(WasmError::Unsupported("unsupported core wasm section")),
            }
            if !section.is_empty() {
                return Err(WasmError::Invalid("section has trailing bytes"));
            }
        }

        if !saw_export || self.start_function_index == u32::MAX {
            return Err(WasmError::Invalid("missing _start or __main_void export"));
        }
        if self.function_count > 0
            && self.code_bodies[..self.function_count]
                .iter()
                .any(Option::is_none)
        {
            return Err(WasmError::Invalid("missing core wasm code body"));
        }
        Ok(())
    }

    pub fn instantiate(self) -> Result<CoreWasmInstance<'a>, WasmError> {
        if self.memory_min_pages > CORE_WASM_MAX_MEMORY_PAGES {
            return Err(WasmError::Unsupported("core wasm memory too large"));
        }
        let mut instance = CoreWasmInstance {
            memory_pages: self.memory_min_pages,
            module: self,
            frames: core_wasm_frames_empty()?,
            frame_len: 0,
            values: [CoreWasmValue::I32(0); CORE_WASM_VALUE_STACK_CAPACITY],
            value_len: 0,
            globals: [CoreWasmValue::I32(0); CORE_WASM_MAX_GLOBALS],
            global_kinds: [CoreWasmValueKind::I32; CORE_WASM_MAX_GLOBALS],
            global_mutable: [false; CORE_WASM_MAX_GLOBALS],
            global_count: self.global_count,
            memory: core_wasm_memory_zeroed()?,
            data_dropped: [false; TINY_WASIP1_MAX_DATA_SEGMENTS],
            element_dropped: [false; CORE_WASM_MAX_ELEMENT_SEGMENTS],
            table_functions: [u32::MAX; CORE_WASM_TABLE_CAPACITY],
            table_size: self.table_min.max(self.table_function_count),
            pending: None,
            done: false,
        };
        instance.table_functions = self.table_functions;
        for (index, global) in self
            .globals
            .iter()
            .copied()
            .flatten()
            .take(self.global_count)
            .enumerate()
        {
            instance.globals[index] = global.initial;
            instance.global_kinds[index] = global.kind;
            instance.global_mutable[index] = global.mutable;
        }
        instance.init_core_data_segments()?;
        instance.push_frame(instance.module.start_function_index)?;
        Ok(instance)
    }

    pub fn instantiate_in_place<'slot>(
        self,
        slot: &'slot mut core::mem::MaybeUninit<CoreWasmInstance<'a>>,
    ) -> Result<&'slot mut CoreWasmInstance<'a>, WasmError> {
        if self.memory_min_pages > CORE_WASM_MAX_MEMORY_PAGES {
            return Err(WasmError::Unsupported("core wasm memory too large"));
        }

        let ptr = slot.as_mut_ptr();
        unsafe {
            core::ptr::addr_of_mut!((*ptr).module).write(self);
            core::ptr::addr_of_mut!((*ptr).frames).write(core_wasm_frames_empty()?);
            core::ptr::addr_of_mut!((*ptr).frame_len).write(0);
            core::ptr::addr_of_mut!((*ptr).values)
                .write([CoreWasmValue::I32(0); CORE_WASM_VALUE_STACK_CAPACITY]);
            core::ptr::addr_of_mut!((*ptr).value_len).write(0);
            core::ptr::addr_of_mut!((*ptr).globals)
                .write([CoreWasmValue::I32(0); CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_kinds)
                .write([CoreWasmValueKind::I32; CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_mutable).write([false; CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_count).write(self.global_count);
            write_core_wasm_memory_zeroed(core::ptr::addr_of_mut!((*ptr).memory))?;
            core::ptr::addr_of_mut!((*ptr).memory_pages).write(self.memory_min_pages);
            core::ptr::addr_of_mut!((*ptr).data_dropped)
                .write([false; TINY_WASIP1_MAX_DATA_SEGMENTS]);
            core::ptr::addr_of_mut!((*ptr).element_dropped)
                .write([false; CORE_WASM_MAX_ELEMENT_SEGMENTS]);
            core::ptr::addr_of_mut!((*ptr).table_functions).write(self.table_functions);
            core::ptr::addr_of_mut!((*ptr).table_size)
                .write(self.table_min.max(self.table_function_count));
            core::ptr::addr_of_mut!((*ptr).pending).write(None);
            core::ptr::addr_of_mut!((*ptr).done).write(false);
        }

        let instance = unsafe { &mut *ptr };
        for (index, global) in self
            .globals
            .iter()
            .copied()
            .flatten()
            .take(self.global_count)
            .enumerate()
        {
            instance.globals[index] = global.initial;
            instance.global_kinds[index] = global.kind;
            instance.global_mutable[index] = global.mutable;
        }
        instance.init_core_data_segments()?;
        instance.push_frame(instance.module.start_function_index)?;
        Ok(instance)
    }

    fn parse_core_type_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > CORE_WASM_MAX_TYPES {
            return Err(WasmError::Unsupported("too many core wasm function types"));
        }
        self.type_count = count;
        for index in 0..count {
            if section.read_u8()? != FUNC_TYPE_FORM {
                return Err(WasmError::Invalid("type section expects function forms"));
            }
            let param_count = section.read_var_u32()? as usize;
            if param_count > CORE_WASM_MAX_PARAMS {
                return Err(WasmError::Unsupported("too many core wasm params"));
            }
            let mut ty = CoreWasmFuncType::EMPTY;
            ty.param_count = param_count;
            for slot in ty.params.iter_mut().take(param_count) {
                *slot = CoreWasmValueKind::decode(section.read_u8()?)?;
            }

            let result_count = section.read_var_u32()? as usize;
            if result_count > CORE_WASM_MAX_RESULTS {
                return Err(WasmError::Unsupported("too many core wasm results"));
            }
            ty.result_count = result_count;
            for slot in ty.results.iter_mut().take(result_count) {
                *slot = CoreWasmValueKind::decode(section.read_u8()?)?;
            }
            self.types[index] = ty;
        }
        Ok(())
    }

    fn parse_core_import_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > CORE_WASM_MAX_IMPORTS {
            return Err(WasmError::Unsupported("too many core wasm imports"));
        }
        self.import_count = count;
        for index in 0..count {
            let module = section.read_name()?;
            let name = section.read_name()?;
            if section.read_u8()? != EXTERNAL_KIND_FUNC {
                return Err(WasmError::Unsupported(
                    "core wasm only supports function imports",
                ));
            }
            let type_index = section.read_var_u32()?;
            self.core_func_type(type_index)?;
            self.imports[index] = Some(CoreWasmImport {
                function_index: index as u32,
                module,
                name,
            });
            self.import_type_indices[index] = type_index;
        }
        Ok(())
    }

    fn parse_core_function_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > CORE_WASM_MAX_FUNCTIONS {
            return Err(WasmError::Unsupported("too many core wasm functions"));
        }
        self.function_count = count;
        for index in 0..count {
            let type_index = section.read_var_u32()?;
            self.core_func_type(type_index)?;
            self.function_type_indices[index] = type_index;
        }
        Ok(())
    }

    fn parse_core_table_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()?;
        if count > 1 {
            return Err(WasmError::Unsupported("too many core wasm tables"));
        }
        for _ in 0..count {
            if section.read_u8()? != VALTYPE_FUNCREF {
                return Err(WasmError::Unsupported("only funcref tables are supported"));
            }
            let flags = section.read_u8()?;
            if flags & !0x01 != 0 {
                return Err(WasmError::Unsupported("unsupported core table limits"));
            }
            let min = section.read_var_u32()? as usize;
            if min > CORE_WASM_TABLE_CAPACITY {
                return Err(WasmError::Unsupported("core table too large"));
            }
            self.table_min = min;
            if flags & 0x01 != 0 {
                let max = section.read_var_u32()? as usize;
                if max < min || max > CORE_WASM_TABLE_CAPACITY {
                    return Err(WasmError::Unsupported("core table limit too large"));
                }
            }
        }
        Ok(())
    }

    fn parse_core_element_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > CORE_WASM_MAX_ELEMENT_SEGMENTS {
            return Err(WasmError::Unsupported("too many core element segments"));
        }
        for slot in self.element_segments.iter_mut() {
            *slot = None;
        }
        for segment_index in 0..count {
            let kind = section.read_var_u32()?;
            match kind {
                0 => {
                    let offset = parse_core_i32_offset_expr(section)? as usize;
                    let segment = self.parse_core_funcidx_element_payload(section)?;
                    self.install_core_element_segment(offset, segment)?;
                    self.element_segments[segment_index] = Some(segment);
                }
                1 => {
                    if section.read_u8()? != 0 {
                        return Err(WasmError::Unsupported(
                            "only function element kind supported",
                        ));
                    }
                    let segment = self.parse_core_funcidx_element_payload(section)?;
                    self.element_segments[segment_index] = Some(segment);
                }
                2 => {
                    if section.read_var_u32()? != 0 {
                        return Err(WasmError::Invalid("core element table index must be zero"));
                    }
                    let offset = parse_core_i32_offset_expr(section)? as usize;
                    if section.read_u8()? != 0 {
                        return Err(WasmError::Unsupported(
                            "only function element kind supported",
                        ));
                    }
                    let segment = self.parse_core_funcidx_element_payload(section)?;
                    self.install_core_element_segment(offset, segment)?;
                    self.element_segments[segment_index] = Some(segment);
                }
                _ => {
                    return Err(WasmError::Unsupported(
                        "unsupported core element section mode",
                    ));
                }
            }
        }
        Ok(())
    }

    fn parse_core_funcidx_element_payload(
        &self,
        section: &mut Reader<'a>,
    ) -> Result<CoreWasmElementSegment, WasmError> {
        let func_count = section.read_var_u32()? as usize;
        if func_count > CORE_WASM_TABLE_CAPACITY {
            return Err(WasmError::Unsupported("core element segment too large"));
        }
        let mut functions = [u32::MAX; CORE_WASM_TABLE_CAPACITY];
        for slot in functions.iter_mut().take(func_count) {
            let function_index = section.read_var_u32()?;
            self.core_func_type_index(function_index)?;
            *slot = function_index;
        }
        Ok(CoreWasmElementSegment {
            functions,
            function_count: func_count,
        })
    }

    fn install_core_element_segment(
        &mut self,
        offset: usize,
        segment: CoreWasmElementSegment,
    ) -> Result<(), WasmError> {
        let end = offset
            .checked_add(segment.function_count)
            .ok_or(WasmError::Unsupported("core element table too large"))?;
        if end > CORE_WASM_TABLE_CAPACITY {
            return Err(WasmError::Unsupported("core element table too large"));
        }
        for (dst, function_index) in self
            .table_functions
            .iter_mut()
            .skip(offset)
            .take(segment.function_count)
            .zip(segment.functions.iter().copied())
        {
            *dst = function_index;
        }
        self.table_function_count = self.table_function_count.max(end);
        Ok(())
    }

    fn parse_core_memory_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()?;
        if count != 1 {
            return Err(WasmError::Unsupported(
                "core wasm supports at most one memory",
            ));
        }
        let flags = section.read_u8()?;
        if flags & !0x01 != 0 {
            return Err(WasmError::Unsupported("unsupported core memory flags"));
        }
        let min = section.read_var_u32()?;
        let max = if flags & 0x01 != 0 {
            section.read_var_u32()?
        } else {
            CORE_WASM_MAX_MEMORY_PAGES
        };
        if min > max || max > CORE_WASM_MAX_MEMORY_PAGES {
            return Err(WasmError::Unsupported("core wasm memory exceeds profile"));
        }
        self.memory_min_pages = min;
        self.memory_max_pages = max;
        Ok(())
    }

    fn parse_core_global_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > CORE_WASM_MAX_GLOBALS {
            return Err(WasmError::Unsupported("too many core wasm globals"));
        }
        self.global_count = count;
        for index in 0..count {
            let kind = CoreWasmValueKind::decode(section.read_u8()?)?;
            let mutable = match section.read_u8()? {
                0 => false,
                1 => true,
                _ => return Err(WasmError::Invalid("invalid core global mutability")),
            };
            let initial = Self::parse_core_const_expr(section, kind)?;
            self.globals[index] = Some(CoreWasmGlobal {
                kind,
                mutable,
                initial,
            });
        }
        Ok(())
    }

    fn parse_core_const_expr(
        section: &mut Reader<'a>,
        kind: CoreWasmValueKind,
    ) -> Result<CoreWasmValue, WasmError> {
        let value = match (kind, section.read_u8()?) {
            (CoreWasmValueKind::I32, OPCODE_I32_CONST) => {
                CoreWasmValue::I32(section.read_var_i32()? as u32)
            }
            (CoreWasmValueKind::I64, OPCODE_I64_CONST) => {
                CoreWasmValue::I64(section.read_var_i64()? as u64)
            }
            (CoreWasmValueKind::F32, OPCODE_F32_CONST) => {
                CoreWasmValue::F32(section.read_fixed_u32()?)
            }
            (CoreWasmValueKind::F64, OPCODE_F64_CONST) => {
                CoreWasmValue::F64(section.read_fixed_u64()?)
            }
            (CoreWasmValueKind::FuncRef, OPCODE_REF_NULL) => {
                let heap_type = section.read_u8()?;
                if heap_type != VALTYPE_FUNCREF {
                    return Err(WasmError::Unsupported(
                        "only null funcref globals supported",
                    ));
                }
                CoreWasmValue::FuncRef(u32::MAX)
            }
            _ => return Err(WasmError::Unsupported("unsupported core global init expr")),
        };
        if section.read_u8()? != OPCODE_END {
            return Err(WasmError::Invalid("core global init expr must end"));
        }
        Ok(value)
    }

    fn parse_core_export_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()?;
        for _ in 0..count {
            let name = section.read_name()?;
            let kind = section.read_u8()?;
            let index = section.read_var_u32()?;
            if name == b"__main_void" {
                if kind != EXTERNAL_KIND_FUNC {
                    return Err(WasmError::Invalid("__main_void must export a function"));
                }
                self.core_func_type_index(index)?;
                self.start_function_index = index;
            } else if name == b"_start" && self.start_function_index == u32::MAX {
                if kind != EXTERNAL_KIND_FUNC {
                    return Err(WasmError::Invalid("_start must export a function"));
                }
                self.core_func_type_index(index)?;
                self.start_function_index = index;
            }
        }
        Ok(())
    }

    fn parse_core_code_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count != self.function_count {
            return Err(WasmError::Invalid("core code/function count mismatch"));
        }
        for local_index in 0..count {
            let body_len = section.read_var_u32()? as usize;
            let body = section.read_bytes(body_len)?;
            let mut body_reader = Reader::new(body);
            let mut local_count = 0usize;
            let mut local_kinds = [CoreWasmValueKind::I32; CORE_WASM_LOCAL_CAPACITY];

            let function_type = self.core_func_type(self.function_type_indices[local_index])?;
            for index in 0..function_type.param_count {
                local_kinds[index] = function_type.params[index];
            }
            local_count += function_type.param_count;

            let local_decl_count = body_reader.read_var_u32()?;
            for _ in 0..local_decl_count {
                let count = body_reader.read_var_u32()? as usize;
                let kind = CoreWasmValueKind::decode(body_reader.read_u8()?)?;
                let end = local_count
                    .checked_add(count)
                    .ok_or(WasmError::Unsupported("too many core wasm locals"))?;
                if end > CORE_WASM_LOCAL_CAPACITY {
                    return Err(WasmError::Unsupported("too many core wasm locals"));
                }
                for slot in local_kinds.iter_mut().take(end).skip(local_count) {
                    *slot = kind;
                }
                local_count = end;
            }
            self.code_bodies[local_index] = Some(CoreWasmCodeBody {
                code: &body[body_reader.pos..],
                local_count,
                local_kinds,
            });
        }
        Ok(())
    }

    fn parse_core_data_section(&mut self, section: &mut Reader<'a>) -> Result<(), WasmError> {
        let count = section.read_var_u32()? as usize;
        if count > self.data_segments.len() {
            return Err(WasmError::Unsupported("too many core wasm data segments"));
        }
        for slot in self.data_segments.iter_mut() {
            *slot = None;
        }
        for slot in self.data_segments.iter_mut().take(count) {
            let mode = section.read_var_u32()?;
            let (active, offset) = match mode {
                0 => (true, parse_core_i32_offset_expr(section)?),
                1 => (false, 0),
                2 => {
                    if section.read_var_u32()? != 0 {
                        return Err(WasmError::Invalid("core data memory index must be zero"));
                    }
                    (true, parse_core_i32_offset_expr(section)?)
                }
                _ => return Err(WasmError::Unsupported("unsupported core data segment mode")),
            };
            let bytes_len = section.read_var_u32()? as usize;
            let bytes = section.read_bytes(bytes_len)?;
            *slot = Some(CoreWasmDataSegment {
                active,
                offset: offset as u32,
                bytes,
            });
        }
        Ok(())
    }

    fn core_func_type(&self, type_index: u32) -> Result<CoreWasmFuncType, WasmError> {
        self.types
            .get(type_index as usize)
            .copied()
            .filter(|_| (type_index as usize) < self.type_count)
            .ok_or(WasmError::Invalid("core function type index out of range"))
    }

    fn core_func_type_index(&self, function_index: u32) -> Result<u32, WasmError> {
        if function_index < self.import_count as u32 {
            self.import_type_indices
                .get(function_index as usize)
                .copied()
                .ok_or(WasmError::Invalid("core import index out of range"))
        } else {
            let local_index = function_index
                .checked_sub(self.import_count as u32)
                .ok_or(WasmError::Invalid("core function index underflow"))?
                as usize;
            self.function_type_indices
                .get(local_index)
                .copied()
                .filter(|_| local_index < self.function_count)
                .ok_or(WasmError::Invalid("core function index out of range"))
        }
    }

    fn core_function_body(&self, function_index: u32) -> Result<CoreWasmCodeBody<'a>, WasmError> {
        let local_index = function_index
            .checked_sub(self.import_count as u32)
            .ok_or(WasmError::Invalid("core function body points to import"))?
            as usize;
        self.code_bodies
            .get(local_index)
            .copied()
            .flatten()
            .filter(|_| local_index < self.function_count)
            .ok_or(WasmError::Invalid("core function body out of range"))
    }
}

#[cfg(any(test, feature = "wasm-engine-wasip1-full"))]
fn core_wasm_memory_zeroed() -> Result<CoreWasmMemory, WasmError> {
    let boxed = std::vec![0u8; CORE_WASM_MEMORY_SIZE].into_boxed_slice();
    boxed
        .try_into()
        .map_err(|_| WasmError::Invalid("core wasm heap memory size mismatch"))
}

#[cfg(not(any(test, feature = "wasm-engine-wasip1-full")))]
fn core_wasm_memory_zeroed() -> Result<CoreWasmMemory, WasmError> {
    Ok([0; CORE_WASM_MEMORY_SIZE])
}

#[cfg(any(test, feature = "wasm-engine-wasip1-full"))]
fn write_core_wasm_memory_zeroed(slot: *mut CoreWasmMemory) -> Result<(), WasmError> {
    unsafe {
        slot.write(core_wasm_memory_zeroed()?);
    }
    Ok(())
}

#[cfg(not(any(test, feature = "wasm-engine-wasip1-full")))]
fn write_core_wasm_memory_zeroed(slot: *mut CoreWasmMemory) -> Result<(), WasmError> {
    unsafe {
        slot.cast::<u8>().write_bytes(0, CORE_WASM_MEMORY_SIZE);
    }
    Ok(())
}

#[cfg(any(test, feature = "wasm-engine-wasip1-full"))]
fn core_wasm_frames_empty<'a>() -> Result<CoreWasmFrames<'a>, WasmError> {
    let boxed = std::vec![CoreWasmFrame::empty(); CORE_WASM_CALL_STACK_CAPACITY].into_boxed_slice();
    boxed
        .try_into()
        .map_err(|_| WasmError::Unsupported("failed to allocate core wasm frames"))
}

#[cfg(not(any(test, feature = "wasm-engine-wasip1-full")))]
fn core_wasm_frames_empty<'a>() -> Result<CoreWasmFrames<'a>, WasmError> {
    Ok([CoreWasmFrame::empty(); CORE_WASM_CALL_STACK_CAPACITY])
}

fn parse_core_i32_offset_expr(section: &mut Reader<'_>) -> Result<i32, WasmError> {
    if section.read_u8()? != OPCODE_I32_CONST {
        return Err(WasmError::Invalid("core offset must be i32.const"));
    }
    let offset = section.read_var_i32()?;
    if offset < 0 {
        return Err(WasmError::Invalid("core offset is negative"));
    }
    if section.read_u8()? != OPCODE_END {
        return Err(WasmError::Invalid("core offset expression must end"));
    }
    Ok(offset)
}

fn decode_core_block_type(byte: u8) -> Result<(usize, CoreWasmValueKind), WasmError> {
    if byte == WASM_BLOCKTYPE_EMPTY {
        Ok((0, CoreWasmValueKind::I32))
    } else {
        Ok((1, CoreWasmValueKind::decode(byte)?))
    }
}

impl<'a> CoreWasmInstance<'a> {
    pub fn new(module: &'a [u8]) -> Result<Self, WasmError> {
        CoreWasmModule::parse(module)?.instantiate()
    }

    fn initialize_parsed_in_place<'slot>(
        slot: &'slot mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'slot mut Self, WasmError> {
        // The module field is parsed in place first so embedded profiles do not
        // need a second CoreWasmModule-sized stack or static workspace.
        let ptr = slot.as_mut_ptr();
        let module_ptr = unsafe { core::ptr::addr_of!((*ptr).module) };
        let memory_min_pages = unsafe { (*module_ptr).memory_min_pages };
        if memory_min_pages > CORE_WASM_MAX_MEMORY_PAGES {
            return Err(WasmError::Unsupported("core wasm memory too large"));
        }

        unsafe {
            core::ptr::addr_of_mut!((*ptr).frames).write(core_wasm_frames_empty()?);
            core::ptr::addr_of_mut!((*ptr).frame_len).write(0);
            core::ptr::addr_of_mut!((*ptr).values)
                .write([CoreWasmValue::I32(0); CORE_WASM_VALUE_STACK_CAPACITY]);
            core::ptr::addr_of_mut!((*ptr).value_len).write(0);
            core::ptr::addr_of_mut!((*ptr).globals)
                .write([CoreWasmValue::I32(0); CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_kinds)
                .write([CoreWasmValueKind::I32; CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_mutable).write([false; CORE_WASM_MAX_GLOBALS]);
            core::ptr::addr_of_mut!((*ptr).global_count).write((*module_ptr).global_count);
            write_core_wasm_memory_zeroed(core::ptr::addr_of_mut!((*ptr).memory))?;
            core::ptr::addr_of_mut!((*ptr).memory_pages).write(memory_min_pages);
            core::ptr::addr_of_mut!((*ptr).data_dropped)
                .write([false; TINY_WASIP1_MAX_DATA_SEGMENTS]);
            core::ptr::addr_of_mut!((*ptr).element_dropped)
                .write([false; CORE_WASM_MAX_ELEMENT_SEGMENTS]);
            core::ptr::addr_of_mut!((*ptr).table_functions).write((*module_ptr).table_functions);
            core::ptr::addr_of_mut!((*ptr).table_size).write(
                (*module_ptr)
                    .table_min
                    .max((*module_ptr).table_function_count),
            );
            core::ptr::addr_of_mut!((*ptr).pending).write(None);
            core::ptr::addr_of_mut!((*ptr).done).write(false);
        }

        let instance = unsafe { &mut *ptr };
        for (index, global) in instance
            .module
            .globals
            .iter()
            .copied()
            .flatten()
            .take(instance.module.global_count)
            .enumerate()
        {
            instance.globals[index] = global.initial;
            instance.global_kinds[index] = global.kind;
            instance.global_mutable[index] = global.mutable;
        }
        instance.init_core_data_segments()?;
        instance.push_frame(instance.module.start_function_index)?;
        Ok(instance)
    }

    pub fn resume(&mut self) -> Result<CoreWasmTrap<'a>, WasmError> {
        self.resume_with_fuel(DEFAULT_RESUME_FUEL)
    }

    pub fn resume_with_fuel(&mut self, mut fuel: u32) -> Result<CoreWasmTrap<'a>, WasmError> {
        if self.done {
            return Ok(CoreWasmTrap::Done);
        }
        if self.pending.is_some() {
            return Err(WasmError::PendingHostCall);
        }

        loop {
            if self.frame_len == 0 {
                self.done = true;
                return Ok(CoreWasmTrap::Done);
            }
            if fuel == 0 {
                return Err(WasmError::FuelExhausted);
            }
            fuel -= 1;

            let opcode = self.current_read_u8()?;
            match opcode {
                OPCODE_UNREACHABLE => return Err(WasmError::Trap),
                OPCODE_NOP => {}
                OPCODE_BLOCK | OPCODE_LOOP => {
                    let block_type = self.current_read_u8()?;
                    let (result_count, result_kind) = decode_core_block_type(block_type)?;
                    let frame = self.current_frame_mut()?;
                    let start_pos = frame.pc;
                    let end_pos = find_matching_end(frame.code, start_pos)?;
                    let kind = if opcode == OPCODE_LOOP {
                        ControlKind::Loop
                    } else {
                        ControlKind::Block
                    };
                    let stack_height = self.value_len;
                    self.push_core_control(ControlFrame {
                        kind,
                        start_pos,
                        else_pos: usize::MAX,
                        end_pos,
                        result_count,
                        result_kind,
                        stack_height,
                    })?;
                }
                OPCODE_IF => {
                    let block_type = self.current_read_u8()?;
                    let (result_count, result_kind) = decode_core_block_type(block_type)?;
                    let condition = self.pop_core_i32()?;
                    let frame = self.current_frame_mut()?;
                    let start_pos = frame.pc;
                    let (else_pos, end_pos) = find_matching_else_or_end(frame.code, start_pos)?;
                    let stack_height = self.value_len;
                    if condition != 0 {
                        self.push_core_control(ControlFrame {
                            kind: ControlKind::If,
                            start_pos,
                            else_pos,
                            end_pos,
                            result_count,
                            result_kind,
                            stack_height,
                        })?;
                    } else if else_pos != usize::MAX {
                        self.push_core_control(ControlFrame {
                            kind: ControlKind::If,
                            start_pos: else_pos.saturating_add(1),
                            else_pos,
                            end_pos,
                            result_count,
                            result_kind,
                            stack_height,
                        })?;
                        self.current_frame_mut()?.pc = else_pos.saturating_add(1);
                    } else if result_count == 0 {
                        self.current_frame_mut()?.pc = end_pos.saturating_add(1);
                    } else {
                        return Err(WasmError::Invalid("if result requires else arm"));
                    }
                }
                OPCODE_ELSE => {
                    let control = self.pop_core_control()?;
                    if control.kind != ControlKind::If {
                        return Err(WasmError::Invalid("else without if"));
                    }
                    self.normalize_core_control_result(control)?;
                    self.current_frame_mut()?.pc = control.end_pos.saturating_add(1);
                }
                OPCODE_BR => {
                    let depth = self.current_read_var_u32()? as usize;
                    self.core_branch(depth)?;
                }
                OPCODE_BR_IF => {
                    let depth = self.current_read_var_u32()? as usize;
                    if self.pop_core_i32()? != 0 {
                        self.core_branch(depth)?;
                    }
                }
                OPCODE_BR_TABLE => {
                    let depth = self.decode_core_br_table_depth()?;
                    self.core_branch(depth)?;
                }
                OPCODE_RETURN => self.pop_frame()?,
                OPCODE_CALL => {
                    let function_index = self.current_read_var_u32()?;
                    if function_index < self.module.import_count as u32 {
                        return self.call_core_import(function_index);
                    }
                    self.push_frame(function_index)?;
                }
                OPCODE_CALL_INDIRECT => {
                    let expected_type_index = self.current_read_var_u32()?;
                    self.expect_zero_table_index_var()?;
                    let table_index = self.pop_core_i32()? as usize;
                    let function_index = *self
                        .table_functions
                        .get(table_index)
                        .ok_or(WasmError::Invalid("core call_indirect table out of range"))?;
                    if table_index >= self.table_size || function_index == u32::MAX {
                        return Err(WasmError::Invalid("core call_indirect empty slot"));
                    }
                    if self.module.core_func_type_index(function_index)? != expected_type_index {
                        return Err(WasmError::Invalid("core call_indirect type mismatch"));
                    }
                    if function_index < self.module.import_count as u32 {
                        return self.call_core_import(function_index);
                    }
                    self.push_frame(function_index)?;
                }
                OPCODE_DROP => {
                    let _ = self.pop_core_value()?;
                }
                OPCODE_SELECT => {
                    let condition = self.pop_core_i32()?;
                    let alternate = self.pop_core_value()?;
                    let consequent = self.pop_core_value()?;
                    self.push_core_value(if condition != 0 {
                        consequent
                    } else {
                        alternate
                    })?;
                }
                OPCODE_LOCAL_GET => {
                    let local = self.current_read_var_u32()? as usize;
                    let value = *self
                        .current_frame()?
                        .locals
                        .get(local)
                        .ok_or(WasmError::Invalid("core local.get out of range"))?;
                    if local >= self.current_frame()?.local_count {
                        return Err(WasmError::Invalid("core local.get inactive local"));
                    }
                    self.push_core_value(value)?;
                }
                OPCODE_LOCAL_SET => {
                    let local = self.current_read_var_u32()? as usize;
                    let value = self.pop_core_value()?;
                    self.set_core_local(local, value)?;
                }
                OPCODE_LOCAL_TEE => {
                    let local = self.current_read_var_u32()? as usize;
                    let value = *self
                        .values
                        .get(self.value_len.saturating_sub(1))
                        .ok_or(WasmError::StackUnderflow)?;
                    self.set_core_local(local, value)?;
                }
                OPCODE_GLOBAL_GET => {
                    let global = self.current_read_var_u32()? as usize;
                    if global >= self.global_count {
                        return Err(WasmError::Invalid("core global.get out of range"));
                    }
                    let value = *self
                        .globals
                        .get(global)
                        .ok_or(WasmError::Invalid("core global.get out of range"))?;
                    self.push_core_value(value)?;
                }
                OPCODE_GLOBAL_SET => {
                    let global = self.current_read_var_u32()? as usize;
                    if global >= self.global_count {
                        return Err(WasmError::Invalid("core global.set out of range"));
                    }
                    if !self.global_mutable[global] {
                        return Err(WasmError::Invalid("core global.set immutable global"));
                    }
                    let value = self.pop_core_value()?;
                    if value.kind() != self.global_kinds[global] {
                        return Err(WasmError::Invalid("core global type mismatch"));
                    }
                    self.globals[global] = value;
                }
                OPCODE_TABLE_GET => {
                    self.expect_zero_table_index_var()?;
                    let index = self.pop_core_i32()? as usize;
                    if index >= self.table_size {
                        return Err(WasmError::Invalid("core table.get out of range"));
                    }
                    self.push_core_value(CoreWasmValue::FuncRef(self.table_functions[index]))?;
                }
                OPCODE_TABLE_SET => {
                    self.expect_zero_table_index_var()?;
                    let value = self.pop_core_value()?.as_funcref()?;
                    let index = self.pop_core_i32()? as usize;
                    if index >= self.table_size {
                        return Err(WasmError::Invalid("core table.set out of range"));
                    }
                    if value != u32::MAX {
                        self.module.core_func_type_index(value)?;
                    }
                    self.table_functions[index] = value;
                }
                OPCODE_I32_LOAD => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I32(self.core_read_u32(addr)?))?;
                }
                OPCODE_I64_LOAD => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(self.core_read_u64(addr)?))?;
                }
                OPCODE_F32_LOAD => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::F32(self.core_read_u32(addr)?))?;
                }
                OPCODE_F64_LOAD => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::F64(self.core_read_u64(addr)?))?;
                }
                OPCODE_I32_LOAD8_S => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I32(
                        (self.core_read_u8(addr)? as i8 as i32) as u32,
                    ))?;
                }
                OPCODE_I32_LOAD8_U => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I32(self.core_read_u8(addr)? as u32))?;
                }
                OPCODE_I32_LOAD16_S => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I32(
                        (self.core_read_u16(addr)? as i16 as i32) as u32,
                    ))?;
                }
                OPCODE_I32_LOAD16_U => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I32(self.core_read_u16(addr)? as u32))?;
                }
                OPCODE_I64_LOAD8_S => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(
                        (self.core_read_u8(addr)? as i8 as i64) as u64,
                    ))?;
                }
                OPCODE_I64_LOAD8_U => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(self.core_read_u8(addr)? as u64))?;
                }
                OPCODE_I64_LOAD16_S => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(
                        (self.core_read_u16(addr)? as i16 as i64) as u64,
                    ))?;
                }
                OPCODE_I64_LOAD16_U => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(self.core_read_u16(addr)? as u64))?;
                }
                OPCODE_I64_LOAD32_S => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(
                        (self.core_read_u32(addr)? as i32 as i64) as u64,
                    ))?;
                }
                OPCODE_I64_LOAD32_U => {
                    let addr = self.core_load_effective_addr()?;
                    self.push_core_value(CoreWasmValue::I64(self.core_read_u32(addr)? as u64))?;
                }
                OPCODE_I32_STORE => {
                    let value = self.pop_core_i32()?;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u32(addr, value)?;
                }
                OPCODE_I64_STORE => {
                    let value = self.pop_core_i64()?;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u64(addr, value)?;
                }
                OPCODE_F32_STORE => {
                    let value = self.pop_core_f32_bits()?;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u32(addr, value)?;
                }
                OPCODE_F64_STORE => {
                    let value = self.pop_core_f64_bits()?;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u64(addr, value)?;
                }
                OPCODE_I32_STORE8 => {
                    let value = self.pop_core_i32()? as u8;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u8(addr, value)?;
                }
                OPCODE_I32_STORE16 => {
                    let value = self.pop_core_i32()? as u16;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u16(addr, value)?;
                }
                OPCODE_I64_STORE8 => {
                    let value = self.pop_core_i64()? as u8;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u8(addr, value)?;
                }
                OPCODE_I64_STORE16 => {
                    let value = self.pop_core_i64()? as u16;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u16(addr, value)?;
                }
                OPCODE_I64_STORE32 => {
                    let value = self.pop_core_i64()? as u32;
                    let addr = self.core_store_effective_addr()?;
                    self.core_write_u32(addr, value)?;
                }
                OPCODE_MEMORY_SIZE => {
                    self.expect_zero_memory_index()?;
                    self.push_core_value(CoreWasmValue::I32(self.memory_pages))?;
                }
                OPCODE_MEMORY_GROW => {
                    self.expect_zero_memory_index()?;
                    let requested_pages = self.pop_core_i32()?;
                    let previous_pages = self.memory_pages;
                    let new_pages = previous_pages
                        .checked_add(requested_pages)
                        .and_then(|pages| {
                            if pages <= self.module.memory_max_pages
                                && pages <= CORE_WASM_MAX_MEMORY_PAGES
                            {
                                Some(pages)
                            } else {
                                None
                            }
                        });
                    if let Some(pages) = new_pages {
                        self.memory_pages = pages;
                        self.push_core_value(CoreWasmValue::I32(previous_pages))?;
                    } else {
                        self.push_core_value(CoreWasmValue::I32(u32::MAX))?;
                    }
                    let event = CoreWasmMemoryGrow {
                        previous_pages,
                        requested_pages,
                        new_pages,
                    };
                    self.pending = Some(CoreWasmPending::MemoryGrow(event));
                    return Ok(CoreWasmTrap::MemoryGrow(event));
                }
                OPCODE_I32_CONST => {
                    let value = self.current_read_var_i32()? as u32;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I64_CONST => {
                    let value = self.current_read_var_i64()? as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_F32_CONST => {
                    let value = self.current_read_fixed_u32()?;
                    self.push_core_value(CoreWasmValue::F32(value))?;
                }
                OPCODE_F64_CONST => {
                    let value = self.current_read_fixed_u64()?;
                    self.push_core_value(CoreWasmValue::F64(value))?;
                }
                OPCODE_I32_EQZ => {
                    let value = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32((value == 0) as u32))?;
                }
                OPCODE_I32_EQ => self.core_binary_i32(|a, b| (a == b) as u32)?,
                OPCODE_I32_NE => self.core_binary_i32(|a, b| (a != b) as u32)?,
                OPCODE_I32_LT_S => self.core_binary_i32(|a, b| ((a as i32) < (b as i32)) as u32)?,
                OPCODE_I32_LT_U => self.core_binary_i32(|a, b| (a < b) as u32)?,
                OPCODE_I32_GT_S => self.core_binary_i32(|a, b| ((a as i32) > (b as i32)) as u32)?,
                OPCODE_I32_GT_U => self.core_binary_i32(|a, b| (a > b) as u32)?,
                OPCODE_I32_LE_S => {
                    self.core_binary_i32(|a, b| ((a as i32) <= (b as i32)) as u32)?
                }
                OPCODE_I32_LE_U => self.core_binary_i32(|a, b| (a <= b) as u32)?,
                OPCODE_I32_GE_S => {
                    self.core_binary_i32(|a, b| ((a as i32) >= (b as i32)) as u32)?
                }
                OPCODE_I32_GE_U => self.core_binary_i32(|a, b| (a >= b) as u32)?,
                OPCODE_I64_EQZ => {
                    let value = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I32((value == 0) as u32))?;
                }
                OPCODE_I64_EQ => self.core_binary_i64_cmp(|a, b| a == b)?,
                OPCODE_I64_NE => self.core_binary_i64_cmp(|a, b| a != b)?,
                OPCODE_I64_LT_S => self.core_binary_i64_cmp(|a, b| (a as i64) < (b as i64))?,
                OPCODE_I64_LT_U => self.core_binary_i64_cmp(|a, b| a < b)?,
                OPCODE_I64_GT_S => self.core_binary_i64_cmp(|a, b| (a as i64) > (b as i64))?,
                OPCODE_I64_GT_U => self.core_binary_i64_cmp(|a, b| a > b)?,
                OPCODE_I64_LE_S => self.core_binary_i64_cmp(|a, b| (a as i64) <= (b as i64))?,
                OPCODE_I64_LE_U => self.core_binary_i64_cmp(|a, b| a <= b)?,
                OPCODE_I64_GE_S => self.core_binary_i64_cmp(|a, b| (a as i64) >= (b as i64))?,
                OPCODE_I64_GE_U => self.core_binary_i64_cmp(|a, b| a >= b)?,
                OPCODE_F32_EQ => self.core_binary_f32_cmp(|a, b| a == b)?,
                OPCODE_F32_NE => self.core_binary_f32_cmp(|a, b| a != b)?,
                OPCODE_F32_LT => self.core_binary_f32_cmp(|a, b| a < b)?,
                OPCODE_F32_GT => self.core_binary_f32_cmp(|a, b| a > b)?,
                OPCODE_F32_LE => self.core_binary_f32_cmp(|a, b| a <= b)?,
                OPCODE_F32_GE => self.core_binary_f32_cmp(|a, b| a >= b)?,
                OPCODE_F64_EQ => self.core_binary_f64_cmp(|a, b| a == b)?,
                OPCODE_F64_NE => self.core_binary_f64_cmp(|a, b| a != b)?,
                OPCODE_F64_LT => self.core_binary_f64_cmp(|a, b| a < b)?,
                OPCODE_F64_GT => self.core_binary_f64_cmp(|a, b| a > b)?,
                OPCODE_F64_LE => self.core_binary_f64_cmp(|a, b| a <= b)?,
                OPCODE_F64_GE => self.core_binary_f64_cmp(|a, b| a >= b)?,
                OPCODE_I32_CLZ => {
                    let value = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32(value.leading_zeros()))?;
                }
                OPCODE_I32_CTZ => {
                    let value = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32(value.trailing_zeros()))?;
                }
                OPCODE_I32_POPCNT => {
                    let value = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32(value.count_ones()))?;
                }
                OPCODE_I32_ADD => self.core_binary_i32(u32::wrapping_add)?,
                OPCODE_I32_SUB => self.core_binary_i32(u32::wrapping_sub)?,
                OPCODE_I32_MUL => self.core_binary_i32(u32::wrapping_mul)?,
                OPCODE_I32_DIV_S => {
                    let rhs = self.pop_core_i32()? as i32;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i32()? as i32;
                    if lhs == i32::MIN && rhs == -1 {
                        return Err(WasmError::Trap);
                    }
                    self.push_core_value(CoreWasmValue::I32(lhs.wrapping_div(rhs) as u32))?;
                }
                OPCODE_I32_DIV_U => {
                    let rhs = self.pop_core_i32()?;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32(lhs / rhs))?;
                }
                OPCODE_I32_REM_S => {
                    let rhs = self.pop_core_i32()? as i32;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i32()? as i32;
                    self.push_core_value(CoreWasmValue::I32(lhs.wrapping_rem(rhs) as u32))?;
                }
                OPCODE_I32_REM_U => {
                    let rhs = self.pop_core_i32()?;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::I32(lhs % rhs))?;
                }
                OPCODE_I32_AND => self.core_binary_i32(|a, b| a & b)?,
                OPCODE_I32_OR => self.core_binary_i32(|a, b| a | b)?,
                OPCODE_I32_XOR => self.core_binary_i32(|a, b| a ^ b)?,
                OPCODE_I32_SHL => self.core_binary_i32(|a, b| a.wrapping_shl(b & 31))?,
                OPCODE_I32_SHR_S => self.core_binary_i32(|a, b| ((a as i32) >> (b & 31)) as u32)?,
                OPCODE_I32_SHR_U => self.core_binary_i32(|a, b| a.wrapping_shr(b & 31))?,
                OPCODE_I32_ROTL => self.core_binary_i32(|a, b| a.rotate_left(b & 31))?,
                OPCODE_I32_ROTR => self.core_binary_i32(|a, b| a.rotate_right(b & 31))?,
                OPCODE_I64_CLZ => {
                    let value = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(value.leading_zeros() as u64))?;
                }
                OPCODE_I64_CTZ => {
                    let value = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(value.trailing_zeros() as u64))?;
                }
                OPCODE_I64_POPCNT => {
                    let value = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(value.count_ones() as u64))?;
                }
                OPCODE_I64_ADD => self.core_binary_i64(u64::wrapping_add)?,
                OPCODE_I64_SUB => self.core_binary_i64(u64::wrapping_sub)?,
                OPCODE_I64_MUL => self.core_binary_i64(u64::wrapping_mul)?,
                OPCODE_I64_DIV_S => {
                    let rhs = self.pop_core_i64()? as i64;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i64()? as i64;
                    if lhs == i64::MIN && rhs == -1 {
                        return Err(WasmError::Trap);
                    }
                    self.push_core_value(CoreWasmValue::I64(lhs.wrapping_div(rhs) as u64))?;
                }
                OPCODE_I64_DIV_U => {
                    let rhs = self.pop_core_i64()?;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(lhs / rhs))?;
                }
                OPCODE_I64_REM_S => {
                    let rhs = self.pop_core_i64()? as i64;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i64()? as i64;
                    self.push_core_value(CoreWasmValue::I64(lhs.wrapping_rem(rhs) as u64))?;
                }
                OPCODE_I64_REM_U => {
                    let rhs = self.pop_core_i64()?;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(lhs % rhs))?;
                }
                OPCODE_I64_AND => self.core_binary_i64(|a, b| a & b)?,
                OPCODE_I64_OR => self.core_binary_i64(|a, b| a | b)?,
                OPCODE_I64_XOR => self.core_binary_i64(|a, b| a ^ b)?,
                OPCODE_I64_SHL => {
                    let rhs = self.pop_core_i64()? as u32;
                    let lhs = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(lhs.wrapping_shl(rhs & 63)))?;
                }
                OPCODE_I64_SHR_S => {
                    let rhs = self.pop_core_i64()? as u32;
                    let lhs = self.pop_core_i64()? as i64;
                    self.push_core_value(CoreWasmValue::I64((lhs >> (rhs & 63)) as u64))?;
                }
                OPCODE_I64_SHR_U => {
                    let rhs = self.pop_core_i64()? as u32;
                    let lhs = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::I64(lhs.wrapping_shr(rhs & 63)))?;
                }
                OPCODE_I64_ROTL => {
                    self.core_binary_i64(|a, b| a.rotate_left((b & 63) as u32))?;
                }
                OPCODE_I64_ROTR => {
                    self.core_binary_i64(|a, b| a.rotate_right((b & 63) as u32))?;
                }
                OPCODE_F32_ABS => self.core_unary_f32(wasm_f32_abs)?,
                OPCODE_F32_NEG => self.core_unary_f32(wasm_f32_neg)?,
                OPCODE_F32_CEIL => self.core_unary_f32(wasm_f32_ceil)?,
                OPCODE_F32_FLOOR => self.core_unary_f32(wasm_f32_floor)?,
                OPCODE_F32_TRUNC => self.core_unary_f32(wasm_f32_trunc)?,
                OPCODE_F32_NEAREST => self.core_unary_f32(wasm_f32_nearest)?,
                OPCODE_F32_SQRT => self.core_unary_f32(wasm_f32_sqrt)?,
                OPCODE_F32_ADD => self.core_binary_f32(|a, b| a + b)?,
                OPCODE_F32_SUB => self.core_binary_f32(|a, b| a - b)?,
                OPCODE_F32_MUL => self.core_binary_f32(|a, b| a * b)?,
                OPCODE_F32_DIV => self.core_binary_f32(|a, b| a / b)?,
                OPCODE_F32_MIN => self.core_binary_f32(wasm_f32_min)?,
                OPCODE_F32_MAX => self.core_binary_f32(wasm_f32_max)?,
                OPCODE_F32_COPYSIGN => self.core_binary_f32(wasm_f32_copysign)?,
                OPCODE_F64_ABS => self.core_unary_f64(wasm_f64_abs)?,
                OPCODE_F64_NEG => self.core_unary_f64(wasm_f64_neg)?,
                OPCODE_F64_CEIL => self.core_unary_f64(wasm_f64_ceil)?,
                OPCODE_F64_FLOOR => self.core_unary_f64(wasm_f64_floor)?,
                OPCODE_F64_TRUNC => self.core_unary_f64(wasm_f64_trunc)?,
                OPCODE_F64_NEAREST => self.core_unary_f64(wasm_f64_nearest)?,
                OPCODE_F64_SQRT => self.core_unary_f64(wasm_f64_sqrt)?,
                OPCODE_F64_ADD => self.core_binary_f64(|a, b| a + b)?,
                OPCODE_F64_SUB => self.core_binary_f64(|a, b| a - b)?,
                OPCODE_F64_MUL => self.core_binary_f64(|a, b| a * b)?,
                OPCODE_F64_DIV => self.core_binary_f64(|a, b| a / b)?,
                OPCODE_F64_MIN => self.core_binary_f64(wasm_f64_min)?,
                OPCODE_F64_MAX => self.core_binary_f64(wasm_f64_max)?,
                OPCODE_F64_COPYSIGN => self.core_binary_f64(wasm_f64_copysign)?,
                OPCODE_I32_WRAP_I64 => {
                    let value = self.pop_core_i64()? as u32;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I32_TRUNC_F32_S => {
                    let value = trunc_f32_to_i32_s(self.pop_core_f32()?)?;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I32_TRUNC_F32_U => {
                    let value = trunc_f32_to_i32_u(self.pop_core_f32()?)?;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I32_TRUNC_F64_S => {
                    let value = trunc_f64_to_i32_s(self.pop_core_f64()?)?;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I32_TRUNC_F64_U => {
                    let value = trunc_f64_to_i32_u(self.pop_core_f64()?)?;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I64_EXTEND_I32_S => {
                    let value = self.pop_core_i32()? as i32 as i64 as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_EXTEND_I32_U => {
                    let value = self.pop_core_i32()? as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_TRUNC_F32_S => {
                    let value = trunc_f32_to_i64_s(self.pop_core_f32()?)?;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_TRUNC_F32_U => {
                    let value = trunc_f32_to_i64_u(self.pop_core_f32()?)?;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_TRUNC_F64_S => {
                    let value = trunc_f64_to_i64_s(self.pop_core_f64()?)?;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_TRUNC_F64_U => {
                    let value = trunc_f64_to_i64_u(self.pop_core_f64()?)?;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_F32_CONVERT_I32_S => {
                    let value = self.pop_core_i32()? as i32 as f32;
                    self.push_core_value(CoreWasmValue::F32(value.to_bits()))?;
                }
                OPCODE_F32_CONVERT_I32_U => {
                    let value = self.pop_core_i32()? as f32;
                    self.push_core_value(CoreWasmValue::F32(value.to_bits()))?;
                }
                OPCODE_F32_CONVERT_I64_S => {
                    let value = self.pop_core_i64()? as i64 as f32;
                    self.push_core_value(CoreWasmValue::F32(value.to_bits()))?;
                }
                OPCODE_F32_CONVERT_I64_U => {
                    let value = self.pop_core_i64()? as f32;
                    self.push_core_value(CoreWasmValue::F32(value.to_bits()))?;
                }
                OPCODE_F32_DEMOTE_F64 => {
                    let value = self.pop_core_f64()? as f32;
                    self.push_core_value(CoreWasmValue::F32(value.to_bits()))?;
                }
                OPCODE_F64_CONVERT_I32_S => {
                    let value = self.pop_core_i32()? as i32 as f64;
                    self.push_core_value(CoreWasmValue::F64(value.to_bits()))?;
                }
                OPCODE_F64_CONVERT_I32_U => {
                    let value = self.pop_core_i32()? as f64;
                    self.push_core_value(CoreWasmValue::F64(value.to_bits()))?;
                }
                OPCODE_F64_CONVERT_I64_S => {
                    let value = self.pop_core_i64()? as i64 as f64;
                    self.push_core_value(CoreWasmValue::F64(value.to_bits()))?;
                }
                OPCODE_F64_CONVERT_I64_U => {
                    let value = self.pop_core_i64()? as f64;
                    self.push_core_value(CoreWasmValue::F64(value.to_bits()))?;
                }
                OPCODE_F64_PROMOTE_F32 => {
                    let value = self.pop_core_f32()? as f64;
                    self.push_core_value(CoreWasmValue::F64(value.to_bits()))?;
                }
                OPCODE_I32_REINTERPRET_F32 => {
                    let value = self.pop_core_f32_bits()?;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I64_REINTERPRET_F64 => {
                    let value = self.pop_core_f64_bits()?;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_F32_REINTERPRET_I32 => {
                    let value = self.pop_core_i32()?;
                    self.push_core_value(CoreWasmValue::F32(value))?;
                }
                OPCODE_F64_REINTERPRET_I64 => {
                    let value = self.pop_core_i64()?;
                    self.push_core_value(CoreWasmValue::F64(value))?;
                }
                OPCODE_I32_EXTEND8_S => {
                    let value = self.pop_core_i32()? as i8 as i32 as u32;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I32_EXTEND16_S => {
                    let value = self.pop_core_i32()? as i16 as i32 as u32;
                    self.push_core_value(CoreWasmValue::I32(value))?;
                }
                OPCODE_I64_EXTEND8_S => {
                    let value = self.pop_core_i64()? as i8 as i64 as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_EXTEND16_S => {
                    let value = self.pop_core_i64()? as i16 as i64 as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_I64_EXTEND32_S => {
                    let value = self.pop_core_i64()? as i32 as i64 as u64;
                    self.push_core_value(CoreWasmValue::I64(value))?;
                }
                OPCODE_REF_NULL => {
                    let heap_type = self.current_read_u8()?;
                    if heap_type != VALTYPE_FUNCREF {
                        return Err(WasmError::Unsupported("only null funcref is supported"));
                    }
                    self.push_core_value(CoreWasmValue::FuncRef(u32::MAX))?;
                }
                OPCODE_REF_IS_NULL => {
                    let value = self.pop_core_value()?.as_funcref()?;
                    self.push_core_value(CoreWasmValue::I32((value == u32::MAX) as u32))?;
                }
                OPCODE_REF_FUNC => {
                    let function_index = self.current_read_var_u32()?;
                    self.module.core_func_type_index(function_index)?;
                    self.push_core_value(CoreWasmValue::FuncRef(function_index))?;
                }
                OPCODE_MISC => {
                    let subopcode = self.current_read_var_u32()?;
                    match subopcode {
                        8 => {
                            let data_index = self.current_read_var_u32()? as usize;
                            self.expect_zero_memory_index()?;
                            let len = self.pop_core_i32()? as usize;
                            let src_addr = self.pop_core_i32()? as usize;
                            let dst_addr = self.pop_core_i32()?;
                            let dst = self.core_translate_addr(dst_addr)?;
                            self.core_memory_init(data_index, dst, src_addr, len)?;
                        }
                        9 => {
                            let data_index = self.current_read_var_u32()? as usize;
                            if data_index >= self.data_dropped.len()
                                || self.module.data_segments[data_index].is_none()
                            {
                                return Err(WasmError::Invalid("core data.drop out of range"));
                            }
                            self.data_dropped[data_index] = true;
                        }
                        10 => {
                            self.expect_zero_memory_index()?;
                            self.expect_zero_memory_index()?;
                            let len = self.pop_core_i32()? as usize;
                            let src_addr = self.pop_core_i32()?;
                            let dst_addr = self.pop_core_i32()?;
                            let src = self.core_translate_addr(src_addr)?;
                            let dst = self.core_translate_addr(dst_addr)?;
                            self.core_memory_copy(dst, src, len)?;
                        }
                        11 => {
                            self.expect_zero_memory_index()?;
                            let len = self.pop_core_i32()? as usize;
                            let value = self.pop_core_i32()? as u8;
                            let dst_addr = self.pop_core_i32()?;
                            let dst = self.core_translate_addr(dst_addr)?;
                            self.core_memory_fill(dst, value, len)?;
                        }
                        12 => {
                            let elem_index = self.current_read_var_u32()? as usize;
                            self.expect_zero_table_index_var()?;
                            let len = self.pop_core_i32()? as usize;
                            let src = self.pop_core_i32()? as usize;
                            let dst = self.pop_core_i32()? as usize;
                            self.core_table_init(elem_index, dst, src, len)?;
                        }
                        13 => {
                            let elem_index = self.current_read_var_u32()? as usize;
                            if elem_index >= self.element_dropped.len()
                                || self.module.element_segments[elem_index].is_none()
                            {
                                return Err(WasmError::Invalid("core elem.drop out of range"));
                            }
                            self.element_dropped[elem_index] = true;
                        }
                        14 => {
                            self.expect_zero_table_index_var()?;
                            self.expect_zero_table_index_var()?;
                            let len = self.pop_core_i32()? as usize;
                            let src = self.pop_core_i32()? as usize;
                            let dst = self.pop_core_i32()? as usize;
                            self.core_table_copy(dst, src, len)?;
                        }
                        15 => {
                            self.expect_zero_table_index_var()?;
                            let delta = self.pop_core_i32()? as usize;
                            let init = self.pop_core_value()?.as_funcref()?;
                            if init != u32::MAX {
                                self.module.core_func_type_index(init)?;
                            }
                            let previous = self.table_size;
                            let Some(new_size) = self.table_size.checked_add(delta) else {
                                self.push_core_value(CoreWasmValue::I32(u32::MAX))?;
                                continue;
                            };
                            if new_size > CORE_WASM_TABLE_CAPACITY {
                                self.push_core_value(CoreWasmValue::I32(u32::MAX))?;
                            } else {
                                for slot in self
                                    .table_functions
                                    .iter_mut()
                                    .take(new_size)
                                    .skip(self.table_size)
                                {
                                    *slot = init;
                                }
                                self.table_size = new_size;
                                self.push_core_value(CoreWasmValue::I32(previous as u32))?;
                            }
                        }
                        16 => {
                            self.expect_zero_table_index_var()?;
                            self.push_core_value(CoreWasmValue::I32(self.table_size as u32))?;
                        }
                        17 => {
                            self.expect_zero_table_index_var()?;
                            let len = self.pop_core_i32()? as usize;
                            let value = self.pop_core_value()?.as_funcref()?;
                            let start = self.pop_core_i32()? as usize;
                            if value != u32::MAX {
                                self.module.core_func_type_index(value)?;
                            }
                            self.core_table_fill(start, value, len)?;
                        }
                        _ => return Err(WasmError::UnsupportedOpcode(OPCODE_MISC)),
                    }
                }
                OPCODE_END => {
                    if self.current_frame()?.control_len == 0 {
                        self.pop_frame()?;
                    } else {
                        let control = self.pop_core_control()?;
                        self.normalize_core_control_result(control)?;
                    }
                }
                _ => return Err(WasmError::UnsupportedOpcode(opcode)),
            }
        }
    }

    pub fn complete_host_import(&mut self, results: &[CoreWasmValue]) -> Result<(), WasmError> {
        let pending = self.pending.take().ok_or(WasmError::ReplyWithoutPending)?;
        let CoreWasmPending::HostImport(import) = pending else {
            self.pending = Some(pending);
            return Err(WasmError::UnexpectedReply);
        };
        if results.len() != import.result_count {
            return Err(WasmError::UnexpectedReply);
        }
        let type_index = self
            .module
            .core_func_type_index(import.import.function_index)?;
        let ty = self.module.core_func_type(type_index)?;
        for (index, result) in results.iter().copied().enumerate() {
            if result.kind() != ty.results[index] {
                return Err(WasmError::Invalid("core import result type mismatch"));
            }
            self.push_core_value(result)?;
        }
        Ok(())
    }

    pub fn complete_memory_grow_event(&mut self) -> Result<CoreWasmMemoryGrow, WasmError> {
        let pending = self.pending.take().ok_or(WasmError::ReplyWithoutPending)?;
        let CoreWasmPending::MemoryGrow(event) = pending else {
            self.pending = Some(pending);
            return Err(WasmError::UnexpectedReply);
        };
        Ok(event)
    }

    pub fn memory_pages(&self) -> u32 {
        self.memory_pages
    }

    pub fn read_memory(&self, addr: u32, out: &mut [u8]) -> Result<(), WasmError> {
        let start = self.core_translate_addr(addr)?;
        let end = start.checked_add(out.len()).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self.memory.get(start..end).ok_or(WasmError::Truncated)?;
        out.copy_from_slice(bytes);
        Ok(())
    }

    pub fn write_memory(&mut self, addr: u32, bytes: &[u8]) -> Result<(), WasmError> {
        let start = self.core_translate_addr(addr)?;
        let end = start.checked_add(bytes.len()).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let dst = self
            .memory
            .get_mut(start..end)
            .ok_or(WasmError::Truncated)?;
        dst.copy_from_slice(bytes);
        Ok(())
    }

    pub fn read_memory_u32(&self, addr: u32) -> Result<u32, WasmError> {
        let offset = self.core_translate_addr(addr)?;
        self.core_read_u32(offset)
    }

    pub fn write_memory_u32(&mut self, addr: u32, value: u32) -> Result<(), WasmError> {
        let offset = self.core_translate_addr(addr)?;
        self.core_write_u32(offset, value)
    }

    fn init_core_data_segments(&mut self) -> Result<(), WasmError> {
        let segments = self.module.data_segments;
        for (index, segment) in segments.into_iter().flatten().enumerate() {
            if !segment.active {
                continue;
            }
            let start = self.core_translate_addr(segment.offset)?;
            let end = start
                .checked_add(segment.bytes.len())
                .ok_or(WasmError::Truncated)?;
            if end > self.core_memory_len()? {
                return Err(WasmError::Truncated);
            }
            let dst = self
                .memory
                .get_mut(start..end)
                .ok_or(WasmError::Truncated)?;
            dst.copy_from_slice(segment.bytes);
            self.data_dropped[index] = false;
        }
        Ok(())
    }

    fn push_frame(&mut self, function_index: u32) -> Result<(), WasmError> {
        if function_index < self.module.import_count as u32 {
            return Err(WasmError::Invalid("cannot push import frame"));
        }
        let body = self.module.core_function_body(function_index)?;
        let type_index = self.module.core_func_type_index(function_index)?;
        let ty = self.module.core_func_type(type_index)?;
        let mut args = [CoreWasmValue::I32(0); CORE_WASM_MAX_PARAMS];
        for index in (0..ty.param_count).rev() {
            let value = self.pop_core_value()?;
            if value.kind() != ty.params[index] {
                return Err(WasmError::Invalid("core call argument type mismatch"));
            }
            args[index] = value;
        }
        let slot = self
            .frames
            .get_mut(self.frame_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = CoreWasmFrame::empty();
        slot.code = body.code;
        slot.local_count = body.local_count;
        slot.local_kinds = body.local_kinds;

        for (index, arg) in args.iter().copied().take(ty.param_count).enumerate() {
            slot.locals[index] = arg;
        }
        for index in ty.param_count..body.local_count {
            slot.locals[index] = CoreWasmValue::zero(body.local_kinds[index]);
        }
        self.frame_len += 1;
        Ok(())
    }

    fn pop_frame(&mut self) -> Result<(), WasmError> {
        if self.frame_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.frame_len -= 1;
        if self.frame_len == 0 {
            self.done = true;
        }
        Ok(())
    }

    fn call_core_import(&mut self, function_index: u32) -> Result<CoreWasmTrap<'a>, WasmError> {
        let import = self
            .module
            .imports
            .get(function_index as usize)
            .copied()
            .flatten()
            .ok_or(WasmError::Invalid("missing core import"))?;
        let ty = self
            .module
            .core_func_type(self.module.import_type_indices[function_index as usize])?;
        let mut args = [CoreWasmValue::I32(0); CORE_WASM_MAX_PARAMS];
        for index in (0..ty.param_count).rev() {
            let value = self.pop_core_value()?;
            if value.kind() != ty.params[index] {
                return Err(WasmError::Invalid("core import argument type mismatch"));
            }
            args[index] = value;
        }
        let call = CoreWasmHostImport {
            import,
            args,
            arg_count: ty.param_count,
            result_count: ty.result_count,
        };
        self.pending = Some(CoreWasmPending::HostImport(call));
        Ok(CoreWasmTrap::HostImport(call))
    }

    fn current_frame(&self) -> Result<&CoreWasmFrame<'a>, WasmError> {
        if self.frame_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.frames
            .get(self.frame_len - 1)
            .ok_or(WasmError::StackUnderflow)
    }

    fn current_frame_mut(&mut self) -> Result<&mut CoreWasmFrame<'a>, WasmError> {
        if self.frame_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.frames
            .get_mut(self.frame_len - 1)
            .ok_or(WasmError::StackUnderflow)
    }

    fn current_read_u8(&mut self) -> Result<u8, WasmError> {
        let frame = self.current_frame_mut()?;
        let byte = *frame.code.get(frame.pc).ok_or(WasmError::Truncated)?;
        frame.pc += 1;
        Ok(byte)
    }

    fn current_read_var_u32(&mut self) -> Result<u32, WasmError> {
        let frame = self.current_frame_mut()?;
        let mut reader = Reader {
            bytes: frame.code,
            pos: frame.pc,
        };
        let value = reader.read_var_u32()?;
        frame.pc = reader.pos;
        Ok(value)
    }

    fn current_read_var_i32(&mut self) -> Result<i32, WasmError> {
        let frame = self.current_frame_mut()?;
        let mut reader = Reader {
            bytes: frame.code,
            pos: frame.pc,
        };
        let value = reader.read_var_i32()?;
        frame.pc = reader.pos;
        Ok(value)
    }

    fn current_read_var_i64(&mut self) -> Result<i64, WasmError> {
        let frame = self.current_frame_mut()?;
        let mut reader = Reader {
            bytes: frame.code,
            pos: frame.pc,
        };
        let value = reader.read_var_i64()?;
        frame.pc = reader.pos;
        Ok(value)
    }

    fn current_read_fixed_u32(&mut self) -> Result<u32, WasmError> {
        let frame = self.current_frame_mut()?;
        let mut reader = Reader {
            bytes: frame.code,
            pos: frame.pc,
        };
        let value = reader.read_fixed_u32()?;
        frame.pc = reader.pos;
        Ok(value)
    }

    fn current_read_fixed_u64(&mut self) -> Result<u64, WasmError> {
        let frame = self.current_frame_mut()?;
        let mut reader = Reader {
            bytes: frame.code,
            pos: frame.pc,
        };
        let value = reader.read_fixed_u64()?;
        frame.pc = reader.pos;
        Ok(value)
    }

    fn push_core_value(&mut self, value: CoreWasmValue) -> Result<(), WasmError> {
        let slot = self
            .values
            .get_mut(self.value_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = value;
        self.value_len += 1;
        Ok(())
    }

    fn pop_core_value(&mut self) -> Result<CoreWasmValue, WasmError> {
        if self.value_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.value_len -= 1;
        Ok(self.values[self.value_len])
    }

    fn pop_core_i32(&mut self) -> Result<u32, WasmError> {
        self.pop_core_value()?.as_i32()
    }

    fn pop_core_i64(&mut self) -> Result<u64, WasmError> {
        self.pop_core_value()?.as_i64()
    }

    fn pop_core_f32_bits(&mut self) -> Result<u32, WasmError> {
        self.pop_core_value()?.as_f32_bits()
    }

    fn pop_core_f64_bits(&mut self) -> Result<u64, WasmError> {
        self.pop_core_value()?.as_f64_bits()
    }

    fn pop_core_f32(&mut self) -> Result<f32, WasmError> {
        self.pop_core_value()?.as_f32()
    }

    fn pop_core_f64(&mut self) -> Result<f64, WasmError> {
        self.pop_core_value()?.as_f64()
    }

    fn set_core_local(&mut self, local: usize, value: CoreWasmValue) -> Result<(), WasmError> {
        let frame = self.current_frame_mut()?;
        if local >= frame.local_count {
            return Err(WasmError::Invalid("core local.set inactive local"));
        }
        if value.kind() != frame.local_kinds[local] {
            return Err(WasmError::Invalid("core local type mismatch"));
        }
        frame.locals[local] = value;
        Ok(())
    }

    fn push_core_control(&mut self, control: ControlFrame) -> Result<(), WasmError> {
        let frame = self.current_frame_mut()?;
        let slot = frame
            .controls
            .get_mut(frame.control_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = control;
        frame.control_len += 1;
        Ok(())
    }

    fn pop_core_control(&mut self) -> Result<ControlFrame, WasmError> {
        let frame = self.current_frame_mut()?;
        if frame.control_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        frame.control_len -= 1;
        Ok(frame.controls[frame.control_len])
    }

    fn normalize_core_control_result(&mut self, control: ControlFrame) -> Result<(), WasmError> {
        if control.result_count == 0 {
            self.value_len = self.value_len.min(control.stack_height);
            return Ok(());
        }
        let result = self.pop_core_value()?;
        if result.kind() != control.result_kind {
            return Err(WasmError::Invalid("core block result type mismatch"));
        }
        self.value_len = control.stack_height;
        self.push_core_value(result)
    }

    fn core_branch(&mut self, depth: usize) -> Result<(), WasmError> {
        let frame = self.current_frame()?;
        let Some(target_index) = frame.control_len.checked_sub(depth.saturating_add(1)) else {
            return Err(WasmError::Invalid("core branch target out of range"));
        };
        let control = frame.controls[target_index];
        if control.result_count != 0 {
            let result = self.pop_core_value()?;
            if result.kind() != control.result_kind {
                return Err(WasmError::Invalid("core branch result type mismatch"));
            }
            self.value_len = control.stack_height;
            self.push_core_value(result)?;
        } else {
            self.value_len = self.value_len.min(control.stack_height);
        }
        let frame = self.current_frame_mut()?;
        match control.kind {
            ControlKind::Loop => {
                frame.control_len = target_index + 1;
                frame.pc = control.start_pos;
            }
            ControlKind::Block | ControlKind::If => {
                frame.control_len = target_index;
                frame.pc = control.end_pos.saturating_add(1);
            }
        }
        Ok(())
    }

    fn decode_core_br_table_depth(&mut self) -> Result<usize, WasmError> {
        let count = self.current_read_var_u32()? as usize;
        if count > CORE_WASM_BR_TABLE_CAPACITY {
            return Err(WasmError::Unsupported("core br_table too large"));
        }
        let mut labels = [0usize; CORE_WASM_BR_TABLE_CAPACITY];
        for slot in labels.iter_mut().take(count) {
            *slot = self.current_read_var_u32()? as usize;
        }
        let default = self.current_read_var_u32()? as usize;
        let selected = self.pop_core_i32()? as usize;
        Ok(if selected < count {
            labels[selected]
        } else {
            default
        })
    }

    fn core_load_effective_addr(&mut self) -> Result<usize, WasmError> {
        let _align = self.current_read_var_u32()?;
        let offset = self.current_read_var_u32()?;
        let base = self.pop_core_i32()?;
        self.core_translate_addr(base.checked_add(offset).ok_or(WasmError::Truncated)?)
    }

    fn core_store_effective_addr(&mut self) -> Result<usize, WasmError> {
        self.core_load_effective_addr()
    }

    fn core_translate_addr(&self, addr: u32) -> Result<usize, WasmError> {
        let len = self.core_memory_len()?;
        let offset = addr as usize;
        if offset < len {
            Ok(offset)
        } else {
            Err(WasmError::Truncated)
        }
    }

    fn core_memory_len(&self) -> Result<usize, WasmError> {
        (self.memory_pages as usize)
            .checked_mul(CORE_WASM_PAGE_SIZE)
            .ok_or(WasmError::Truncated)
    }

    fn core_read_u8(&self, offset: usize) -> Result<u8, WasmError> {
        if offset >= self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        self.memory.get(offset).copied().ok_or(WasmError::Truncated)
    }

    fn core_read_u16(&self, offset: usize) -> Result<u16, WasmError> {
        let end = offset.checked_add(2).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self.memory.get(offset..end).ok_or(WasmError::Truncated)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn core_read_u32(&self, offset: usize) -> Result<u32, WasmError> {
        let end = offset.checked_add(4).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self.memory.get(offset..end).ok_or(WasmError::Truncated)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn core_read_u64(&self, offset: usize) -> Result<u64, WasmError> {
        let end = offset.checked_add(8).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self.memory.get(offset..end).ok_or(WasmError::Truncated)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn core_write_u8(&mut self, offset: usize, value: u8) -> Result<(), WasmError> {
        if offset >= self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let byte = self.memory.get_mut(offset).ok_or(WasmError::Truncated)?;
        *byte = value;
        Ok(())
    }

    fn core_write_u16(&mut self, offset: usize, value: u16) -> Result<(), WasmError> {
        let end = offset.checked_add(2).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self
            .memory
            .get_mut(offset..end)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn core_write_u32(&mut self, offset: usize, value: u32) -> Result<(), WasmError> {
        let end = offset.checked_add(4).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self
            .memory
            .get_mut(offset..end)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn core_write_u64(&mut self, offset: usize, value: u64) -> Result<(), WasmError> {
        let end = offset.checked_add(8).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self
            .memory
            .get_mut(offset..end)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn core_memory_copy(&mut self, dst: usize, src: usize, len: usize) -> Result<(), WasmError> {
        let src_end = src.checked_add(len).ok_or(WasmError::Truncated)?;
        let dst_end = dst.checked_add(len).ok_or(WasmError::Truncated)?;
        let memory_len = self.core_memory_len()?;
        if src_end > memory_len || dst_end > memory_len {
            return Err(WasmError::Truncated);
        }
        self.memory.copy_within(src..src_end, dst);
        Ok(())
    }

    fn core_memory_fill(&mut self, dst: usize, value: u8, len: usize) -> Result<(), WasmError> {
        let end = dst.checked_add(len).ok_or(WasmError::Truncated)?;
        if end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let bytes = self.memory.get_mut(dst..end).ok_or(WasmError::Truncated)?;
        bytes.fill(value);
        Ok(())
    }

    fn core_memory_init(
        &mut self,
        data_index: usize,
        dst: usize,
        src: usize,
        len: usize,
    ) -> Result<(), WasmError> {
        if data_index >= self.data_dropped.len() || self.data_dropped[data_index] {
            return Err(WasmError::Invalid("core memory.init data dropped"));
        }
        let segment = self
            .module
            .data_segments
            .get(data_index)
            .copied()
            .flatten()
            .ok_or(WasmError::Invalid("core memory.init data out of range"))?;
        let src_end = src.checked_add(len).ok_or(WasmError::Truncated)?;
        let dst_end = dst.checked_add(len).ok_or(WasmError::Truncated)?;
        if src_end > segment.bytes.len() || dst_end > self.core_memory_len()? {
            return Err(WasmError::Truncated);
        }
        let src_bytes = segment
            .bytes
            .get(src..src_end)
            .ok_or(WasmError::Truncated)?;
        let dst_bytes = self
            .memory
            .get_mut(dst..dst_end)
            .ok_or(WasmError::Truncated)?;
        dst_bytes.copy_from_slice(src_bytes);
        Ok(())
    }

    fn core_table_copy(&mut self, dst: usize, src: usize, len: usize) -> Result<(), WasmError> {
        let src_end = src.checked_add(len).ok_or(WasmError::Truncated)?;
        let dst_end = dst.checked_add(len).ok_or(WasmError::Truncated)?;
        if src_end > self.table_size || dst_end > self.table_size {
            return Err(WasmError::Invalid("core table.copy out of range"));
        }
        self.table_functions.copy_within(src..src_end, dst);
        Ok(())
    }

    fn core_table_init(
        &mut self,
        elem_index: usize,
        dst: usize,
        src: usize,
        len: usize,
    ) -> Result<(), WasmError> {
        if elem_index >= self.element_dropped.len() || self.element_dropped[elem_index] {
            return Err(WasmError::Invalid("core table.init element dropped"));
        }
        let segment = self
            .module
            .element_segments
            .get(elem_index)
            .copied()
            .flatten()
            .ok_or(WasmError::Invalid("core table.init element out of range"))?;
        let src_end = src.checked_add(len).ok_or(WasmError::Truncated)?;
        let dst_end = dst.checked_add(len).ok_or(WasmError::Truncated)?;
        if src_end > segment.function_count || dst_end > self.table_size {
            return Err(WasmError::Invalid("core table.init out of range"));
        }
        for (dst_slot, function_index) in self
            .table_functions
            .iter_mut()
            .skip(dst)
            .take(len)
            .zip(segment.functions.iter().skip(src).copied())
        {
            *dst_slot = function_index;
        }
        Ok(())
    }

    fn core_table_fill(&mut self, start: usize, value: u32, len: usize) -> Result<(), WasmError> {
        let end = start.checked_add(len).ok_or(WasmError::Truncated)?;
        if end > self.table_size {
            return Err(WasmError::Invalid("core table.fill out of range"));
        }
        for slot in self.table_functions.iter_mut().take(end).skip(start) {
            *slot = value;
        }
        Ok(())
    }

    fn expect_zero_memory_index(&mut self) -> Result<(), WasmError> {
        if self.current_read_u8()? != 0 {
            return Err(WasmError::Invalid(
                "core memory instruction index must be zero",
            ));
        }
        Ok(())
    }

    fn expect_zero_table_index_var(&mut self) -> Result<(), WasmError> {
        if self.current_read_var_u32()? != 0 {
            return Err(WasmError::Invalid(
                "core table instruction index must be zero",
            ));
        }
        Ok(())
    }

    fn core_binary_i32(&mut self, op: fn(u32, u32) -> u32) -> Result<(), WasmError> {
        let rhs = self.pop_core_i32()?;
        let lhs = self.pop_core_i32()?;
        self.push_core_value(CoreWasmValue::I32(op(lhs, rhs)))
    }

    fn core_binary_i64(&mut self, op: fn(u64, u64) -> u64) -> Result<(), WasmError> {
        let rhs = self.pop_core_i64()?;
        let lhs = self.pop_core_i64()?;
        self.push_core_value(CoreWasmValue::I64(op(lhs, rhs)))
    }

    fn core_binary_i64_cmp(&mut self, op: fn(u64, u64) -> bool) -> Result<(), WasmError> {
        let rhs = self.pop_core_i64()?;
        let lhs = self.pop_core_i64()?;
        self.push_core_value(CoreWasmValue::I32(op(lhs, rhs) as u32))
    }

    fn core_unary_f32(&mut self, op: fn(f32) -> f32) -> Result<(), WasmError> {
        let value = self.pop_core_f32()?;
        self.push_core_value(CoreWasmValue::F32(op(value).to_bits()))
    }

    fn core_binary_f32(&mut self, op: fn(f32, f32) -> f32) -> Result<(), WasmError> {
        let rhs = self.pop_core_f32()?;
        let lhs = self.pop_core_f32()?;
        self.push_core_value(CoreWasmValue::F32(op(lhs, rhs).to_bits()))
    }

    fn core_binary_f32_cmp(&mut self, op: fn(f32, f32) -> bool) -> Result<(), WasmError> {
        let rhs = self.pop_core_f32()?;
        let lhs = self.pop_core_f32()?;
        self.push_core_value(CoreWasmValue::I32(op(lhs, rhs) as u32))
    }

    fn core_unary_f64(&mut self, op: fn(f64) -> f64) -> Result<(), WasmError> {
        let value = self.pop_core_f64()?;
        self.push_core_value(CoreWasmValue::F64(op(value).to_bits()))
    }

    fn core_binary_f64(&mut self, op: fn(f64, f64) -> f64) -> Result<(), WasmError> {
        let rhs = self.pop_core_f64()?;
        let lhs = self.pop_core_f64()?;
        self.push_core_value(CoreWasmValue::F64(op(lhs, rhs).to_bits()))
    }

    fn core_binary_f64_cmp(&mut self, op: fn(f64, f64) -> bool) -> Result<(), WasmError> {
        let rhs = self.pop_core_f64()?;
        let lhs = self.pop_core_f64()?;
        self.push_core_value(CoreWasmValue::I32(op(lhs, rhs) as u32))
    }
}

fn wasm_f32_min(lhs: f32, rhs: f32) -> f32 {
    if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else {
        lhs.min(rhs)
    }
}

fn wasm_f32_abs(value: f32) -> f32 {
    f32::from_bits(value.to_bits() & 0x7fff_ffff)
}

fn wasm_f32_neg(value: f32) -> f32 {
    f32::from_bits(value.to_bits() ^ 0x8000_0000)
}

fn wasm_f32_trunc(value: f32) -> f32 {
    if !value.is_finite() || wasm_f32_abs(value) >= 9_223_372_036_854_775_808.0 {
        return value;
    }
    (value as i64) as f32
}

fn wasm_f32_floor(value: f32) -> f32 {
    let truncated = wasm_f32_trunc(value);
    if truncated > value {
        truncated - 1.0
    } else {
        truncated
    }
}

fn wasm_f32_ceil(value: f32) -> f32 {
    let truncated = wasm_f32_trunc(value);
    if truncated < value {
        truncated + 1.0
    } else {
        truncated
    }
}

fn wasm_f32_nearest(value: f32) -> f32 {
    if !value.is_finite() {
        return value;
    }
    let floor = wasm_f32_floor(value);
    let ceil = wasm_f32_ceil(value);
    let floor_delta = value - floor;
    let ceil_delta = ceil - value;
    if floor_delta < ceil_delta {
        floor
    } else if ceil_delta < floor_delta {
        ceil
    } else if ((floor as i64) & 1) == 0 {
        floor
    } else {
        ceil
    }
}

fn wasm_f32_sqrt(value: f32) -> f32 {
    if value.is_nan() || value < 0.0 {
        return f32::NAN;
    }
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let mut x = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..8 {
        x = 0.5 * (x + value / x);
    }
    x
}

fn wasm_f32_max(lhs: f32, rhs: f32) -> f32 {
    if lhs.is_nan() || rhs.is_nan() {
        f32::NAN
    } else {
        lhs.max(rhs)
    }
}

fn wasm_f32_copysign(lhs: f32, rhs: f32) -> f32 {
    f32::from_bits((lhs.to_bits() & 0x7fff_ffff) | (rhs.to_bits() & 0x8000_0000))
}

fn wasm_f64_min(lhs: f64, rhs: f64) -> f64 {
    if lhs.is_nan() || rhs.is_nan() {
        f64::NAN
    } else {
        lhs.min(rhs)
    }
}

fn wasm_f64_abs(value: f64) -> f64 {
    f64::from_bits(value.to_bits() & 0x7fff_ffff_ffff_ffff)
}

fn wasm_f64_neg(value: f64) -> f64 {
    f64::from_bits(value.to_bits() ^ 0x8000_0000_0000_0000)
}

fn wasm_f64_trunc(value: f64) -> f64 {
    if !value.is_finite() || wasm_f64_abs(value) >= 9_223_372_036_854_775_808.0 {
        return value;
    }
    (value as i64) as f64
}

fn wasm_f64_floor(value: f64) -> f64 {
    let truncated = wasm_f64_trunc(value);
    if truncated > value {
        truncated - 1.0
    } else {
        truncated
    }
}

fn wasm_f64_ceil(value: f64) -> f64 {
    let truncated = wasm_f64_trunc(value);
    if truncated < value {
        truncated + 1.0
    } else {
        truncated
    }
}

fn wasm_f64_nearest(value: f64) -> f64 {
    if !value.is_finite() {
        return value;
    }
    let floor = wasm_f64_floor(value);
    let ceil = wasm_f64_ceil(value);
    let floor_delta = value - floor;
    let ceil_delta = ceil - value;
    if floor_delta < ceil_delta {
        floor
    } else if ceil_delta < floor_delta {
        ceil
    } else if ((floor as i64) & 1) == 0 {
        floor
    } else {
        ceil
    }
}

fn wasm_f64_sqrt(value: f64) -> f64 {
    if value.is_nan() || value < 0.0 {
        return f64::NAN;
    }
    if value == 0.0 || !value.is_finite() {
        return value;
    }
    let mut x = if value >= 1.0 { value } else { 1.0 };
    for _ in 0..12 {
        x = 0.5 * (x + value / x);
    }
    x
}

fn wasm_f64_max(lhs: f64, rhs: f64) -> f64 {
    if lhs.is_nan() || rhs.is_nan() {
        f64::NAN
    } else {
        lhs.max(rhs)
    }
}

fn wasm_f64_copysign(lhs: f64, rhs: f64) -> f64 {
    f64::from_bits(
        (lhs.to_bits() & 0x7fff_ffff_ffff_ffff) | (rhs.to_bits() & 0x8000_0000_0000_0000),
    )
}

fn trunc_f32_to_i32_s(value: f32) -> Result<u32, WasmError> {
    if !value.is_finite() || value <= i32::MIN as f32 - 1.0 || value >= i32::MAX as f32 + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f32_trunc(value) as i32 as u32)
}

fn trunc_f32_to_i32_u(value: f32) -> Result<u32, WasmError> {
    if !value.is_finite() || value <= -1.0 || value >= (u32::MAX as f32) + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f32_trunc(value) as u32)
}

fn trunc_f64_to_i32_s(value: f64) -> Result<u32, WasmError> {
    if !value.is_finite() || value <= i32::MIN as f64 - 1.0 || value >= i32::MAX as f64 + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f64_trunc(value) as i32 as u32)
}

fn trunc_f64_to_i32_u(value: f64) -> Result<u32, WasmError> {
    if !value.is_finite() || value <= -1.0 || value >= (u32::MAX as f64) + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f64_trunc(value) as u32)
}

fn trunc_f32_to_i64_s(value: f32) -> Result<u64, WasmError> {
    if !value.is_finite() || value <= i64::MIN as f32 - 1.0 || value >= i64::MAX as f32 + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f32_trunc(value) as i64 as u64)
}

fn trunc_f32_to_i64_u(value: f32) -> Result<u64, WasmError> {
    if !value.is_finite() || value <= -1.0 || value >= (u64::MAX as f32) + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f32_trunc(value) as u64)
}

fn trunc_f64_to_i64_s(value: f64) -> Result<u64, WasmError> {
    if !value.is_finite() || value <= i64::MIN as f64 - 1.0 || value >= i64::MAX as f64 + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f64_trunc(value) as i64 as u64)
}

fn trunc_f64_to_i64_u(value: f64) -> Result<u64, WasmError> {
    if !value.is_finite() || value <= -1.0 || value >= (u64::MAX as f64) + 1.0 {
        return Err(WasmError::Trap);
    }
    Ok(wasm_f64_trunc(value) as u64)
}

impl<'a> CoreWasip1Instance<'a> {
    pub fn new(module: &'a [u8], handlers: Wasip1HandlerSet) -> Result<Self, WasmError> {
        let core_module = CoreWasmModule::parse(module)?;
        validate_core_wasip1_imports(&core_module, handlers)?;
        Ok(Self {
            core: core_module.instantiate()?,
            handlers,
            done: false,
        })
    }

    pub fn write_new_in_place<'slot>(
        module: &'a [u8],
        handlers: Wasip1HandlerSet,
        slot: &'slot mut core::mem::MaybeUninit<Self>,
    ) -> Result<&'slot mut Self, WasmError> {
        let ptr = slot.as_mut_ptr();
        unsafe {
            let core_module_slot = core::ptr::addr_of_mut!((*ptr).core.module)
                as *mut core::mem::MaybeUninit<CoreWasmModule<'a>>;
            let core_module = CoreWasmModule::parse_in_place(module, &mut *core_module_slot)?;
            validate_core_wasip1_imports(core_module, handlers)?;
            core::ptr::addr_of_mut!((*ptr).handlers).write(handlers);
            core::ptr::addr_of_mut!((*ptr).done).write(false);
            let core_slot = core::ptr::addr_of_mut!((*ptr).core)
                as *mut core::mem::MaybeUninit<CoreWasmInstance<'a>>;
            CoreWasmInstance::initialize_parsed_in_place(&mut *core_slot)?;
            Ok(&mut *ptr)
        }
    }

    pub fn resume(&mut self) -> Result<CoreWasip1Trap, WasmError> {
        self.resume_with_fuel(DEFAULT_RESUME_FUEL)
    }

    pub fn resume_with_fuel(&mut self, fuel: u32) -> Result<CoreWasip1Trap, WasmError> {
        if self.done {
            return Ok(CoreWasip1Trap::Done);
        }
        match self.core.resume_with_fuel(fuel)? {
            CoreWasmTrap::Done => {
                self.done = true;
                Ok(CoreWasip1Trap::Done)
            }
            CoreWasmTrap::MemoryGrow(event) => Ok(CoreWasip1Trap::MemoryGrow(event)),
            CoreWasmTrap::HostImport(import) => self.translate_wasip1_import(import),
        }
    }

    pub fn complete_host_call(&mut self, errno: u32) -> Result<(), WasmError> {
        self.core.complete_host_import(&[CoreWasmValue::I32(errno)])
    }

    pub fn complete_fd_write(
        &mut self,
        call: TinyWasip1FdWriteCall,
        errno: u32,
    ) -> Result<(), WasmError> {
        let written = if errno == 0 {
            self.fd_write_total_len(call)?
        } else {
            0
        };
        self.core.write_memory_u32(call.nwritten, written)?;
        self.complete_host_call(errno)
    }

    pub fn complete_fd_read(
        &mut self,
        call: CoreWasip1FdReadCall,
        bytes: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            let (dst, max_len) = self.fd_read_iovec(call)?;
            if bytes.len() > max_len as usize {
                return Err(WasmError::Unsupported("fd_read reply exceeds iovec"));
            }
            self.core.write_memory(dst, bytes)?;
            self.core.write_memory_u32(call.nread, bytes.len() as u32)?;
        } else {
            self.core.write_memory_u32(call.nread, 0)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_fdstat_get(
        &mut self,
        call: CoreWasip1FdRequestCall,
        stat: CoreWasip1FdStat,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            let mut bytes = [0u8; WASIP1_FDSTAT_SIZE];
            bytes[WASIP1_FDSTAT_FILETYPE_OFFSET as usize] = stat.filetype();
            bytes[WASIP1_FDSTAT_FLAGS_OFFSET as usize..WASIP1_FDSTAT_FLAGS_OFFSET as usize + 2]
                .copy_from_slice(&stat.flags().to_le_bytes());
            bytes[WASIP1_FDSTAT_RIGHTS_BASE_OFFSET as usize
                ..WASIP1_FDSTAT_RIGHTS_BASE_OFFSET as usize + 8]
                .copy_from_slice(&stat.rights_base().to_le_bytes());
            bytes[WASIP1_FDSTAT_RIGHTS_INHERITING_OFFSET as usize
                ..WASIP1_FDSTAT_RIGHTS_INHERITING_OFFSET as usize + 8]
                .copy_from_slice(&stat.rights_inheriting().to_le_bytes());
            self.core.write_memory(call.out_ptr, &bytes)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_clock_time_get(
        &mut self,
        call: CoreWasip1ClockTimeGetCall,
        nanos: u64,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.core
                .write_memory(call.time_ptr, &nanos.to_le_bytes())?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_clock_res_get(
        &mut self,
        call: CoreWasip1ClockResGetCall,
        resolution_nanos: u64,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.core
                .write_memory(call.resolution_ptr, &resolution_nanos.to_le_bytes())?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_poll_oneoff(
        &mut self,
        call: TinyWasip1PollOneoffCall,
        ready: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.core.write_memory_u32(call.nevents, ready)?;
            if ready > 0 && call.out_ptr != 0 {
                let mut event = [0u8; WASIP1_EVENT_SIZE];
                self.core.read_memory(
                    call.in_ptr
                        .saturating_add(WASIP1_SUBSCRIPTION_USERDATA_OFFSET),
                    &mut event[..8],
                )?;
                let event_type = self.read_memory_u8(
                    call.in_ptr
                        .saturating_add(WASIP1_SUBSCRIPTION_EVENTTYPE_OFFSET),
                )?;
                event[WASIP1_EVENT_ERROR_OFFSET as usize..WASIP1_EVENT_ERROR_OFFSET as usize + 2]
                    .copy_from_slice(&(0u16).to_le_bytes());
                event[WASIP1_EVENT_TYPE_OFFSET as usize] = event_type;
                self.core.write_memory(call.out_ptr, &event)?;
            }
        }
        self.complete_host_call(errno)
    }

    pub fn complete_random_get(
        &mut self,
        call: CoreWasip1RandomGetCall,
        bytes: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            if bytes.len() > call.buf_len as usize {
                return Err(WasmError::Unsupported("random_get reply too large"));
            }
            self.core.write_memory(call.buf, bytes)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_sched_yield(&mut self, errno: u32) -> Result<(), WasmError> {
        self.complete_host_call(errno)
    }

    pub fn complete_proc_raise(&mut self, errno: u32) -> Result<(), WasmError> {
        self.complete_host_call(errno)
    }

    pub fn complete_path_minimal(
        &mut self,
        _call: CoreWasip1PathCall,
        errno: u32,
    ) -> Result<(), WasmError> {
        self.complete_host_call(errno)
    }

    pub fn complete_path_full(
        &mut self,
        _call: CoreWasip1PathCall,
        errno: u32,
    ) -> Result<(), WasmError> {
        self.complete_host_call(errno)
    }

    pub fn path_bytes(&self, call: CoreWasip1PathCall) -> Result<CoreWasip1PathBytes, WasmError> {
        let (ptr, len) = match call.kind {
            CoreWasip1PathKind::PathOpen
            | CoreWasip1PathKind::PathFilestatGet
            | CoreWasip1PathKind::PathFilestatSetTimes
            | CoreWasip1PathKind::PathLink => (call.arg_i32(2)?, call.arg_i32(3)?),
            CoreWasip1PathKind::PathReadlink
            | CoreWasip1PathKind::PathCreateDirectory
            | CoreWasip1PathKind::PathRemoveDirectory
            | CoreWasip1PathKind::PathUnlinkFile
            | CoreWasip1PathKind::PathRename => (call.arg_i32(1)?, call.arg_i32(2)?),
            CoreWasip1PathKind::PathSymlink => (call.arg_i32(0)?, call.arg_i32(1)?),
            _ => return Err(WasmError::Invalid("path import has no path bytes")),
        };
        if len as usize > CORE_WASIP1_PATH_CAPACITY {
            return Err(WasmError::Unsupported("path import path too long"));
        }
        let mut bytes = [0u8; CORE_WASIP1_PATH_CAPACITY];
        self.core.read_memory(ptr, &mut bytes[..len as usize])?;
        Ok(CoreWasip1PathBytes {
            bytes,
            len: len as usize,
        })
    }

    pub fn complete_path_open(
        &mut self,
        call: CoreWasip1PathCall,
        opened_fd: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::PathOpen {
            return Err(WasmError::Invalid("path_open completion kind mismatch"));
        }
        if errno == 0 {
            self.core.write_memory_u32(call.arg_i32(8)?, opened_fd)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_prestat_get(
        &mut self,
        call: CoreWasip1PathCall,
        name_len: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdPrestatGet {
            return Err(WasmError::Invalid(
                "fd_prestat_get completion kind mismatch",
            ));
        }
        if errno == 0 {
            let out = call.arg_i32(1)?;
            let mut bytes = [0u8; WASIP1_PRESTAT_SIZE];
            bytes[WASIP1_PRESTAT_TAG_OFFSET as usize] = WASIP1_PRESTAT_TAG_DIR;
            bytes[WASIP1_PRESTAT_DIR_NAME_LEN_OFFSET as usize
                ..WASIP1_PRESTAT_DIR_NAME_LEN_OFFSET as usize + 4]
                .copy_from_slice(&name_len.to_le_bytes());
            self.core.write_memory(out, &bytes)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_prestat_dir_name(
        &mut self,
        call: CoreWasip1PathCall,
        name: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdPrestatDirName {
            return Err(WasmError::Invalid(
                "fd_prestat_dir_name completion kind mismatch",
            ));
        }
        let ptr = call.arg_i32(1)?;
        let len = call.arg_i32(2)?;
        if errno == 0 {
            if name.len() > len as usize {
                return Err(WasmError::Unsupported("preopen name exceeds guest buffer"));
            }
            self.core.write_memory(ptr, name)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_filestat_get(
        &mut self,
        call: CoreWasip1PathCall,
        stat: CoreWasip1FileStat,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdFilestatGet {
            return Err(WasmError::Invalid(
                "fd_filestat_get completion kind mismatch",
            ));
        }
        self.complete_filestat_at(call.arg_i32(1)?, stat, errno)
    }

    pub fn complete_path_filestat_get(
        &mut self,
        call: CoreWasip1PathCall,
        stat: CoreWasip1FileStat,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::PathFilestatGet {
            return Err(WasmError::Invalid(
                "path_filestat_get completion kind mismatch",
            ));
        }
        self.complete_filestat_at(call.arg_i32(4)?, stat, errno)
    }

    pub fn complete_path_readlink(
        &mut self,
        call: CoreWasip1PathCall,
        target: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::PathReadlink {
            return Err(WasmError::Invalid("path_readlink completion kind mismatch"));
        }
        let buf = call.arg_i32(3)?;
        let buf_len = call.arg_i32(4)?;
        let bufused = call.arg_i32(5)?;
        if errno == 0 {
            if target.len() > buf_len as usize {
                return Err(WasmError::Unsupported("readlink target exceeds buffer"));
            }
            self.core.write_memory(buf, target)?;
            self.core.write_memory_u32(bufused, target.len() as u32)?;
        } else {
            self.core.write_memory_u32(bufused, 0)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_readdir(
        &mut self,
        call: CoreWasip1PathCall,
        bytes: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdReaddir {
            return Err(WasmError::Invalid("fd_readdir completion kind mismatch"));
        }
        let buf = call.arg_i32(1)?;
        let buf_len = call.arg_i32(2)?;
        let bufused = call.arg_i32(4)?;
        if errno == 0 {
            if bytes.len() > buf_len as usize {
                return Err(WasmError::Unsupported("fd_readdir reply exceeds buffer"));
            }
            self.core.write_memory(buf, bytes)?;
            self.core.write_memory_u32(bufused, bytes.len() as u32)?;
        } else {
            self.core.write_memory_u32(bufused, 0)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_pread(
        &mut self,
        call: CoreWasip1PathCall,
        bytes: &[u8],
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdPread {
            return Err(WasmError::Invalid("fd_pread completion kind mismatch"));
        }
        let nread = call.arg_i32(4)?;
        if errno == 0 {
            let (dst, max_len) = self.path_fd_iovec(call, 1, 2)?;
            if bytes.len() > max_len as usize {
                return Err(WasmError::Unsupported("fd_pread reply exceeds iovec"));
            }
            self.core.write_memory(dst, bytes)?;
            self.core.write_memory_u32(nread, bytes.len() as u32)?;
        } else {
            self.core.write_memory_u32(nread, 0)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_pwrite(
        &mut self,
        call: CoreWasip1PathCall,
        written: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdPwrite {
            return Err(WasmError::Invalid("fd_pwrite completion kind mismatch"));
        }
        self.core
            .write_memory_u32(call.arg_i32(4)?, if errno == 0 { written } else { 0 })?;
        self.complete_host_call(errno)
    }

    pub fn complete_fd_seek(
        &mut self,
        call: CoreWasip1PathCall,
        new_offset: u64,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdSeek {
            return Err(WasmError::Invalid("fd_seek completion kind mismatch"));
        }
        if errno == 0 {
            self.core
                .write_memory(call.arg_i32(3)?, &new_offset.to_le_bytes())?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_fd_tell(
        &mut self,
        call: CoreWasip1PathCall,
        offset: u64,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1PathKind::FdTell {
            return Err(WasmError::Invalid("fd_tell completion kind mismatch"));
        }
        if errno == 0 {
            self.core
                .write_memory(call.arg_i32(1)?, &offset.to_le_bytes())?;
        }
        self.complete_host_call(errno)
    }

    fn complete_filestat_at(
        &mut self,
        ptr: u32,
        stat: CoreWasip1FileStat,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            let mut bytes = [0u8; WASIP1_FILESTAT_SIZE];
            bytes[WASIP1_FILESTAT_FILETYPE_OFFSET as usize] = stat.filetype();
            bytes[WASIP1_FILESTAT_SIZE_OFFSET as usize..WASIP1_FILESTAT_SIZE_OFFSET as usize + 8]
                .copy_from_slice(&stat.size().to_le_bytes());
            self.core.write_memory(ptr, &bytes)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_socket(
        &mut self,
        _call: CoreWasip1SocketCall,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            return Err(WasmError::Invalid(
                "socket success requires typed socket completion",
            ));
        }
        self.complete_host_call(errno)
    }

    pub fn complete_sock_accept(
        &mut self,
        call: CoreWasip1SocketCall,
        accepted_fd: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1SocketKind::SockAccept {
            return Err(WasmError::Invalid("socket completion kind mismatch"));
        }
        if errno == 0 {
            self.core.write_memory_u32(call.arg_i32(2)?, accepted_fd)?;
        }
        self.complete_host_call(errno)
    }

    pub fn sock_recv_iovec(&self, call: CoreWasip1SocketCall) -> Result<(u32, u32), WasmError> {
        if call.kind != CoreWasip1SocketKind::SockRecv {
            return Err(WasmError::Invalid("socket recv kind mismatch"));
        }
        self.fd_read_iovec(CoreWasip1FdReadCall {
            fd: call.fd()?,
            iovs: call.arg_i32(1)?,
            iovs_len: call.arg_i32(2)?,
            nread: call.arg_i32(4)?,
        })
    }

    pub fn complete_sock_recv(
        &mut self,
        call: CoreWasip1SocketCall,
        bytes: &[u8],
        ro_flags: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1SocketKind::SockRecv {
            return Err(WasmError::Invalid("socket completion kind mismatch"));
        }
        let nread = call.arg_i32(4)?;
        let ro_flags_ptr = call.arg_i32(5)?;
        if errno == 0 {
            let (dst, max_len) = self.sock_recv_iovec(call)?;
            if bytes.len() > max_len as usize {
                return Err(WasmError::Unsupported("sock_recv reply exceeds iovec"));
            }
            self.core.write_memory(dst, bytes)?;
            self.core.write_memory_u32(nread, bytes.len() as u32)?;
            self.core.write_memory_u32(ro_flags_ptr, ro_flags)?;
        } else {
            self.core.write_memory_u32(nread, 0)?;
            self.core.write_memory_u32(ro_flags_ptr, 0)?;
        }
        self.complete_host_call(errno)
    }

    pub fn sock_send_payload(
        &self,
        call: CoreWasip1SocketCall,
    ) -> Result<TinyWasip1Payload, WasmError> {
        if call.kind != CoreWasip1SocketKind::SockSend {
            return Err(WasmError::Invalid("socket send kind mismatch"));
        }
        self.fd_write_payload(TinyWasip1FdWriteCall {
            fd: call.fd()?,
            iovs: call.arg_i32(1)?,
            iovs_len: call.arg_i32(2)?,
            nwritten: call.arg_i32(4)?,
        })
    }

    pub fn socket_as_engine_req(
        &self,
        call: CoreWasip1SocketCall,
        lease_id: u8,
    ) -> Result<EngineReq, WasmError> {
        match call.kind {
            CoreWasip1SocketKind::SockSend => {
                let payload = self.sock_send_payload(call)?;
                Ok(EngineReq::FdWrite(
                    FdWrite::new_with_lease(call.fd()?, lease_id, payload.as_bytes()).map_err(
                        |_| WasmError::Invalid("sock_send payload does not fit fd_write"),
                    )?,
                ))
            }
            CoreWasip1SocketKind::SockRecv => {
                let (_, max_len) = self.sock_recv_iovec(call)?;
                let request_len = max_len.min(WASIP1_STREAM_CHUNK_CAPACITY as u32);
                if request_len > u8::MAX as u32 {
                    return Err(WasmError::Invalid("sock_recv length does not fit fd_read"));
                }
                Ok(EngineReq::FdRead(
                    FdRead::new_with_lease(call.fd()?, lease_id, request_len as u8)
                        .map_err(|_| WasmError::Invalid("sock_recv length does not fit fd_read"))?,
                ))
            }
            CoreWasip1SocketKind::SockShutdown => {
                Ok(EngineReq::FdClose(FdRequest::new(call.fd()?)))
            }
            CoreWasip1SocketKind::SockAccept => Err(WasmError::Unsupported(
                "sock_accept requires explicit network accept route",
            )),
        }
    }

    pub fn complete_sock_send(
        &mut self,
        call: CoreWasip1SocketCall,
        written: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1SocketKind::SockSend {
            return Err(WasmError::Invalid("socket completion kind mismatch"));
        }
        self.core
            .write_memory_u32(call.arg_i32(4)?, if errno == 0 { written } else { 0 })?;
        self.complete_host_call(errno)
    }

    pub fn complete_sock_shutdown(
        &mut self,
        call: CoreWasip1SocketCall,
        errno: u32,
    ) -> Result<(), WasmError> {
        if call.kind != CoreWasip1SocketKind::SockShutdown {
            return Err(WasmError::Invalid("socket completion kind mismatch"));
        }
        self.complete_host_call(errno)
    }

    pub fn complete_args_sizes_get(
        &mut self,
        call: CoreWasip1ArgsSizesGetCall,
        argc: u32,
        argv_buf_size: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.core.write_memory_u32(call.argc_ptr, argc)?;
            self.core
                .write_memory_u32(call.argv_buf_size_ptr, argv_buf_size)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_environ_sizes_get(
        &mut self,
        call: CoreWasip1EnvironSizesGetCall,
        environ_count: u32,
        environ_buf_size: u32,
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.core
                .write_memory_u32(call.environ_count_ptr, environ_count)?;
            self.core
                .write_memory_u32(call.environ_buf_size_ptr, environ_buf_size)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_args_get(
        &mut self,
        call: CoreWasip1ArgsGetCall,
        args: &[&[u8]],
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.write_cstr_vector(call.argv, call.argv_buf, args)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_environ_get(
        &mut self,
        call: CoreWasip1EnvironGetCall,
        environ: &[(&[u8], &[u8])],
        errno: u32,
    ) -> Result<(), WasmError> {
        if errno == 0 {
            self.write_env_vector(call.environ, call.environ_buf, environ)?;
        }
        self.complete_host_call(errno)
    }

    pub fn complete_memory_grow_event(&mut self) -> Result<CoreWasmMemoryGrow, WasmError> {
        self.core.complete_memory_grow_event()
    }

    pub fn read_memory(&self, addr: u32, out: &mut [u8]) -> Result<(), WasmError> {
        self.core.read_memory(addr, out)
    }

    pub fn write_memory(&mut self, addr: u32, bytes: &[u8]) -> Result<(), WasmError> {
        self.core.write_memory(addr, bytes)
    }

    pub fn read_memory_u32(&self, addr: u32) -> Result<u32, WasmError> {
        self.core.read_memory_u32(addr)
    }

    pub fn fd_write_payload(
        &self,
        call: TinyWasip1FdWriteCall,
    ) -> Result<TinyWasip1Payload, WasmError> {
        let payload_len = self.fd_write_total_len(call)?;
        if payload_len as usize > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(WasmError::Unsupported("tiny fd_write payload too large"));
        }
        let mut bytes = [0u8; WASIP1_STREAM_CHUNK_CAPACITY];
        if call.iovs_len == 0 {
            self.core
                .read_memory(call.iovs, &mut bytes[..payload_len as usize])?;
        } else {
            let mut copied = 0usize;
            for index in 0..call.iovs_len {
                let iov = call
                    .iovs
                    .checked_add(index.saturating_mul(8))
                    .ok_or(WasmError::Truncated)?;
                let ptr = self.core.read_memory_u32(iov)?;
                let len = self.core.read_memory_u32(iov.saturating_add(4))? as usize;
                self.core
                    .read_memory(ptr, &mut bytes[copied..copied + len])?;
                copied += len;
            }
        }
        Ok(TinyWasip1Payload {
            bytes,
            len: payload_len as u8,
        })
    }

    pub fn fd_write_total_len(&self, call: TinyWasip1FdWriteCall) -> Result<u32, WasmError> {
        if call.iovs_len == 0 {
            return Ok(call.nwritten);
        }
        let mut total = 0u32;
        for index in 0..call.iovs_len {
            let iov = call
                .iovs
                .checked_add(index.saturating_mul(8))
                .ok_or(WasmError::Truncated)?;
            let len = self.core.read_memory_u32(iov.saturating_add(4))?;
            total = total.checked_add(len).ok_or(WasmError::Truncated)?;
        }
        Ok(total)
    }

    pub fn poll_oneoff_delay_ticks(
        &self,
        call: TinyWasip1PollOneoffCall,
    ) -> Result<u64, WasmError> {
        if call.nsubscriptions != 1 {
            return Err(WasmError::Unsupported(
                "only one poll_oneoff subscription is supported",
            ));
        }
        if self
            .read_memory_u8(
                call.in_ptr
                    .saturating_add(WASIP1_SUBSCRIPTION_EVENTTYPE_OFFSET),
            )
            .ok()
            == Some(WASIP1_EVENTTYPE_CLOCK)
        {
            if let Ok(timeout_nanos) = self.read_core_u64(
                call.in_ptr
                    .saturating_add(WASIP1_SUBSCRIPTION_CLOCK_TIMEOUT_OFFSET),
            ) {
                if timeout_nanos != 0 {
                    return Ok(timeout_nanos / 1_000_000);
                }
            }
        }
        if call.nevents == 0 {
            let seconds = self.read_core_u64(call.in_ptr)?;
            let nanos = self.core.read_memory_u32(call.in_ptr.saturating_add(8))? as u64;
            return Ok(seconds
                .saturating_mul(1000)
                .saturating_add(nanos / 1_000_000));
        }
        match self.read_core_u64(call.in_ptr) {
            Ok(0) | Err(_) => Err(WasmError::Truncated),
            Ok(delay) => Ok(delay),
        }
    }

    fn read_memory_u8(&self, addr: u32) -> Result<u8, WasmError> {
        let mut byte = [0u8; 1];
        self.core.read_memory(addr, &mut byte)?;
        Ok(byte[0])
    }

    fn translate_wasip1_import(
        &mut self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if call.import.module != WASIP1_IMPORT_MODULE {
            return Err(WasmError::Unsupported("unsupported host import module"));
        }
        match call.import.name {
            WASIP1_IMPORT_FD_WRITE => self.translate_fd_write(call),
            WASIP1_IMPORT_FD_READ => self.translate_fd_read(call),
            WASIP1_IMPORT_FD_FDSTAT_GET => self.translate_fd_fdstat_get(call),
            WASIP1_IMPORT_FD_CLOSE => self.translate_fd_close(call),
            WASIP1_IMPORT_FD_PRESTAT_GET => {
                self.translate_path_minimal(call, CoreWasip1PathKind::FdPrestatGet)
            }
            WASIP1_IMPORT_FD_PRESTAT_DIR_NAME => {
                self.translate_path_minimal(call, CoreWasip1PathKind::FdPrestatDirName)
            }
            WASIP1_IMPORT_FD_FILESTAT_GET => {
                self.translate_path_minimal(call, CoreWasip1PathKind::FdFilestatGet)
            }
            WASIP1_IMPORT_FD_READDIR => {
                self.translate_path_minimal(call, CoreWasip1PathKind::FdReaddir)
            }
            WASIP1_IMPORT_FD_ADVISE => self.translate_path_full(call, CoreWasip1PathKind::FdAdvise),
            WASIP1_IMPORT_FD_ALLOCATE => {
                self.translate_path_full(call, CoreWasip1PathKind::FdAllocate)
            }
            WASIP1_IMPORT_FD_DATASYNC => {
                self.translate_path_full(call, CoreWasip1PathKind::FdDatasync)
            }
            WASIP1_IMPORT_FD_FDSTAT_SET_FLAGS => {
                self.translate_path_full(call, CoreWasip1PathKind::FdFdstatSetFlags)
            }
            WASIP1_IMPORT_FD_FDSTAT_SET_RIGHTS => {
                self.translate_path_full(call, CoreWasip1PathKind::FdFdstatSetRights)
            }
            WASIP1_IMPORT_FD_FILESTAT_SET_SIZE => {
                self.translate_path_full(call, CoreWasip1PathKind::FdFilestatSetSize)
            }
            WASIP1_IMPORT_FD_FILESTAT_SET_TIMES => {
                self.translate_path_full(call, CoreWasip1PathKind::FdFilestatSetTimes)
            }
            WASIP1_IMPORT_FD_PREAD => self.translate_path_full(call, CoreWasip1PathKind::FdPread),
            WASIP1_IMPORT_FD_PWRITE => self.translate_path_full(call, CoreWasip1PathKind::FdPwrite),
            WASIP1_IMPORT_FD_RENUMBER => {
                self.translate_path_full(call, CoreWasip1PathKind::FdRenumber)
            }
            WASIP1_IMPORT_FD_SEEK => self.translate_path_full(call, CoreWasip1PathKind::FdSeek),
            WASIP1_IMPORT_FD_SYNC => self.translate_path_full(call, CoreWasip1PathKind::FdSync),
            WASIP1_IMPORT_FD_TELL => self.translate_path_full(call, CoreWasip1PathKind::FdTell),
            WASIP1_IMPORT_CLOCK_RES_GET => self.translate_clock_res_get(call),
            WASIP1_IMPORT_CLOCK_TIME_GET => self.translate_clock_time_get(call),
            WASIP1_IMPORT_POLL_ONEOFF => self.translate_poll_oneoff(call),
            WASIP1_IMPORT_SCHED_YIELD => self.translate_sched_yield(call),
            WASIP1_IMPORT_PATH_OPEN => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathOpen)
            }
            WASIP1_IMPORT_PATH_FILESTAT_GET => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathFilestatGet)
            }
            WASIP1_IMPORT_PATH_READLINK => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathReadlink)
            }
            WASIP1_IMPORT_PATH_CREATE_DIRECTORY => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathCreateDirectory)
            }
            WASIP1_IMPORT_PATH_REMOVE_DIRECTORY => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathRemoveDirectory)
            }
            WASIP1_IMPORT_PATH_UNLINK_FILE => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathUnlinkFile)
            }
            WASIP1_IMPORT_PATH_RENAME => {
                self.translate_path_minimal(call, CoreWasip1PathKind::PathRename)
            }
            WASIP1_IMPORT_PATH_FILESTAT_SET_TIMES => {
                self.translate_path_full(call, CoreWasip1PathKind::PathFilestatSetTimes)
            }
            WASIP1_IMPORT_PATH_LINK => self.translate_path_full(call, CoreWasip1PathKind::PathLink),
            WASIP1_IMPORT_PATH_SYMLINK => {
                self.translate_path_full(call, CoreWasip1PathKind::PathSymlink)
            }
            WASIP1_IMPORT_RANDOM_GET => self.translate_random_get(call),
            WASIP1_IMPORT_ARGS_SIZES_GET => self.translate_args_sizes_get(call),
            WASIP1_IMPORT_ARGS_GET => self.translate_args_get(call),
            WASIP1_IMPORT_ENVIRON_SIZES_GET => self.translate_environ_sizes_get(call),
            WASIP1_IMPORT_ENVIRON_GET => self.translate_environ_get(call),
            WASIP1_IMPORT_PROC_EXIT => self.translate_proc_exit(call),
            WASIP1_IMPORT_PROC_RAISE => self.translate_proc_raise(call),
            WASIP1_IMPORT_SOCK_ACCEPT => {
                self.translate_socket(call, CoreWasip1SocketKind::SockAccept)
            }
            WASIP1_IMPORT_SOCK_RECV => self.translate_socket(call, CoreWasip1SocketKind::SockRecv),
            WASIP1_IMPORT_SOCK_SEND => self.translate_socket(call, CoreWasip1SocketKind::SockSend),
            WASIP1_IMPORT_SOCK_SHUTDOWN => {
                self.translate_socket(call, CoreWasip1SocketKind::SockShutdown)
            }
            _ => Err(WasmError::Unsupported("unsupported wasip1 import")),
        }
    }

    fn translate_fd_write(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::FdWrite) {
            return Err(WasmError::Unsupported(
                "wasip1 fd_write disabled by feature profile",
            ));
        }
        if call.arg_count != 4 || call.result_count != 1 {
            return Err(WasmError::Invalid("fd_write import signature mismatch"));
        }
        let args = call.args();
        let fd = args[0].as_i32()?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(CoreWasip1Trap::FdWrite(TinyWasip1FdWriteCall {
            fd: fd as u8,
            iovs: args[1].as_i32()?,
            iovs_len: args[2].as_i32()?,
            nwritten: args[3].as_i32()?,
        }))
    }

    fn translate_fd_read(&self, call: CoreWasmHostImport<'a>) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::FdRead) {
            return Err(WasmError::Unsupported(
                "wasip1 fd_read disabled by feature profile",
            ));
        }
        if call.arg_count != 4 || call.result_count != 1 {
            return Err(WasmError::Invalid("fd_read import signature mismatch"));
        }
        let args = call.args();
        let fd = args[0].as_i32()?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(CoreWasip1Trap::FdRead(CoreWasip1FdReadCall {
            fd: fd as u8,
            iovs: args[1].as_i32()?,
            iovs_len: args[2].as_i32()?,
            nread: args[3].as_i32()?,
        }))
    }

    fn translate_fd_fdstat_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::FdFdstatGet) {
            return Err(WasmError::Unsupported(
                "wasip1 fd_fdstat_get disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid(
                "fd_fdstat_get import signature mismatch",
            ));
        }
        let args = call.args();
        let fd = args[0].as_i32()?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(CoreWasip1Trap::FdFdstatGet(CoreWasip1FdRequestCall {
            fd: fd as u8,
            out_ptr: args[1].as_i32()?,
        }))
    }

    fn translate_fd_close(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::FdClose) {
            return Err(WasmError::Unsupported(
                "wasip1 fd_close disabled by feature profile",
            ));
        }
        if call.arg_count != 1 || call.result_count != 1 {
            return Err(WasmError::Invalid("fd_close import signature mismatch"));
        }
        let fd = call.args()[0].as_i32()?;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        Ok(CoreWasip1Trap::FdClose(CoreWasip1FdRequestCall {
            fd: fd as u8,
            out_ptr: 0,
        }))
    }

    fn translate_clock_time_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ClockTimeGet) {
            return Err(WasmError::Unsupported(
                "wasip1 clock_time_get disabled by feature profile",
            ));
        }
        if call.arg_count != 3 || call.result_count != 1 {
            return Err(WasmError::Invalid(
                "clock_time_get import signature mismatch",
            ));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::ClockTimeGet(CoreWasip1ClockTimeGetCall {
            clock_id: args[0].as_i32()?,
            precision: args[1].as_i64()?,
            time_ptr: args[2].as_i32()?,
        }))
    }

    fn translate_clock_res_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ClockResGet) {
            return Err(WasmError::Unsupported(
                "wasip1 clock_res_get disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid(
                "clock_res_get import signature mismatch",
            ));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::ClockResGet(CoreWasip1ClockResGetCall {
            clock_id: args[0].as_i32()?,
            resolution_ptr: args[1].as_i32()?,
        }))
    }

    fn translate_poll_oneoff(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::PollOneoff) {
            return Err(WasmError::Unsupported(
                "wasip1 poll_oneoff disabled by feature profile",
            ));
        }
        if call.arg_count != 4 || call.result_count != 1 {
            return Err(WasmError::Invalid("poll_oneoff import signature mismatch"));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::PollOneoff(TinyWasip1PollOneoffCall {
            in_ptr: args[0].as_i32()?,
            out_ptr: args[1].as_i32()?,
            nsubscriptions: args[2].as_i32()?,
            nevents: args[3].as_i32()?,
        }))
    }

    fn translate_random_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::RandomGet) {
            return Err(WasmError::Unsupported(
                "wasip1 random_get disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid("random_get import signature mismatch"));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::RandomGet(CoreWasip1RandomGetCall {
            buf: args[0].as_i32()?,
            buf_len: args[1].as_i32()?,
        }))
    }

    fn translate_sched_yield(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::SchedYield) {
            return Err(WasmError::Unsupported(
                "wasip1 sched_yield disabled by feature profile",
            ));
        }
        if call.arg_count != 0 || call.result_count != 1 {
            return Err(WasmError::Invalid("sched_yield import signature mismatch"));
        }
        Ok(CoreWasip1Trap::SchedYield)
    }

    fn translate_path_minimal(
        &self,
        call: CoreWasmHostImport<'a>,
        kind: CoreWasip1PathKind,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::PathMinimal) {
            return Err(WasmError::Unsupported(
                "wasip1 path-minimal disabled by feature profile",
            ));
        }
        Ok(CoreWasip1Trap::PathMinimal(CoreWasip1PathCall {
            kind,
            args: call.args,
            arg_count: call.arg_count,
        }))
    }

    fn translate_path_full(
        &self,
        call: CoreWasmHostImport<'a>,
        kind: CoreWasip1PathKind,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::PathFull) {
            return Err(WasmError::Unsupported(
                "wasip1 path-full disabled by feature profile",
            ));
        }
        Ok(CoreWasip1Trap::PathFull(CoreWasip1PathCall {
            kind,
            args: call.args,
            arg_count: call.arg_count,
        }))
    }

    fn translate_socket(
        &self,
        call: CoreWasmHostImport<'a>,
        kind: CoreWasip1SocketKind,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::NetworkObject) {
            return Err(WasmError::Unsupported(
                "wasip1 NetworkObject imports disabled by feature profile",
            ));
        }
        Ok(CoreWasip1Trap::Socket(CoreWasip1SocketCall {
            kind,
            args: call.args,
            arg_count: call.arg_count,
        }))
    }

    fn translate_args_sizes_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ArgsEnv) {
            return Err(WasmError::Unsupported(
                "wasip1 args/env disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid(
                "args_sizes_get import signature mismatch",
            ));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::ArgsSizesGet(CoreWasip1ArgsSizesGetCall {
            argc_ptr: args[0].as_i32()?,
            argv_buf_size_ptr: args[1].as_i32()?,
        }))
    }

    fn translate_args_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ArgsEnv) {
            return Err(WasmError::Unsupported(
                "wasip1 args/env disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid("args_get import signature mismatch"));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::ArgsGet(CoreWasip1ArgsGetCall {
            argv: args[0].as_i32()?,
            argv_buf: args[1].as_i32()?,
        }))
    }

    fn translate_environ_sizes_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ArgsEnv) {
            return Err(WasmError::Unsupported(
                "wasip1 args/env disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid(
                "environ_sizes_get import signature mismatch",
            ));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::EnvironSizesGet(
            CoreWasip1EnvironSizesGetCall {
                environ_count_ptr: args[0].as_i32()?,
                environ_buf_size_ptr: args[1].as_i32()?,
            },
        ))
    }

    fn translate_environ_get(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ArgsEnv) {
            return Err(WasmError::Unsupported(
                "wasip1 args/env disabled by feature profile",
            ));
        }
        if call.arg_count != 2 || call.result_count != 1 {
            return Err(WasmError::Invalid("environ_get import signature mismatch"));
        }
        let args = call.args();
        Ok(CoreWasip1Trap::EnvironGet(CoreWasip1EnvironGetCall {
            environ: args[0].as_i32()?,
            environ_buf: args[1].as_i32()?,
        }))
    }

    fn translate_proc_exit(
        &mut self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ProcExit) {
            return Err(WasmError::Unsupported(
                "wasip1 proc_exit disabled by feature profile",
            ));
        }
        if call.arg_count != 1 || call.result_count != 0 {
            return Err(WasmError::Invalid("proc_exit import signature mismatch"));
        }
        let code = call.args()[0].as_i32()?;
        self.core.complete_host_import(&[])?;
        self.done = true;
        Ok(CoreWasip1Trap::ProcExit(code))
    }

    fn translate_proc_raise(
        &self,
        call: CoreWasmHostImport<'a>,
    ) -> Result<CoreWasip1Trap, WasmError> {
        if !self.handlers.supports(Wasip1Syscall::ProcRaise) {
            return Err(WasmError::Unsupported(
                "wasip1 proc_raise disabled by feature profile",
            ));
        }
        if call.arg_count != 1 || call.result_count != 1 {
            return Err(WasmError::Invalid("proc_raise import signature mismatch"));
        }
        Ok(CoreWasip1Trap::ProcRaise(call.args()[0].as_i32()?))
    }

    fn read_core_u64(&self, addr: u32) -> Result<u64, WasmError> {
        let mut bytes = [0u8; 8];
        self.core.read_memory(addr, &mut bytes)?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn fd_read_iovec(&self, call: CoreWasip1FdReadCall) -> Result<(u32, u32), WasmError> {
        if call.iovs_len != 1 {
            return Err(WasmError::Unsupported(
                "only one fd_read iovec is supported",
            ));
        }
        Ok((
            self.core.read_memory_u32(call.iovs)?,
            self.core.read_memory_u32(call.iovs.saturating_add(4))?,
        ))
    }

    fn path_fd_iovec(
        &self,
        call: CoreWasip1PathCall,
        iovs_arg: usize,
        iovs_len_arg: usize,
    ) -> Result<(u32, u32), WasmError> {
        if call.arg_i32(iovs_len_arg)? != 1 {
            return Err(WasmError::Unsupported(
                "only one path fd iovec is supported",
            ));
        }
        let iovs = call.arg_i32(iovs_arg)?;
        Ok((
            self.core.read_memory_u32(iovs)?,
            self.core.read_memory_u32(iovs.saturating_add(4))?,
        ))
    }

    fn write_cstr_vector(
        &mut self,
        ptrs: u32,
        mut buf: u32,
        items: &[&[u8]],
    ) -> Result<(), WasmError> {
        for (index, item) in items.iter().enumerate() {
            self.core
                .write_memory_u32(ptrs.saturating_add((index as u32).saturating_mul(4)), buf)?;
            self.core.write_memory(buf, item)?;
            buf = buf
                .checked_add(item.len() as u32)
                .ok_or(WasmError::Truncated)?;
            self.core.write_memory(buf, &[0])?;
            buf = buf.checked_add(1).ok_or(WasmError::Truncated)?;
        }
        Ok(())
    }

    fn write_env_vector(
        &mut self,
        ptrs: u32,
        mut buf: u32,
        items: &[(&[u8], &[u8])],
    ) -> Result<(), WasmError> {
        for (index, (key, value)) in items.iter().enumerate() {
            self.core
                .write_memory_u32(ptrs.saturating_add((index as u32).saturating_mul(4)), buf)?;
            self.core.write_memory(buf, key)?;
            buf = buf
                .checked_add(key.len() as u32)
                .ok_or(WasmError::Truncated)?;
            self.core.write_memory(buf, b"=")?;
            buf = buf.checked_add(1).ok_or(WasmError::Truncated)?;
            self.core.write_memory(buf, value)?;
            buf = buf
                .checked_add(value.len() as u32)
                .ok_or(WasmError::Truncated)?;
            self.core.write_memory(buf, &[0])?;
            buf = buf.checked_add(1).ok_or(WasmError::Truncated)?;
        }
        Ok(())
    }
}

impl<'a> TinyWasmModule<'a> {
    pub fn parse(bytes: &'a [u8]) -> Result<Self, WasmError> {
        let mut reader = Reader::new(bytes);
        if reader.read_bytes(4)? != WASM_MAGIC {
            return Err(WasmError::Invalid("invalid wasm magic"));
        }
        if reader.read_bytes(4)? != WASM_VERSION {
            return Err(WasmError::Invalid("unsupported wasm version"));
        }

        let mut func_types = [FuncSig::Unsupported; MAX_FUNC_TYPES];
        let mut type_count = 0usize;
        let mut saw_imports = false;
        let mut saw_functions = false;
        let mut saw_exports = false;
        let mut import_count = 0u32;
        let mut optional_import = None;
        let mut local_start_sig = FuncSig::Unsupported;
        let mut start_export_index = None;
        let mut start_body = None;

        while !reader.is_empty() {
            let section_id = reader.read_u8()?;
            let section_len = reader.read_var_u32()? as usize;
            let section_bytes = reader.read_bytes(section_len)?;
            let mut section = Reader::new(section_bytes);
            match section_id {
                SECTION_TYPE => {
                    type_count = parse_type_section(&mut section, &mut func_types)?;
                }
                SECTION_IMPORT => {
                    let imports = parse_import_section(&mut section, &func_types[..type_count])?;
                    import_count = imports.0;
                    optional_import = imports.1;
                    saw_imports = true;
                }
                SECTION_FUNCTION => {
                    local_start_sig =
                        parse_function_section(&mut section, &func_types[..type_count])?;
                    saw_functions = true;
                }
                SECTION_EXPORT => {
                    start_export_index = Some(parse_export_section(&mut section)?);
                    saw_exports = true;
                }
                SECTION_CODE => {
                    start_body = Some(parse_code_section(&mut section)?);
                }
                _ => {}
            }
            if !section.is_empty() {
                return Err(WasmError::Invalid("section has trailing bytes"));
            }
        }

        if !saw_imports {
            return Err(WasmError::Invalid("missing import section"));
        }
        if !saw_functions {
            return Err(WasmError::Invalid("missing function section"));
        }
        if !saw_exports {
            return Err(WasmError::Invalid("missing export section"));
        }
        if !(MIN_IMPORT_COUNT..=MAX_IMPORT_COUNT).contains(&import_count) {
            return Err(WasmError::Unsupported(
                "demo expects two or three function imports",
            ));
        }
        if local_start_sig != FuncSig::UnitToUnit {
            return Err(WasmError::Unsupported("start function must be () -> ()"));
        }
        if start_export_index != Some(import_count) {
            return Err(WasmError::Invalid(
                "expected _start to resolve to the first local function",
            ));
        }

        Ok(Self {
            start_body: start_body.ok_or(WasmError::Invalid("missing code section"))?,
            optional_import,
        })
    }

    pub fn instantiate(self) -> TinyWasmInstance<'a> {
        TinyWasmInstance {
            code: self.start_body,
            pc: 0,
            stack: [0; STACK_CAPACITY],
            stack_len: 0,
            pending: None,
            done: false,
            optional_import: self.optional_import,
        }
    }
}

impl<'a> TinyWasmInstance<'a> {
    pub fn new(module: &'a [u8]) -> Result<Self, WasmError> {
        Ok(TinyWasmModule::parse(module)?.instantiate())
    }

    pub fn resume(&mut self) -> Result<GuestTrap, WasmError> {
        self.resume_with_fuel(DEFAULT_RESUME_FUEL)
    }

    pub fn resume_with_fuel(&mut self, mut fuel: u32) -> Result<GuestTrap, WasmError> {
        if self.done {
            return Ok(GuestTrap::Done);
        }
        if self.pending.is_some() {
            return Err(WasmError::PendingHostCall);
        }

        let mut reader = Reader {
            bytes: self.code,
            pos: self.pc,
        };

        loop {
            if fuel == 0 {
                self.pc = reader.pos;
                return Err(WasmError::FuelExhausted);
            }
            fuel -= 1;
            let opcode = reader.read_u8()?;
            match opcode {
                OPCODE_UNREACHABLE => {
                    self.pc = reader.pos;
                    return Err(WasmError::Trap);
                }
                OPCODE_NOP => {
                    self.pc = reader.pos;
                }
                OPCODE_I32_CONST => {
                    let value = reader.read_var_i32()?;
                    self.push(value)?;
                    self.pc = reader.pos;
                }
                OPCODE_CALL => {
                    let function_index = reader.read_var_u32()?;
                    self.pc = reader.pos;
                    return if function_index == LOG_IMPORT_INDEX {
                        {
                            let value = self.pop()? as u32;
                            self.pending = Some(PendingHostCall::LogU32(value));
                            Ok(GuestTrap::HostCall(EngineReq::LogU32(value)))
                        }
                    } else if function_index == YIELD_IMPORT_INDEX {
                        {
                            self.pending = Some(PendingHostCall::Yield);
                            Ok(GuestTrap::HostCall(EngineReq::Yield))
                        }
                    } else if function_index == SLEEP_IMPORT_INDEX {
                        match self.optional_import {
                            Some(OptionalImport::SleepUntil) => {
                                let tick = self.pop()? as u64;
                                self.pending = Some(PendingHostCall::SleepUntil(tick));
                                Ok(GuestTrap::HostCall(EngineReq::TimerSleepUntil(
                                    TimerSleepUntil::new(tick),
                                )))
                            }
                            Some(OptionalImport::GpioSet) => {
                                let set = GpioSet::from_wasm_value(self.pop()? as u32);
                                self.pending = Some(PendingHostCall::GpioSet(set));
                                Ok(GuestTrap::HostCall(EngineReq::GpioSet(set)))
                            }
                            None => Err(WasmError::Unsupported("call target not supported")),
                        }
                    } else {
                        Err(WasmError::Unsupported("call target not supported"))
                    };
                }
                OPCODE_END => {
                    self.pc = reader.pos;
                    if self.stack_len != 0 {
                        return Err(WasmError::Invalid(
                            "start function must end with an empty stack",
                        ));
                    }
                    self.done = true;
                    return Ok(GuestTrap::Done);
                }
                _ => return Err(WasmError::Unsupported("opcode not supported")),
            }
        }
    }

    pub fn resume_with_budget(&mut self, run: BudgetRun) -> Result<BudgetedGuestTrap, WasmError> {
        match self.resume_with_fuel(run.fuel()) {
            Ok(trap) => Ok(BudgetedGuestTrap::Guest(trap)),
            Err(WasmError::FuelExhausted) => Ok(BudgetedGuestTrap::BudgetExpired(
                BudgetExpired::new(run.run_id(), run.generation()),
            )),
            Err(error) => Err(error),
        }
    }

    pub fn complete_host_call(&mut self, reply: EngineRet) -> Result<(), WasmError> {
        let pending = self.pending.take().ok_or(WasmError::ReplyWithoutPending)?;
        match (pending, reply) {
            (PendingHostCall::LogU32(expected), EngineRet::Logged(actual))
                if expected == actual =>
            {
                Ok(())
            }
            (PendingHostCall::Yield, EngineRet::Yielded) => Ok(()),
            (PendingHostCall::SleepUntil(expected), EngineRet::TimerSleepDone(done))
                if expected == done.tick() =>
            {
                Ok(())
            }
            (PendingHostCall::GpioSet(expected), EngineRet::GpioSetDone(actual))
                if expected == actual =>
            {
                Ok(())
            }
            _ => Err(WasmError::UnexpectedReply),
        }
    }

    fn push(&mut self, value: i32) -> Result<(), WasmError> {
        let slot = self
            .stack
            .get_mut(self.stack_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = value;
        self.stack_len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, WasmError> {
        if self.stack_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.stack_len -= 1;
        Ok(self.stack[self.stack_len])
    }
}

#[cfg(test)]
impl<'a> TinyWasip1TrafficLightInstance<'a> {
    #[doc(hidden)]
    pub fn from_wasip1_app(module: &'a [u8]) -> Result<Self, WasmError> {
        Self::from_wasip1_app_unchecked_handlers(module)
    }

    #[doc(hidden)]
    pub fn from_wasip1_app_with_handlers(
        module: &'a [u8],
        handlers: Wasip1HandlerSet,
    ) -> Result<Self, WasmError> {
        validate_tiny_wasip1_imports_against_handlers(module, handlers)?;
        Self::from_wasip1_app_unchecked_handlers(module)
    }

    fn from_wasip1_app_unchecked_handlers(module: &'a [u8]) -> Result<Self, WasmError> {
        let parsed = parse_tiny_wasip1_traffic_module(module)?;
        let has_data_segments = parsed.data_segments.iter().any(Option::is_some);
        match parsed.traffic_steps {
            Some(_) => {}
            None if has_data_segments => {}
            None => {
                return Err(WasmError::Invalid(
                    "wasip1 app has no executable syscall stream for this engine",
                ));
            }
        };
        let mut instance = Self {
            program: parsed.program,
            pc: 0,
            stack: [0; STACK_CAPACITY],
            stack_len: 0,
            values: [Wasip1Value::I32(0); TINY_WASIP1_VALUE_STACK_CAPACITY],
            value_len: 0,
            locals: [Wasip1Value::I32(0); TINY_WASIP1_LOCAL_CAPACITY],
            local_kinds: [Wasip1ValueKind::I32; TINY_WASIP1_LOCAL_CAPACITY],
            local_count: 0,
            controls: [ControlFrame {
                kind: ControlKind::Block,
                start_pos: 0,
                else_pos: usize::MAX,
                end_pos: 0,
                result_count: 0,
                result_kind: CoreWasmValueKind::I32,
                stack_height: 0,
            }; TINY_WASIP1_CONTROL_STACK_CAPACITY],
            control_len: 0,
            global0: parsed.memory_base.saturating_add(RUST_WASIP1_STACK_SLOP),
            pending: None,
            done: false,
            memory: [0; TINY_WASIP1_MEMORY_SIZE],
            memory_base: parsed.memory_base,
            fd_write_count: 0,
            poll_count: 0,
        };
        instance.init_data_segments(parsed.data_segments)?;
        instance.init_rust_std_main_state();
        Ok(instance)
    }

    #[doc(hidden)]
    pub fn resume(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        if self.done {
            return Ok(TinyWasip1Trap::Done);
        }
        if self.pending.is_some() {
            return Err(WasmError::PendingHostCall);
        }
        if matches!(self.program, TinyWasip1Program::RustStdMain { .. }) {
            return self.resume_rust_std_main();
        }

        let mut reader = Reader {
            bytes: self.program_code(),
            pos: self.pc,
        };
        loop {
            let opcode = reader.read_u8()?;
            match opcode {
                OPCODE_I32_CONST => {
                    let value = reader.read_var_i32()?;
                    self.push(value)?;
                    self.pc = reader.pos;
                }
                OPCODE_DROP => {
                    let _ = self.pop()?;
                    self.pc = reader.pos;
                }
                OPCODE_CALL => {
                    let function_index = reader.read_var_u32()?;
                    self.pc = reader.pos;
                    return self.call_tiny_wasip1(function_index);
                }
                OPCODE_END => {
                    self.pc = reader.pos;
                    if self.stack_len != 0 {
                        return Err(WasmError::Invalid(
                            "start function must end with an empty stack",
                        ));
                    }
                    self.done = true;
                    return Ok(TinyWasip1Trap::Done);
                }
                _ => return Err(WasmError::Unsupported("opcode not supported")),
            }
        }
    }

    #[doc(hidden)]
    pub fn complete_host_call(&mut self, errno: u32) -> Result<(), WasmError> {
        let pending = self.pending.take().ok_or(WasmError::ReplyWithoutPending)?;
        match pending {
            PendingWasip1Call::FdWrite(call, return_mode) => {
                let written = self.fd_write_payload(call)?.len as u32;
                if let Ok(nwritten) = self.translate_addr(call.nwritten) {
                    self.write_u32(nwritten, written)?;
                }
                self.fd_write_count = self.fd_write_count.saturating_add(1);
                match return_mode {
                    PendingWasip1Return::ErrnoI32Stack => self.push(errno as i32)?,
                    PendingWasip1Return::ErrnoValueStack => {
                        self.push_value(Wasip1Value::I32(errno))?
                    }
                    PendingWasip1Return::FdWriteLen => {
                        self.push_value(Wasip1Value::I32(written))?
                    }
                }
            }
            PendingWasip1Call::PollOneoff(call, return_mode) => {
                let delay = self.poll_oneoff_delay_ticks(call)?;
                if let Ok(nevents) = self.translate_addr(call.nevents) {
                    self.write_u32(nevents, call.nsubscriptions)?;
                }
                if let Ok(out) = self.translate_addr(call.out_ptr) {
                    self.write_u64(out, delay)?;
                }
                self.poll_count = self.poll_count.saturating_add(1);
                match return_mode {
                    PendingWasip1Return::ErrnoI32Stack => self.push(errno as i32)?,
                    PendingWasip1Return::ErrnoValueStack => {
                        self.push_value(Wasip1Value::I32(errno))?
                    }
                    PendingWasip1Return::FdWriteLen => {
                        return Err(WasmError::UnexpectedReply);
                    }
                }
            }
        }
        Ok(())
    }

    #[doc(hidden)]
    pub fn fd_write_payload(
        &self,
        call: TinyWasip1FdWriteCall,
    ) -> Result<TinyWasip1Payload, WasmError> {
        let (payload_ptr, payload_len) = if call.iovs_len == 0 {
            (call.iovs as usize, call.nwritten as usize)
        } else if call.iovs_len == 1 {
            let Ok(iovs) = self.translate_addr(call.iovs) else {
                return Err(WasmError::Truncated);
            };
            (
                match self.read_u32(iovs) {
                    Ok(ptr) => ptr as usize,
                    Err(_) => return Err(WasmError::Truncated),
                },
                match self.read_u32(iovs + 4) {
                    Ok(len) => len as usize,
                    Err(_) => return Err(WasmError::Truncated),
                },
            )
        } else {
            return Err(WasmError::Unsupported("only one iovec is supported"));
        };
        if payload_len > WASIP1_STREAM_CHUNK_CAPACITY {
            return Err(WasmError::Unsupported("tiny fd_write payload too large"));
        }
        let Ok(payload_offset) = self.translate_addr(payload_ptr as u32) else {
            return Err(WasmError::Truncated);
        };
        let Some(payload) = self
            .memory
            .get(payload_offset..payload_offset + payload_len)
        else {
            return Err(WasmError::Truncated);
        };
        let mut bytes = [0u8; WASIP1_STREAM_CHUNK_CAPACITY];
        bytes[..payload_len].copy_from_slice(payload);
        Ok(TinyWasip1Payload {
            bytes,
            len: payload_len as u8,
        })
    }

    #[doc(hidden)]
    pub fn poll_oneoff_delay_ticks(
        &self,
        call: TinyWasip1PollOneoffCall,
    ) -> Result<u64, WasmError> {
        if call.nsubscriptions != 1 {
            return Err(WasmError::Unsupported(
                "only one poll_oneoff subscription is supported",
            ));
        }
        let Ok(offset) = self.translate_addr(call.in_ptr) else {
            return Err(WasmError::Truncated);
        };
        if call.nevents == 0 {
            let seconds = self.read_u64(offset)?;
            let nanos = self.read_u32(offset + 8)? as u64;
            return Ok(seconds
                .saturating_mul(1000)
                .saturating_add(nanos / 1_000_000));
        }
        match self.read_u64(offset) {
            Ok(0) | Err(_) => Err(WasmError::Truncated),
            Ok(delay) => Ok(delay),
        }
    }

    fn program_code(&self) -> &'a [u8] {
        match self.program {
            TinyWasip1Program::Direct { code } | TinyWasip1Program::Rust { code, .. } => code,
            TinyWasip1Program::RustStdMain { body, .. } => body.code,
        }
    }

    fn call_tiny_wasip1(&mut self, function_index: u32) -> Result<TinyWasip1Trap, WasmError> {
        match self.program {
            TinyWasip1Program::Direct { .. } => self.call_direct_wasip1(function_index),
            TinyWasip1Program::Rust { fd_write_index, .. } if function_index == fd_write_index => {
                self.call_import_fd_write()
            }
            TinyWasip1Program::Rust {
                poll_oneoff_index, ..
            } if function_index == poll_oneoff_index => self.call_import_poll_oneoff(),
            TinyWasip1Program::Rust { .. } => {
                Err(WasmError::Unsupported("unsupported rust wasi import call"))
            }
            TinyWasip1Program::RustStdMain { .. } => Err(WasmError::Unsupported(
                "unexpected rust std main import call",
            )),
        }
    }

    fn call_direct_wasip1(&mut self, function_index: u32) -> Result<TinyWasip1Trap, WasmError> {
        match function_index {
            WASIP1_FD_WRITE_IMPORT_INDEX => self.call_import_fd_write(),
            WASIP1_POLL_ONEOFF_IMPORT_INDEX => self.call_import_poll_oneoff(),
            _ => Err(WasmError::Unsupported("unsupported wasi import call")),
        }
    }

    fn call_import_fd_write(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        let nwritten = self.pop()? as u32;
        let iovs_len = self.pop()? as u32;
        let iovs = self.pop()? as u32;
        let fd = self.pop()? as u32;
        if fd > u8::MAX as u32 {
            return Err(WasmError::Invalid("fd does not fit u8"));
        }
        let call = TinyWasip1FdWriteCall {
            fd: fd as u8,
            iovs,
            iovs_len,
            nwritten,
        };
        self.pending = Some(PendingWasip1Call::FdWrite(
            call,
            PendingWasip1Return::ErrnoI32Stack,
        ));
        Ok(TinyWasip1Trap::FdWrite(call))
    }

    fn call_import_poll_oneoff(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        let nevents = self.pop()? as u32;
        let nsubscriptions = self.pop()? as u32;
        let out_ptr = self.pop()? as u32;
        let in_ptr = self.pop()? as u32;
        let call = TinyWasip1PollOneoffCall {
            in_ptr,
            out_ptr,
            nsubscriptions,
            nevents,
        };
        self.pending = Some(PendingWasip1Call::PollOneoff(
            call,
            PendingWasip1Return::ErrnoI32Stack,
        ));
        Ok(TinyWasip1Trap::PollOneoff(call))
    }

    fn init_data_segments(
        &mut self,
        segments: [Option<TinyWasip1DataSegment<'a>>; TINY_WASIP1_MAX_DATA_SEGMENTS],
    ) -> Result<(), WasmError> {
        for segment in segments.into_iter().flatten() {
            let segment_start = segment.offset;
            let segment_end = segment_start
                .checked_add(segment.bytes.len() as u32)
                .ok_or(WasmError::Truncated)?;
            let window_start = self.memory_base;
            let window_end = window_start
                .checked_add(TINY_WASIP1_MEMORY_SIZE as u32)
                .ok_or(WasmError::Truncated)?;
            let copy_start = segment_start.max(window_start);
            let copy_end = segment_end.min(window_end);
            if copy_start >= copy_end {
                continue;
            };
            let src_offset = copy_start
                .checked_sub(segment_start)
                .ok_or(WasmError::Truncated)?;
            let dst_offset = copy_start
                .checked_sub(window_start)
                .ok_or(WasmError::Truncated)?;
            let len = copy_end
                .checked_sub(copy_start)
                .ok_or(WasmError::Truncated)? as usize;
            let src_offset = src_offset as usize;
            let dst_offset = dst_offset as usize;
            let dst = self
                .memory
                .get_mut(dst_offset..dst_offset + len)
                .ok_or(WasmError::Truncated)?;
            let src = segment
                .bytes
                .get(src_offset..src_offset + len)
                .ok_or(WasmError::Truncated)?;
            dst.copy_from_slice(src);
        }
        Ok(())
    }

    fn push(&mut self, value: i32) -> Result<(), WasmError> {
        let slot = self
            .stack
            .get_mut(self.stack_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = value;
        self.stack_len += 1;
        Ok(())
    }

    fn pop(&mut self) -> Result<i32, WasmError> {
        if self.stack_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.stack_len -= 1;
        Ok(self.stack[self.stack_len])
    }

    fn read_u32(&self, offset: usize) -> Result<u32, WasmError> {
        let bytes = self
            .memory
            .get(offset..offset + 4)
            .ok_or(WasmError::Truncated)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_u8_at(&self, offset: usize) -> Result<u8, WasmError> {
        self.memory.get(offset).copied().ok_or(WasmError::Truncated)
    }

    fn read_u64(&self, offset: usize) -> Result<u64, WasmError> {
        let bytes = self
            .memory
            .get(offset..offset + 8)
            .ok_or(WasmError::Truncated)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn write_u32(&mut self, offset: usize, value: u32) -> Result<(), WasmError> {
        let bytes = self
            .memory
            .get_mut(offset..offset + 4)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn write_u8(&mut self, offset: usize, value: u8) -> Result<(), WasmError> {
        let byte = self.memory.get_mut(offset).ok_or(WasmError::Truncated)?;
        *byte = value;
        Ok(())
    }

    fn write_u16(&mut self, offset: usize, value: u16) -> Result<(), WasmError> {
        let bytes = self
            .memory
            .get_mut(offset..offset + 2)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn write_u64(&mut self, offset: usize, value: u64) -> Result<(), WasmError> {
        let bytes = self
            .memory
            .get_mut(offset..offset + 8)
            .ok_or(WasmError::Truncated)?;
        bytes.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn translate_addr(&self, addr: u32) -> Result<usize, WasmError> {
        addr.checked_sub(self.memory_base)
            .map(|offset| offset as usize)
            .filter(|offset| *offset < TINY_WASIP1_MEMORY_SIZE)
            .ok_or(WasmError::Truncated)
    }

    fn init_rust_std_main_state(&mut self) {
        if let TinyWasip1Program::RustStdMain { body, .. } = self.program {
            self.local_count = body.local_count;
            self.local_kinds = body.local_kinds;
            let mut index = 0usize;
            while index < body.local_count {
                self.locals[index] = Wasip1Value::zero(body.local_kinds[index]);
                index += 1;
            }
        }
    }

    fn resume_rust_std_main(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        let (code, write_index, clock_nanosleep_index) = match self.program {
            TinyWasip1Program::RustStdMain {
                body,
                write_index,
                clock_nanosleep_index,
            } => (body.code, write_index, clock_nanosleep_index),
            _ => return Err(WasmError::Invalid("not a rust std main program")),
        };
        let mut reader = Reader {
            bytes: code,
            pos: self.pc,
        };
        loop {
            let opcode = reader.read_u8()?;
            match opcode {
                OPCODE_UNREACHABLE => {
                    self.pc = reader.pos;
                    return Err(WasmError::Trap);
                }
                OPCODE_NOP => {
                    self.pc = reader.pos;
                }
                OPCODE_BLOCK | OPCODE_LOOP => {
                    let block_type = reader.read_u8()?;
                    if block_type != WASM_BLOCKTYPE_EMPTY {
                        return Err(WasmError::Unsupported("block results are not supported"));
                    }
                    let start_pos = reader.pos;
                    let end_pos = find_matching_end(code, start_pos)?;
                    let kind = if opcode == OPCODE_LOOP {
                        ControlKind::Loop
                    } else {
                        ControlKind::Block
                    };
                    self.push_control(ControlFrame {
                        kind,
                        start_pos,
                        else_pos: usize::MAX,
                        end_pos,
                        result_count: 0,
                        result_kind: CoreWasmValueKind::I32,
                        stack_height: self.value_len,
                    })?;
                    self.pc = reader.pos;
                }
                OPCODE_BR => {
                    let depth = reader.read_var_u32()? as usize;
                    self.branch(depth, &mut reader)?;
                }
                OPCODE_BR_IF => {
                    let depth = reader.read_var_u32()? as usize;
                    if self.pop_value_i32()? != 0 {
                        self.branch(depth, &mut reader)?;
                    } else {
                        self.pc = reader.pos;
                    }
                }
                OPCODE_BR_TABLE => {
                    let depth = self.decode_br_table_depth(&mut reader)?;
                    self.branch(depth, &mut reader)?;
                }
                OPCODE_RETURN => {
                    self.pc = reader.pos;
                    self.done = true;
                    return Ok(TinyWasip1Trap::Done);
                }
                OPCODE_CALL => {
                    let function_index = reader.read_var_u32()?;
                    self.pc = reader.pos;
                    if function_index == write_index {
                        return self.call_rust_std_write();
                    }
                    if function_index == clock_nanosleep_index {
                        return self.call_rust_std_clock_nanosleep();
                    }
                    return Err(WasmError::Unsupported("unsupported rust std function call"));
                }
                OPCODE_DROP => {
                    let _ = self.pop_value()?;
                    self.pc = reader.pos;
                }
                OPCODE_SELECT => {
                    let condition = self.pop_value_i32()?;
                    let alternate = self.pop_value()?;
                    let consequent = self.pop_value()?;
                    self.push_value(if condition != 0 {
                        consequent
                    } else {
                        alternate
                    })?;
                    self.pc = reader.pos;
                }
                OPCODE_LOCAL_GET => {
                    let local = reader.read_var_u32()? as usize;
                    self.push_value(
                        *self
                            .locals
                            .get(local)
                            .ok_or(WasmError::Invalid("local.get index out of range"))?,
                    )?;
                    self.pc = reader.pos;
                }
                OPCODE_LOCAL_SET => {
                    let local = reader.read_var_u32()? as usize;
                    let value = self.pop_value()?;
                    self.set_local(local, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_LOCAL_TEE => {
                    let local = reader.read_var_u32()? as usize;
                    let value = *self
                        .values
                        .get(self.value_len.saturating_sub(1))
                        .ok_or(WasmError::StackUnderflow)?;
                    self.set_local(local, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_GLOBAL_GET => {
                    let global = reader.read_var_u32()?;
                    match global {
                        0 => self.push_value(Wasip1Value::I32(self.global0))?,
                        _ => return Err(WasmError::Unsupported("unsupported global.get")),
                    }
                    self.pc = reader.pos;
                }
                OPCODE_GLOBAL_SET => {
                    let global = reader.read_var_u32()?;
                    let value = self.pop_value_i32()?;
                    match global {
                        0 => self.global0 = value,
                        _ => return Err(WasmError::Unsupported("unsupported global.set")),
                    }
                    self.pc = reader.pos;
                }
                OPCODE_I32_LOAD => {
                    let addr = self.load_effective_addr(&mut reader)?;
                    self.push_value(Wasip1Value::I32(self.read_u32(addr)?))?;
                    self.pc = reader.pos;
                }
                OPCODE_I64_LOAD => {
                    let addr = self.load_effective_addr(&mut reader)?;
                    self.push_value(Wasip1Value::I64(self.read_u64(addr)?))?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_LOAD8_U => {
                    let addr = self.load_effective_addr(&mut reader)?;
                    self.push_value(Wasip1Value::I32(self.read_u8_at(addr)? as u32))?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_STORE => {
                    let value = self.pop_value_i32()?;
                    let addr = self.store_effective_addr(&mut reader)?;
                    self.write_u32(addr, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_I64_STORE => {
                    let value = self.pop_value_i64()?;
                    let addr = self.store_effective_addr(&mut reader)?;
                    self.write_u64(addr, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_STORE8 => {
                    let value = self.pop_value_i32()? as u8;
                    let addr = self.store_effective_addr(&mut reader)?;
                    self.write_u8(addr, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_STORE16 => {
                    let value = self.pop_value_i32()? as u16;
                    let addr = self.store_effective_addr(&mut reader)?;
                    self.write_u16(addr, value)?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_CONST => {
                    self.push_value(Wasip1Value::I32(reader.read_var_i32()? as u32))?;
                    self.pc = reader.pos;
                }
                OPCODE_I64_CONST => {
                    self.push_value(Wasip1Value::I64(reader.read_var_i64()? as u64))?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_EQZ => {
                    let value = self.pop_value_i32()?;
                    self.push_value(Wasip1Value::I32((value == 0) as u32))?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_EQ => self.binary_i32(reader.pos, |a, b| (a == b) as u32)?,
                OPCODE_I32_NE => self.binary_i32(reader.pos, |a, b| (a != b) as u32)?,
                OPCODE_I32_GT_S => {
                    self.binary_i32(reader.pos, |a, b| ((a as i32) > (b as i32)) as u32)?
                }
                OPCODE_I32_GT_U => self.binary_i32(reader.pos, |a, b| (a > b) as u32)?,
                OPCODE_I64_EQ => self.binary_i64_cmp(reader.pos, |a, b| a == b)?,
                OPCODE_I64_NE => self.binary_i64_cmp(reader.pos, |a, b| a != b)?,
                OPCODE_I64_LT_U => self.binary_i64_cmp(reader.pos, |a, b| a < b)?,
                OPCODE_I32_ADD => self.binary_i32(reader.pos, u32::wrapping_add)?,
                OPCODE_I32_SUB => self.binary_i32(reader.pos, u32::wrapping_sub)?,
                OPCODE_I32_MUL => self.binary_i32(reader.pos, u32::wrapping_mul)?,
                OPCODE_I32_DIV_U => {
                    let rhs = self.pop_value_i32()?;
                    if rhs == 0 {
                        return Err(WasmError::Trap);
                    }
                    let lhs = self.pop_value_i32()?;
                    self.push_value(Wasip1Value::I32(lhs / rhs))?;
                    self.pc = reader.pos;
                }
                OPCODE_I32_AND => self.binary_i32(reader.pos, |a, b| a & b)?,
                OPCODE_I64_ADD => self.binary_i64(reader.pos, u64::wrapping_add)?,
                OPCODE_I64_SUB => self.binary_i64(reader.pos, u64::wrapping_sub)?,
                OPCODE_I64_AND => self.binary_i64(reader.pos, |a, b| a & b)?,
                OPCODE_I64_OR => self.binary_i64(reader.pos, |a, b| a | b)?,
                OPCODE_I64_SHL => {
                    let rhs = self.pop_value_i64()? as u32;
                    let lhs = self.pop_value_i64()?;
                    self.push_value(Wasip1Value::I64(lhs.wrapping_shl(rhs & 63)))?;
                    self.pc = reader.pos;
                }
                OPCODE_I64_EXTEND_I32_U => {
                    let value = self.pop_value_i32()? as u64;
                    self.push_value(Wasip1Value::I64(value))?;
                    self.pc = reader.pos;
                }
                OPCODE_END => {
                    if self.control_len == 0 {
                        self.pc = reader.pos;
                        self.done = true;
                        return Ok(TinyWasip1Trap::Done);
                    }
                    self.control_len -= 1;
                    self.pc = reader.pos;
                }
                _ => return Err(WasmError::Unsupported("opcode not supported")),
            }
        }
    }

    fn push_value(&mut self, value: Wasip1Value) -> Result<(), WasmError> {
        let slot = self
            .values
            .get_mut(self.value_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = value;
        self.value_len += 1;
        Ok(())
    }

    fn pop_value(&mut self) -> Result<Wasip1Value, WasmError> {
        if self.value_len == 0 {
            return Err(WasmError::StackUnderflow);
        }
        self.value_len -= 1;
        Ok(self.values[self.value_len])
    }

    fn pop_value_i32(&mut self) -> Result<u32, WasmError> {
        self.pop_value()?.as_i32()
    }

    fn pop_value_i64(&mut self) -> Result<u64, WasmError> {
        self.pop_value()?.as_i64()
    }

    fn set_local(&mut self, local: usize, value: Wasip1Value) -> Result<(), WasmError> {
        let kind = *self
            .local_kinds
            .get(local)
            .ok_or(WasmError::Invalid("local.set index out of range"))?;
        match (kind, value) {
            (Wasip1ValueKind::I32, Wasip1Value::I32(_))
            | (Wasip1ValueKind::I64, Wasip1Value::I64(_)) => {
                *self
                    .locals
                    .get_mut(local)
                    .ok_or(WasmError::Invalid("local.set index out of range"))? = value;
                Ok(())
            }
            _ => Err(WasmError::Invalid("local type mismatch")),
        }
    }

    fn push_control(&mut self, frame: ControlFrame) -> Result<(), WasmError> {
        let slot = self
            .controls
            .get_mut(self.control_len)
            .ok_or(WasmError::StackOverflow)?;
        *slot = frame;
        self.control_len += 1;
        Ok(())
    }

    fn branch(&mut self, depth: usize, reader: &mut Reader<'_>) -> Result<(), WasmError> {
        let Some(target_index) = self.control_len.checked_sub(depth.saturating_add(1)) else {
            return Err(WasmError::Invalid("branch target out of range"));
        };
        let frame = self.controls[target_index];
        match frame.kind {
            ControlKind::Loop => {
                self.control_len = target_index + 1;
                reader.pos = frame.start_pos;
                self.pc = reader.pos;
            }
            ControlKind::Block | ControlKind::If => {
                self.control_len = target_index;
                reader.pos = frame.end_pos.saturating_add(1);
                self.pc = reader.pos;
            }
        }
        Ok(())
    }

    fn decode_br_table_depth(&mut self, reader: &mut Reader<'_>) -> Result<usize, WasmError> {
        let count = reader.read_var_u32()? as usize;
        if count > TINY_WASIP1_BR_TABLE_CAPACITY {
            return Err(WasmError::Unsupported("br_table too large"));
        }
        let mut labels = [0usize; TINY_WASIP1_BR_TABLE_CAPACITY];
        let mut index = 0usize;
        while index < count {
            labels[index] = reader.read_var_u32()? as usize;
            index += 1;
        }
        let default = reader.read_var_u32()? as usize;
        let selected = self.pop_value_i32()? as usize;
        Ok(if selected < count {
            labels[selected]
        } else {
            default
        })
    }

    fn load_effective_addr(&mut self, reader: &mut Reader<'_>) -> Result<usize, WasmError> {
        let _align = reader.read_var_u32()?;
        let offset = reader.read_var_u32()?;
        let base = self.pop_value_i32()?;
        let addr = base.checked_add(offset).ok_or(WasmError::Truncated)?;
        self.translate_addr(addr)
    }

    fn store_effective_addr(&mut self, reader: &mut Reader<'_>) -> Result<usize, WasmError> {
        let _align = reader.read_var_u32()?;
        let offset = reader.read_var_u32()?;
        let base = self.pop_value_i32()?;
        let addr = base.checked_add(offset).ok_or(WasmError::Truncated)?;
        self.translate_addr(addr)
    }

    fn binary_i32(&mut self, pc: usize, op: fn(u32, u32) -> u32) -> Result<(), WasmError> {
        let rhs = self.pop_value_i32()?;
        let lhs = self.pop_value_i32()?;
        self.push_value(Wasip1Value::I32(op(lhs, rhs)))?;
        self.pc = pc;
        Ok(())
    }

    fn binary_i64(&mut self, pc: usize, op: fn(u64, u64) -> u64) -> Result<(), WasmError> {
        let rhs = self.pop_value_i64()?;
        let lhs = self.pop_value_i64()?;
        self.push_value(Wasip1Value::I64(op(lhs, rhs)))?;
        self.pc = pc;
        Ok(())
    }

    fn binary_i64_cmp(&mut self, pc: usize, op: fn(u64, u64) -> bool) -> Result<(), WasmError> {
        let rhs = self.pop_value_i64()?;
        let lhs = self.pop_value_i64()?;
        self.push_value(Wasip1Value::I32(op(lhs, rhs) as u32))?;
        self.pc = pc;
        Ok(())
    }

    fn call_rust_std_write(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        let len = self.pop_value_i32()?;
        let ptr = self.pop_value_i32()?;
        let fd = self.pop_value_i32()?;
        if fd > u8::MAX as u32 || len > u8::MAX as u32 {
            return Err(WasmError::Invalid("rust std write argument out of range"));
        }
        let call = TinyWasip1FdWriteCall {
            fd: fd as u8,
            iovs: ptr,
            iovs_len: 0,
            nwritten: len,
        };
        self.pending = Some(PendingWasip1Call::FdWrite(
            call,
            PendingWasip1Return::FdWriteLen,
        ));
        Ok(TinyWasip1Trap::FdWrite(call))
    }

    fn call_rust_std_clock_nanosleep(&mut self) -> Result<TinyWasip1Trap, WasmError> {
        let rmtp = self.pop_value_i32()?;
        let rqtp = self.pop_value_i32()?;
        let _flags = self.pop_value_i32()?;
        let _clockid = self.pop_value_i32()?;
        let call = TinyWasip1PollOneoffCall {
            in_ptr: rqtp,
            out_ptr: rmtp,
            nsubscriptions: 1,
            nevents: 0,
        };
        self.pending = Some(PendingWasip1Call::PollOneoff(
            call,
            PendingWasip1Return::ErrnoValueStack,
        ));
        Ok(TinyWasip1Trap::PollOneoff(call))
    }
}

fn find_matching_end(code: &[u8], start_pos: usize) -> Result<usize, WasmError> {
    let mut reader = Reader {
        bytes: code,
        pos: start_pos,
    };
    let mut depth = 0usize;
    while !reader.is_empty() {
        let opcode_pos = reader.pos;
        let opcode = reader.read_u8()?;
        match opcode {
            OPCODE_BLOCK | OPCODE_LOOP | OPCODE_IF => {
                let block_type = reader.read_u8()?;
                let _ = decode_core_block_type(block_type)?;
                depth = depth.saturating_add(1);
            }
            OPCODE_END => {
                if depth == 0 {
                    return Ok(opcode_pos);
                }
                depth -= 1;
            }
            _ => skip_instruction_immediate(&mut reader, opcode)?,
        }
    }
    Err(WasmError::Truncated)
}

fn find_matching_else_or_end(code: &[u8], start_pos: usize) -> Result<(usize, usize), WasmError> {
    let mut reader = Reader {
        bytes: code,
        pos: start_pos,
    };
    let mut depth = 0usize;
    let mut else_pos = usize::MAX;
    while !reader.is_empty() {
        let opcode_pos = reader.pos;
        let opcode = reader.read_u8()?;
        match opcode {
            OPCODE_BLOCK | OPCODE_LOOP | OPCODE_IF => {
                let block_type = reader.read_u8()?;
                let _ = decode_core_block_type(block_type)?;
                depth = depth.saturating_add(1);
            }
            OPCODE_ELSE if depth == 0 => {
                else_pos = opcode_pos;
            }
            OPCODE_END => {
                if depth == 0 {
                    return Ok((else_pos, opcode_pos));
                }
                depth -= 1;
            }
            _ => skip_instruction_immediate(&mut reader, opcode)?,
        }
    }
    Err(WasmError::Truncated)
}

fn skip_instruction_immediate(reader: &mut Reader<'_>, opcode: u8) -> Result<(), WasmError> {
    match opcode {
        OPCODE_BR | OPCODE_BR_IF | OPCODE_CALL | OPCODE_LOCAL_GET | OPCODE_LOCAL_SET
        | OPCODE_LOCAL_TEE | OPCODE_GLOBAL_GET | OPCODE_GLOBAL_SET | OPCODE_TABLE_GET
        | OPCODE_TABLE_SET | OPCODE_REF_FUNC => {
            reader.read_var_u32()?;
        }
        OPCODE_CALL_INDIRECT => {
            reader.read_var_u32()?;
            reader.read_var_u32()?;
        }
        OPCODE_BR_TABLE => {
            let count = reader.read_var_u32()?;
            for _ in 0..count {
                reader.read_var_u32()?;
            }
            reader.read_var_u32()?;
        }
        OPCODE_I32_LOAD | OPCODE_I64_LOAD | OPCODE_F32_LOAD | OPCODE_F64_LOAD
        | OPCODE_I32_LOAD8_S | OPCODE_I32_LOAD8_U | OPCODE_I32_LOAD16_S | OPCODE_I32_LOAD16_U
        | OPCODE_I64_LOAD8_S | OPCODE_I64_LOAD8_U | OPCODE_I64_LOAD16_S | OPCODE_I64_LOAD16_U
        | OPCODE_I64_LOAD32_S | OPCODE_I64_LOAD32_U | OPCODE_I32_STORE | OPCODE_I64_STORE
        | OPCODE_F32_STORE | OPCODE_F64_STORE | OPCODE_I32_STORE8 | OPCODE_I32_STORE16
        | OPCODE_I64_STORE8 | OPCODE_I64_STORE16 | OPCODE_I64_STORE32 => {
            reader.read_var_u32()?;
            reader.read_var_u32()?;
        }
        OPCODE_I32_CONST => {
            reader.read_var_i32()?;
        }
        OPCODE_I64_CONST => {
            reader.read_var_i64()?;
        }
        OPCODE_F32_CONST => {
            reader.read_fixed_u32()?;
        }
        OPCODE_F64_CONST => {
            reader.read_fixed_u64()?;
        }
        OPCODE_MEMORY_SIZE | OPCODE_MEMORY_GROW => {
            reader.read_u8()?;
        }
        OPCODE_REF_NULL => {
            reader.read_u8()?;
        }
        OPCODE_MISC => {
            let subopcode = reader.read_var_u32()?;
            match subopcode {
                8 => {
                    reader.read_var_u32()?;
                    reader.read_u8()?;
                }
                9 => {
                    reader.read_var_u32()?;
                }
                10 => {
                    reader.read_u8()?;
                    reader.read_u8()?;
                }
                11 => {
                    reader.read_u8()?;
                }
                12 => {
                    reader.read_var_u32()?;
                    reader.read_var_u32()?;
                }
                13 => {
                    reader.read_var_u32()?;
                }
                14 => {
                    reader.read_var_u32()?;
                    reader.read_var_u32()?;
                }
                15 | 16 | 17 => {
                    reader.read_var_u32()?;
                }
                _ => return Err(WasmError::UnsupportedOpcode(OPCODE_MISC)),
            }
        }
        OPCODE_UNREACHABLE
        | OPCODE_NOP
        | OPCODE_ELSE
        | OPCODE_RETURN
        | OPCODE_SELECT
        | OPCODE_DROP
        | OPCODE_I32_EQZ
        | OPCODE_I32_EQ
        | OPCODE_I32_NE
        | OPCODE_I32_LT_S
        | OPCODE_I32_LT_U
        | OPCODE_I32_GT_S
        | OPCODE_I32_GT_U
        | OPCODE_I32_LE_S
        | OPCODE_I32_LE_U
        | OPCODE_I32_GE_S
        | OPCODE_I32_GE_U
        | OPCODE_I64_EQZ
        | OPCODE_I64_EQ
        | OPCODE_I64_NE
        | OPCODE_I64_LT_S
        | OPCODE_I64_LT_U
        | OPCODE_I64_GT_S
        | OPCODE_I64_GT_U
        | OPCODE_I64_LE_S
        | OPCODE_I64_LE_U
        | OPCODE_I64_GE_S
        | OPCODE_I64_GE_U
        | OPCODE_F32_EQ
        | OPCODE_F32_NE
        | OPCODE_F32_LT
        | OPCODE_F32_GT
        | OPCODE_F32_LE
        | OPCODE_F32_GE
        | OPCODE_F64_EQ
        | OPCODE_F64_NE
        | OPCODE_F64_LT
        | OPCODE_F64_GT
        | OPCODE_F64_LE
        | OPCODE_F64_GE
        | OPCODE_I32_CLZ
        | OPCODE_I32_CTZ
        | OPCODE_I32_POPCNT
        | OPCODE_I32_ADD
        | OPCODE_I32_SUB
        | OPCODE_I32_MUL
        | OPCODE_I32_DIV_S
        | OPCODE_I32_DIV_U
        | OPCODE_I32_REM_S
        | OPCODE_I32_REM_U
        | OPCODE_I32_AND
        | OPCODE_I32_OR
        | OPCODE_I32_XOR
        | OPCODE_I32_SHL
        | OPCODE_I32_SHR_S
        | OPCODE_I32_SHR_U
        | OPCODE_I32_ROTL
        | OPCODE_I32_ROTR
        | OPCODE_I64_CLZ
        | OPCODE_I64_CTZ
        | OPCODE_I64_POPCNT
        | OPCODE_I64_ADD
        | OPCODE_I64_SUB
        | OPCODE_I64_MUL
        | OPCODE_I64_DIV_S
        | OPCODE_I64_DIV_U
        | OPCODE_I64_REM_S
        | OPCODE_I64_REM_U
        | OPCODE_I64_AND
        | OPCODE_I64_OR
        | OPCODE_I64_XOR
        | OPCODE_I64_SHL
        | OPCODE_I64_SHR_S
        | OPCODE_I64_SHR_U
        | OPCODE_I64_ROTL
        | OPCODE_I64_ROTR
        | OPCODE_F32_ABS
        | OPCODE_F32_NEG
        | OPCODE_F32_CEIL
        | OPCODE_F32_FLOOR
        | OPCODE_F32_TRUNC
        | OPCODE_F32_NEAREST
        | OPCODE_F32_SQRT
        | OPCODE_F32_ADD
        | OPCODE_F32_SUB
        | OPCODE_F32_MUL
        | OPCODE_F32_DIV
        | OPCODE_F32_MIN
        | OPCODE_F32_MAX
        | OPCODE_F32_COPYSIGN
        | OPCODE_F64_ABS
        | OPCODE_F64_NEG
        | OPCODE_F64_CEIL
        | OPCODE_F64_FLOOR
        | OPCODE_F64_TRUNC
        | OPCODE_F64_NEAREST
        | OPCODE_F64_SQRT
        | OPCODE_F64_ADD
        | OPCODE_F64_SUB
        | OPCODE_F64_MUL
        | OPCODE_F64_DIV
        | OPCODE_F64_MIN
        | OPCODE_F64_MAX
        | OPCODE_F64_COPYSIGN
        | OPCODE_I32_WRAP_I64
        | OPCODE_I32_TRUNC_F32_S
        | OPCODE_I32_TRUNC_F32_U
        | OPCODE_I32_TRUNC_F64_S
        | OPCODE_I32_TRUNC_F64_U
        | OPCODE_I64_EXTEND_I32_S
        | OPCODE_I64_EXTEND_I32_U
        | OPCODE_I64_TRUNC_F32_S
        | OPCODE_I64_TRUNC_F32_U
        | OPCODE_I64_TRUNC_F64_S
        | OPCODE_I64_TRUNC_F64_U
        | OPCODE_F32_CONVERT_I32_S
        | OPCODE_F32_CONVERT_I32_U
        | OPCODE_F32_CONVERT_I64_S
        | OPCODE_F32_CONVERT_I64_U
        | OPCODE_F32_DEMOTE_F64
        | OPCODE_F64_CONVERT_I32_S
        | OPCODE_F64_CONVERT_I32_U
        | OPCODE_F64_CONVERT_I64_S
        | OPCODE_F64_CONVERT_I64_U
        | OPCODE_F64_PROMOTE_F32
        | OPCODE_I32_REINTERPRET_F32
        | OPCODE_I64_REINTERPRET_F64
        | OPCODE_F32_REINTERPRET_I32
        | OPCODE_F64_REINTERPRET_I64
        | OPCODE_I32_EXTEND8_S
        | OPCODE_I32_EXTEND16_S
        | OPCODE_I64_EXTEND8_S
        | OPCODE_I64_EXTEND16_S
        | OPCODE_I64_EXTEND32_S
        | OPCODE_REF_IS_NULL => {}
        _ => return Err(WasmError::UnsupportedOpcode(opcode)),
    }
    Ok(())
}

fn parse_type_section(
    section: &mut Reader<'_>,
    func_types: &mut [FuncSig; MAX_FUNC_TYPES],
) -> Result<usize, WasmError> {
    let count = section.read_var_u32()? as usize;
    if count > MAX_FUNC_TYPES {
        return Err(WasmError::Unsupported("too many function types"));
    }
    for slot in func_types.iter_mut() {
        *slot = FuncSig::Unsupported;
    }
    for func_type in func_types.iter_mut().take(count) {
        if section.read_u8()? != FUNC_TYPE_FORM {
            return Err(WasmError::Invalid("type section expects function forms"));
        }
        let param_count = section.read_var_u32()?;
        *func_type = match param_count {
            0 => {
                let result_count = section.read_var_u32()?;
                if result_count != 0 {
                    return Err(WasmError::Unsupported("results are not supported"));
                }
                FuncSig::UnitToUnit
            }
            1 => {
                if section.read_u8()? != VALTYPE_I32 {
                    return Err(WasmError::Unsupported("only i32 params are supported"));
                }
                let result_count = section.read_var_u32()?;
                if result_count != 0 {
                    return Err(WasmError::Unsupported("results are not supported"));
                }
                FuncSig::I32ToUnit
            }
            _ => return Err(WasmError::Unsupported("only arity 0 or 1 is supported")),
        };
    }
    Ok(count)
}

fn parse_import_section(
    section: &mut Reader<'_>,
    func_types: &[FuncSig],
) -> Result<(u32, Option<OptionalImport>), WasmError> {
    let count = section.read_var_u32()?;
    if !(MIN_IMPORT_COUNT..=MAX_IMPORT_COUNT).contains(&count) {
        return Err(WasmError::Unsupported("demo expects two or three imports"));
    }
    let mut optional_import = None;
    for index in 0..count {
        let module_name = section.read_name()?;
        let field_name = section.read_name()?;
        if section.read_u8()? != EXTERNAL_KIND_FUNC {
            return Err(WasmError::Unsupported(
                "only function imports are supported",
            ));
        }
        let type_index = section.read_var_u32()? as usize;
        let func_type = *func_types
            .get(type_index)
            .ok_or(WasmError::Invalid("import type index out of range"))?;
        match index {
            LOG_IMPORT_INDEX => {
                if module_name != b"hibana"
                    || field_name != b"log_u32"
                    || func_type != FuncSig::I32ToUnit
                {
                    return Err(WasmError::Invalid(
                        "first import must be hibana.log_u32(i32)",
                    ));
                }
            }
            YIELD_IMPORT_INDEX => {
                if module_name != b"hibana"
                    || field_name != b"yield_now"
                    || func_type != FuncSig::UnitToUnit
                {
                    return Err(WasmError::Invalid(
                        "second import must be hibana.yield_now()",
                    ));
                }
            }
            SLEEP_IMPORT_INDEX => {
                if module_name != b"hibana" || func_type != FuncSig::I32ToUnit {
                    return Err(WasmError::Invalid(
                        "third import must be a hibana i32 import",
                    ));
                }
                optional_import = match field_name {
                    b"sleep_until" => Some(OptionalImport::SleepUntil),
                    b"gpio_set" => Some(OptionalImport::GpioSet),
                    _ => return Err(WasmError::Invalid("unsupported third hibana import")),
                };
            }
            _ => return Err(WasmError::Unsupported("unexpected import index")),
        }
    }
    Ok((count, optional_import))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct TinyWasip1ImportInfo {
    import_count: u32,
    fd_write_index: u32,
    poll_oneoff_index: u32,
    saw_std_start_import: bool,
}

#[cfg(test)]
impl TinyWasip1ImportInfo {
    const fn looks_like_ordinary_rust_std(self) -> bool {
        self.import_count > 2 && self.saw_std_start_import
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg(test)]
struct TinyWasip1ExportInfo {
    selected_index: u32,
    start_index: Option<u32>,
    has_start: bool,
    has_memory: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg(test)]
struct TinyWasip1NameInfo {
    rust_main_index: Option<u32>,
    write_index: Option<u32>,
    clock_nanosleep_index: Option<u32>,
}

#[cfg(test)]
fn validate_tiny_wasip1_imports_against_handlers(
    bytes: &[u8],
    handlers: Wasip1HandlerSet,
) -> Result<(), WasmError> {
    if contains_bytes(bytes, WASIP1_IMPORT_FD_WRITE) && !handlers.supports(Wasip1Syscall::FdWrite) {
        return Err(WasmError::Unsupported(
            "wasip1 fd_write disabled by feature profile",
        ));
    }
    if contains_bytes(bytes, WASIP1_IMPORT_POLL_ONEOFF)
        && !handlers.supports(Wasip1Syscall::PollOneoff)
    {
        return Err(WasmError::Unsupported(
            "wasip1 poll_oneoff disabled by feature profile",
        ));
    }
    if contains_bytes(bytes, WASIP1_IMPORT_PROC_EXIT) && !handlers.supports(Wasip1Syscall::ProcExit)
    {
        return Err(WasmError::Unsupported(
            "wasip1 proc_exit disabled by feature profile",
        ));
    }
    if contains_bytes(bytes, WASIP1_IMPORT_PROC_RAISE)
        && !handlers.supports(Wasip1Syscall::ProcRaise)
    {
        return Err(WasmError::Unsupported(
            "wasip1 proc_raise disabled by feature profile",
        ));
    }
    if contains_bytes(bytes, WASIP1_IMPORT_SCHED_YIELD)
        && !handlers.supports(Wasip1Syscall::SchedYield)
    {
        return Err(WasmError::Unsupported(
            "wasip1 sched_yield disabled by feature profile",
        ));
    }
    if contains_bytes(bytes, WASIP1_IMPORT_CLOCK_RES_GET)
        && !handlers.supports(Wasip1Syscall::ClockResGet)
    {
        return Err(WasmError::Unsupported(
            "wasip1 clock_res_get disabled by feature profile",
        ));
    }
    if (contains_bytes(bytes, WASIP1_IMPORT_SOCK_ACCEPT)
        || contains_bytes(bytes, WASIP1_IMPORT_SOCK_RECV)
        || contains_bytes(bytes, WASIP1_IMPORT_SOCK_SEND)
        || contains_bytes(bytes, WASIP1_IMPORT_SOCK_SHUTDOWN))
        && !handlers.supports(Wasip1Syscall::NetworkObject)
    {
        return Err(WasmError::Unsupported(
            "wasip1 NetworkObject imports disabled by feature profile",
        ));
    }
    if (contains_bytes(bytes, WASIP1_IMPORT_ARGS_GET)
        || contains_bytes(bytes, WASIP1_IMPORT_ARGS_SIZES_GET)
        || contains_bytes(bytes, WASIP1_IMPORT_ENVIRON_GET)
        || contains_bytes(bytes, WASIP1_IMPORT_ENVIRON_SIZES_GET))
        && !handlers.supports(Wasip1Syscall::ArgsEnv)
    {
        return Err(WasmError::Unsupported(
            "wasip1 args/env disabled by feature profile",
        ));
    }
    Ok(())
}

fn validate_core_wasip1_imports(
    module: &CoreWasmModule<'_>,
    handlers: Wasip1HandlerSet,
) -> Result<(), WasmError> {
    for index in 0..module.import_count {
        let import = module.imports[index].ok_or(WasmError::Invalid("missing core import"))?;
        if import.module != WASIP1_IMPORT_MODULE {
            return Err(WasmError::Unsupported("unsupported host import module"));
        }
        let ty = module.core_func_type(module.import_type_indices[index])?;
        match import.name {
            WASIP1_IMPORT_FD_WRITE => {
                if !handlers.supports(Wasip1Syscall::FdWrite) {
                    return Err(WasmError::Unsupported(
                        "wasip1 fd_write disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 4],
                    &[CoreWasmValueKind::I32; 1],
                    "fd_write import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_READ => {
                if !handlers.supports(Wasip1Syscall::FdRead) {
                    return Err(WasmError::Unsupported(
                        "wasip1 fd_read disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 4],
                    &[CoreWasmValueKind::I32; 1],
                    "fd_read import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_FDSTAT_GET => {
                if !handlers.supports(Wasip1Syscall::FdFdstatGet) {
                    return Err(WasmError::Unsupported(
                        "wasip1 fd_fdstat_get disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "fd_fdstat_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_CLOSE => {
                if !handlers.supports(Wasip1Syscall::FdClose) {
                    return Err(WasmError::Unsupported(
                        "wasip1 fd_close disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 1],
                    &[CoreWasmValueKind::I32; 1],
                    "fd_close import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_PRESTAT_GET | WASIP1_IMPORT_FD_FILESTAT_GET => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    "path-minimal two-arg import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_PRESTAT_DIR_NAME
            | WASIP1_IMPORT_PATH_CREATE_DIRECTORY
            | WASIP1_IMPORT_PATH_REMOVE_DIRECTORY
            | WASIP1_IMPORT_PATH_UNLINK_FILE => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 3],
                    "path-minimal three-arg import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_READDIR => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    "fd_readdir import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_OPEN => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                    ],
                    "path_open import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_FILESTAT_GET => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                    ],
                    "path_filestat_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_READLINK | WASIP1_IMPORT_PATH_RENAME => {
                validate_core_path_minimal_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 6],
                    "path-minimal six-arg import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_DATASYNC | WASIP1_IMPORT_FD_SYNC => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 1],
                    "path-full one-arg import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_FDSTAT_SET_FLAGS
            | WASIP1_IMPORT_FD_RENUMBER
            | WASIP1_IMPORT_FD_TELL => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    "path-full two-arg import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_FDSTAT_SET_RIGHTS => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                    ],
                    "fd_fdstat_set_rights import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_ALLOCATE => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                    ],
                    "fd_allocate import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_FILESTAT_SET_SIZE => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32, CoreWasmValueKind::I64],
                    "fd_filestat_set_size import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_ADVISE => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    "fd_advise import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_FILESTAT_SET_TIMES => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    "fd_filestat_set_times import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_PREAD | WASIP1_IMPORT_FD_PWRITE => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    "path-full fd vector-offset import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_FD_SEEK => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                    ],
                    "fd_seek import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_FILESTAT_SET_TIMES => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    "path_filestat_set_times import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_LINK => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                    ],
                    "path_link import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PATH_SYMLINK => {
                validate_core_path_full_import(
                    handlers,
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I32,
                    ],
                    "path_symlink import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_CLOCK_RES_GET => {
                if !handlers.supports(Wasip1Syscall::ClockResGet) {
                    return Err(WasmError::Unsupported(
                        "wasip1 clock_res_get disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32, CoreWasmValueKind::I32],
                    &[CoreWasmValueKind::I32; 1],
                    "clock_res_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_CLOCK_TIME_GET => {
                if !handlers.supports(Wasip1Syscall::ClockTimeGet) {
                    return Err(WasmError::Unsupported(
                        "wasip1 clock_time_get disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[
                        CoreWasmValueKind::I32,
                        CoreWasmValueKind::I64,
                        CoreWasmValueKind::I32,
                    ],
                    &[CoreWasmValueKind::I32; 1],
                    "clock_time_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_POLL_ONEOFF => {
                if !handlers.supports(Wasip1Syscall::PollOneoff) {
                    return Err(WasmError::Unsupported(
                        "wasip1 poll_oneoff disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 4],
                    &[CoreWasmValueKind::I32; 1],
                    "poll_oneoff import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_RANDOM_GET => {
                if !handlers.supports(Wasip1Syscall::RandomGet) {
                    return Err(WasmError::Unsupported(
                        "wasip1 random_get disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "random_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_SCHED_YIELD => {
                if !handlers.supports(Wasip1Syscall::SchedYield) {
                    return Err(WasmError::Unsupported(
                        "wasip1 sched_yield disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[],
                    &[CoreWasmValueKind::I32; 1],
                    "sched_yield import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_ARGS_SIZES_GET => {
                if !handlers.supports(Wasip1Syscall::ArgsEnv) {
                    return Err(WasmError::Unsupported(
                        "wasip1 args/env disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "args_sizes_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_ARGS_GET => {
                if !handlers.supports(Wasip1Syscall::ArgsEnv) {
                    return Err(WasmError::Unsupported(
                        "wasip1 args/env disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "args_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_ENVIRON_SIZES_GET => {
                if !handlers.supports(Wasip1Syscall::ArgsEnv) {
                    return Err(WasmError::Unsupported(
                        "wasip1 args/env disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "environ_sizes_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_ENVIRON_GET => {
                if !handlers.supports(Wasip1Syscall::ArgsEnv) {
                    return Err(WasmError::Unsupported(
                        "wasip1 args/env disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    &[CoreWasmValueKind::I32; 1],
                    "environ_get import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PROC_EXIT => {
                if !handlers.supports(Wasip1Syscall::ProcExit) {
                    return Err(WasmError::Unsupported(
                        "wasip1 proc_exit disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 1],
                    &[],
                    "proc_exit import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_PROC_RAISE => {
                if !handlers.supports(Wasip1Syscall::ProcRaise) {
                    return Err(WasmError::Unsupported(
                        "wasip1 proc_raise disabled by feature profile",
                    ));
                }
                validate_core_import_sig(
                    ty,
                    &[CoreWasmValueKind::I32; 1],
                    &[CoreWasmValueKind::I32; 1],
                    "proc_raise import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_SOCK_ACCEPT => {
                validate_core_sock_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 3],
                    "sock_accept import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_SOCK_RECV => {
                validate_core_sock_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 6],
                    "sock_recv import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_SOCK_SEND => {
                validate_core_sock_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 5],
                    "sock_send import signature mismatch",
                )?;
            }
            WASIP1_IMPORT_SOCK_SHUTDOWN => {
                validate_core_sock_import(
                    handlers,
                    ty,
                    &[CoreWasmValueKind::I32; 2],
                    "sock_shutdown import signature mismatch",
                )?;
            }
            _ => return Err(WasmError::Unsupported("unsupported wasip1 import")),
        }
    }
    Ok(())
}

fn validate_core_import_sig(
    ty: CoreWasmFuncType,
    params: &[CoreWasmValueKind],
    results: &[CoreWasmValueKind],
    error: &'static str,
) -> Result<(), WasmError> {
    if ty.param_count != params.len() || ty.result_count != results.len() {
        return Err(WasmError::Invalid(error));
    }
    for (actual, expected) in ty.params.iter().take(ty.param_count).zip(params.iter()) {
        if actual != expected {
            return Err(WasmError::Invalid(error));
        }
    }
    for (actual, expected) in ty.results.iter().take(ty.result_count).zip(results.iter()) {
        if actual != expected {
            return Err(WasmError::Invalid(error));
        }
    }
    Ok(())
}

fn validate_core_path_minimal_import(
    handlers: Wasip1HandlerSet,
    ty: CoreWasmFuncType,
    params: &[CoreWasmValueKind],
    error: &'static str,
) -> Result<(), WasmError> {
    if !handlers.supports(Wasip1Syscall::PathMinimal) {
        return Err(WasmError::Unsupported(
            "wasip1 path-minimal disabled by feature profile",
        ));
    }
    validate_core_import_sig(ty, params, &[CoreWasmValueKind::I32; 1], error)
}

fn validate_core_path_full_import(
    handlers: Wasip1HandlerSet,
    ty: CoreWasmFuncType,
    params: &[CoreWasmValueKind],
    error: &'static str,
) -> Result<(), WasmError> {
    if !handlers.supports(Wasip1Syscall::PathFull) {
        return Err(WasmError::Unsupported(
            "wasip1 path-full disabled by feature profile",
        ));
    }
    validate_core_import_sig(ty, params, &[CoreWasmValueKind::I32; 1], error)
}

fn validate_core_sock_import(
    handlers: Wasip1HandlerSet,
    ty: CoreWasmFuncType,
    params: &[CoreWasmValueKind],
    error: &'static str,
) -> Result<(), WasmError> {
    if !handlers.supports(Wasip1Syscall::NetworkObject) {
        return Err(WasmError::Unsupported(
            "wasip1 NetworkObject imports disabled by feature profile",
        ));
    }
    validate_core_import_sig(ty, params, &[CoreWasmValueKind::I32; 1], error)
}

#[cfg(test)]
fn parse_tiny_wasip1_import_section(
    section: &mut Reader<'_>,
) -> Result<TinyWasip1ImportInfo, WasmError> {
    let count = section.read_var_u32()?;
    if count < 2 {
        return Err(WasmError::Unsupported(
            "tiny wasip1 traffic guest expects fd_write and poll_oneoff imports",
        ));
    }
    let mut fd_write_index = None;
    let mut poll_oneoff_index = None;
    let mut saw_std_start_import = false;
    for index in 0..count {
        let module_name = section.read_name()?;
        let field_name = section.read_name()?;
        if section.read_u8()? != EXTERNAL_KIND_FUNC {
            return Err(WasmError::Unsupported(
                "tiny wasip1 traffic imports must be functions",
            ));
        }
        let _type_index = section.read_var_u32()?;
        if module_name == WASIP1_IMPORT_MODULE && field_name == WASIP1_IMPORT_FD_WRITE {
            fd_write_index = Some(index);
        } else if module_name == WASIP1_IMPORT_MODULE && field_name == WASIP1_IMPORT_POLL_ONEOFF {
            poll_oneoff_index = Some(index);
        } else if module_name == WASIP1_IMPORT_MODULE && is_rust_std_start_import(field_name) {
            saw_std_start_import = true;
        }
    }
    Ok(TinyWasip1ImportInfo {
        import_count: count,
        fd_write_index: fd_write_index.ok_or(WasmError::Invalid("missing fd_write import"))?,
        poll_oneoff_index: poll_oneoff_index
            .ok_or(WasmError::Invalid("missing poll_oneoff import"))?,
        saw_std_start_import,
    })
}

#[cfg(test)]
fn is_rust_std_start_import(field_name: &[u8]) -> bool {
    field_name == WASIP1_IMPORT_ARGS_GET
        || field_name == WASIP1_IMPORT_ARGS_SIZES_GET
        || field_name == WASIP1_IMPORT_ENVIRON_GET
        || field_name == WASIP1_IMPORT_ENVIRON_SIZES_GET
        || field_name == WASIP1_IMPORT_PROC_EXIT
        || field_name == WASIP1_IMPORT_PROC_RAISE
}

#[cfg(test)]
fn parse_tiny_wasip1_memory_section(section: &mut Reader<'_>) -> Result<(), WasmError> {
    let count = section.read_var_u32()?;
    if count != 1 {
        return Err(WasmError::Unsupported(
            "tiny wasip1 traffic guest expects one memory",
        ));
    }
    let flags = section.read_u8()?;
    let min_pages = section.read_var_u32()?;
    if flags & 0x01 != 0 {
        let _max_pages = section.read_var_u32()?;
    }
    if flags & !0x03 != 0 || min_pages == 0 {
        return Err(WasmError::Invalid("unsupported tiny wasip1 memory limits"));
    }
    Ok(())
}

#[cfg(test)]
fn parse_tiny_wasip1_traffic_module<'a>(
    bytes: &'a [u8],
) -> Result<ParsedTinyWasip1Module<'a>, WasmError> {
    let mut reader = Reader::new(bytes);
    if reader.read_bytes(4)? != WASM_MAGIC {
        return Err(WasmError::Invalid("invalid wasm magic"));
    }
    if reader.read_bytes(4)? != WASM_VERSION {
        return Err(WasmError::Invalid("unsupported wasm version"));
    }

    let mut saw_imports = false;
    let mut saw_memory = false;
    let mut saw_export = false;
    let mut imports = None;
    let mut exports = None;
    let mut code_section_bytes = None;
    let mut names = TinyWasip1NameInfo::default();
    let mut data_segments = [None; TINY_WASIP1_MAX_DATA_SEGMENTS];
    let mut min_data_offset = u32::MAX;
    while !reader.is_empty() {
        let section_id = reader.read_u8()?;
        let section_len = reader.read_var_u32()? as usize;
        let section_bytes = reader.read_bytes(section_len)?;
        let mut section = Reader::new(section_bytes);
        let consumed = match section_id {
            SECTION_IMPORT => {
                imports = Some(parse_tiny_wasip1_import_section(&mut section)?);
                saw_imports = true;
                true
            }
            SECTION_MEMORY => {
                parse_tiny_wasip1_memory_section(&mut section)?;
                saw_memory = true;
                true
            }
            SECTION_EXPORT => {
                exports = Some(parse_tiny_wasip1_export_section(&mut section)?);
                saw_export = true;
                true
            }
            SECTION_CODE => {
                code_section_bytes = Some(section_bytes);
                false
            }
            SECTION_DATA => {
                parse_tiny_wasip1_data_section(
                    &mut section,
                    &mut data_segments,
                    &mut min_data_offset,
                )?;
                true
            }
            SECTION_CUSTOM => {
                parse_tiny_wasip1_name_section(&mut section, &mut names)?;
                true
            }
            _ => false,
        };
        if consumed && !section.is_empty() {
            return Err(WasmError::Invalid("section has trailing bytes"));
        }
    }
    if !saw_imports || !saw_memory || !saw_export || code_section_bytes.is_none() {
        return Err(WasmError::Invalid("tiny wasip1 traffic module incomplete"));
    }
    let imports = imports.ok_or(WasmError::Invalid("missing import section"))?;
    let exports = exports.ok_or(WasmError::Invalid("missing export section"))?;
    let code_section_bytes =
        code_section_bytes.ok_or(WasmError::Invalid("missing code section"))?;

    {
        if exports.has_start
            && exports.has_memory
            && imports.looks_like_ordinary_rust_std()
            && names.rust_main_index.is_some()
            && names.write_index.is_some()
            && names.clock_nanosleep_index.is_some()
        {
            let selected_index = names.rust_main_index.ok_or(WasmError::Invalid(
                "ordinary rust std artifact missing main",
            ))?;
            let local_index =
                selected_index
                    .checked_sub(imports.import_count)
                    .ok_or(WasmError::Invalid(
                        "ordinary rust std main points to import",
                    ))? as usize;
            let mut code_section = Reader::new(code_section_bytes);
            let body = parse_tiny_wasip1_code_section(&mut code_section, local_index)?;
            if !code_section.is_empty() {
                return Err(WasmError::Invalid("code section has trailing bytes"));
            }
            let memory_base = min_data_offset
                .checked_sub(RUST_WASIP1_STACK_SLOP)
                .ok_or(WasmError::Invalid("data offset too low"))?;
            return Ok(ParsedTinyWasip1Module {
                program: TinyWasip1Program::RustStdMain {
                    body,
                    write_index: names
                        .write_index
                        .ok_or(WasmError::Invalid("missing write"))?,
                    clock_nanosleep_index: names
                        .clock_nanosleep_index
                        .ok_or(WasmError::Invalid("missing clock_nanosleep"))?,
                },
                memory_base,
                traffic_steps: None,
                data_segments,
            });
        }
    }

    let mut code_section = Reader::new(code_section_bytes);
    let selected_index = exports.selected_index;
    let local_index = selected_index
        .checked_sub(imports.import_count)
        .ok_or(WasmError::Invalid("start export points to import"))? as usize;
    let code = parse_tiny_wasip1_code_section(&mut code_section, local_index)?.code;
    if !code_section.is_empty() {
        return Err(WasmError::Invalid("code section has trailing bytes"));
    }

    if imports.import_count == 2
        && imports.fd_write_index == WASIP1_FD_WRITE_IMPORT_INDEX
        && imports.poll_oneoff_index == WASIP1_POLL_ONEOFF_IMPORT_INDEX
    {
        return Ok(ParsedTinyWasip1Module {
            program: TinyWasip1Program::Direct { code },
            memory_base: 0,
            traffic_steps: None,
            data_segments,
        });
    }

    let memory_base = min_data_offset
        .checked_sub(RUST_WASIP1_STACK_SLOP)
        .ok_or(WasmError::Invalid("data offset too low"))?;
    Ok(ParsedTinyWasip1Module {
        program: TinyWasip1Program::Rust {
            code,
            fd_write_index: imports.fd_write_index,
            poll_oneoff_index: imports.poll_oneoff_index,
        },
        memory_base,
        traffic_steps: None,
        data_segments,
    })
}

#[cfg(test)]
fn parse_tiny_wasip1_export_section(
    section: &mut Reader<'_>,
) -> Result<TinyWasip1ExportInfo, WasmError> {
    let count = section.read_var_u32()?;
    let mut start_index = None;
    let mut main_index = None;
    let mut saw_memory = false;
    for _ in 0..count {
        let export_name = section.read_name()?;
        let kind = section.read_u8()?;
        let index = section.read_var_u32()?;
        if export_name == b"_start" {
            if kind != EXTERNAL_KIND_FUNC {
                return Err(WasmError::Invalid("_start must export a function"));
            }
            start_index = Some(index);
        } else if export_name == b"__main_void" {
            if kind != EXTERNAL_KIND_FUNC {
                return Err(WasmError::Invalid("__main_void must export a function"));
            }
            main_index = Some(index);
        } else if export_name == b"memory" {
            if kind != 2 || index != 0 {
                return Err(WasmError::Invalid("memory export mismatch"));
            }
            saw_memory = true;
        }
    }
    if !saw_memory {
        return Err(WasmError::Invalid("missing memory export"));
    }
    let selected_index = main_index
        .or(start_index)
        .ok_or(WasmError::Invalid("missing _start export"))?;
    Ok(TinyWasip1ExportInfo {
        selected_index,
        start_index,
        has_start: start_index.is_some(),
        has_memory: saw_memory,
    })
}

#[cfg(test)]
fn parse_tiny_wasip1_name_section(
    section: &mut Reader<'_>,
    names: &mut TinyWasip1NameInfo,
) -> Result<(), WasmError> {
    let section_name = section.read_name()?;
    if section_name != b"name" {
        while !section.is_empty() {
            let _ = section.read_u8()?;
        }
        return Ok(());
    }
    while !section.is_empty() {
        let subsection_id = section.read_u8()?;
        let subsection_len = section.read_var_u32()? as usize;
        let subsection_bytes = section.read_bytes(subsection_len)?;
        if subsection_id != 1 {
            continue;
        }
        let mut subsection = Reader::new(subsection_bytes);
        let count = subsection.read_var_u32()?;
        for _ in 0..count {
            let function_index = subsection.read_var_u32()?;
            let function_name = subsection.read_name()?;
            if function_name == b"write" {
                names.write_index = Some(function_index);
            } else if function_name == b"clock_nanosleep" {
                names.clock_nanosleep_index = Some(function_index);
            } else if function_name != b"__main_void"
                && function_name != b"_start"
                && contains_bytes(function_name, b"4main17h")
            {
                if names.rust_main_index.is_some() {
                    return Err(WasmError::Unsupported(
                        "multiple ordinary rust std main candidates",
                    ));
                }
                names.rust_main_index = Some(function_index);
            }
        }
        if !subsection.is_empty() {
            return Err(WasmError::Invalid("name subsection has trailing bytes"));
        }
    }
    Ok(())
}

#[cfg(test)]
fn parse_tiny_wasip1_code_section<'a>(
    section: &mut Reader<'a>,
    wanted_local_index: usize,
) -> Result<TinyWasip1CodeBody<'a>, WasmError> {
    let count = section.read_var_u32()? as usize;
    let mut wanted_body = None;
    let mut index = 0usize;
    while index < count {
        let body_len = section.read_var_u32()? as usize;
        let body = section.read_bytes(body_len)?;
        if index == wanted_local_index {
            let mut body_reader = Reader::new(body);
            let local_decl_count = body_reader.read_var_u32()?;
            let mut local_count = 0usize;
            let mut local_kinds = [Wasip1ValueKind::I32; TINY_WASIP1_LOCAL_CAPACITY];
            for _ in 0..local_decl_count {
                let count = body_reader.read_var_u32()? as usize;
                let ty = body_reader.read_u8()?;
                let kind = match ty {
                    0x7f => Wasip1ValueKind::I32,
                    0x7e => Wasip1ValueKind::I64,
                    _ => return Err(WasmError::Unsupported("unsupported local type")),
                };
                let end = local_count
                    .checked_add(count)
                    .ok_or(WasmError::Unsupported("too many locals"))?;
                if end > TINY_WASIP1_LOCAL_CAPACITY {
                    return Err(WasmError::Unsupported("too many locals"));
                }
                for slot in local_kinds.iter_mut().take(end).skip(local_count) {
                    *slot = kind;
                }
                local_count = end;
            }
            wanted_body = Some(TinyWasip1CodeBody {
                code: &body[body_reader.pos..],
                local_count,
                local_kinds,
            });
        }
        index += 1;
    }
    wanted_body.ok_or(WasmError::Invalid("missing selected tiny wasip1 body"))
}

#[cfg(test)]
fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    if needle.is_empty() {
        return true;
    }
    if needle.len() > haystack.len() {
        return false;
    }
    let mut index = 0usize;
    while index <= haystack.len() - needle.len() {
        if &haystack[index..index + needle.len()] == needle {
            return true;
        }
        index += 1;
    }
    false
}

#[cfg(test)]
fn parse_tiny_wasip1_data_section<'a>(
    section: &mut Reader<'a>,
    segments: &mut [Option<TinyWasip1DataSegment<'a>>; TINY_WASIP1_MAX_DATA_SEGMENTS],
    min_data_offset: &mut u32,
) -> Result<(), WasmError> {
    let count = section.read_var_u32()? as usize;
    if count > segments.len() {
        return Err(WasmError::Unsupported("too many tiny wasip1 data segments"));
    }
    for slot in segments.iter_mut() {
        *slot = None;
    }
    for slot in segments.iter_mut().take(count) {
        let mode = section.read_var_u32()?;
        if mode != 0 {
            return Err(WasmError::Unsupported(
                "only active memory-zero data is supported",
            ));
        }
        if section.read_u8()? != OPCODE_I32_CONST {
            return Err(WasmError::Invalid("data offset must be i32.const"));
        }
        let offset = section.read_var_i32()?;
        if offset < 0 {
            return Err(WasmError::Invalid("negative data offset"));
        }
        if section.read_u8()? != OPCODE_END {
            return Err(WasmError::Invalid("data offset expression must end"));
        }
        let bytes_len = section.read_var_u32()? as usize;
        let bytes = section.read_bytes(bytes_len)?;
        let offset = offset as u32;
        *min_data_offset = (*min_data_offset).min(offset);
        *slot = Some(TinyWasip1DataSegment { offset, bytes });
    }
    Ok(())
}

fn parse_function_section(
    section: &mut Reader<'_>,
    func_types: &[FuncSig],
) -> Result<FuncSig, WasmError> {
    let count = section.read_var_u32()?;
    if count != 1 {
        return Err(WasmError::Unsupported(
            "demo expects exactly one local function",
        ));
    }
    let type_index = section.read_var_u32()? as usize;
    let func_type = *func_types
        .get(type_index)
        .ok_or(WasmError::Invalid("function type index out of range"))?;
    Ok(func_type)
}

fn parse_export_section(section: &mut Reader<'_>) -> Result<u32, WasmError> {
    let count = section.read_var_u32()?;
    for _ in 0..count {
        let export_name = section.read_name()?;
        let kind = section.read_u8()?;
        let index = section.read_var_u32()?;
        if export_name == b"_start" {
            if kind != EXTERNAL_KIND_FUNC {
                return Err(WasmError::Invalid("_start must export a function"));
            }
            return Ok(index);
        }
    }
    Err(WasmError::Invalid("missing _start export"))
}

fn parse_code_section<'a>(section: &mut Reader<'a>) -> Result<&'a [u8], WasmError> {
    let count = section.read_var_u32()?;
    if count != 1 {
        return Err(WasmError::Unsupported("demo expects exactly one code body"));
    }
    let body_len = section.read_var_u32()? as usize;
    let body = section.read_bytes(body_len)?;
    let mut body_reader = Reader::new(body);
    let local_decl_count = body_reader.read_var_u32()?;
    if local_decl_count != 0 {
        return Err(WasmError::Unsupported("locals are not supported"));
    }
    Ok(&body[body_reader.pos..])
}

#[cfg(test)]
mod tests {
    use super::{
        BAD_ROUTE_EARLY_YIELD_WASM_GUEST, BudgetedGuestTrap, CoreWasip1FdStat, CoreWasip1FileStat,
        CoreWasip1Instance, CoreWasip1PathKind, CoreWasip1SocketKind, CoreWasip1Trap,
        CoreWasmInstance, CoreWasmTrap, CoreWasmValue, DEMO_WASM_GUEST, EXTERNAL_KIND_FUNC,
        FUEL_EXHAUSTION_WASM_GUEST, GPIO_WASM_GUEST, GuestTrap, NORMAL_WASM_GUEST, OPCODE_CALL,
        OPCODE_DROP, OPCODE_END, OPCODE_I32_CONST, OPCODE_I64_CONST, ROUTE_WASM_ALERT_VALUE,
        ROUTE_WASM_GUEST, ROUTE_WASM_NORMAL_VALUE, SECTION_CODE, SECTION_EXPORT, SECTION_FUNCTION,
        SECTION_IMPORT, SECTION_MEMORY, SECTION_TYPE, SLEEP_WASM_GUEST, TRAP_WASM_GUEST,
        TinyWasip1TrafficLightInstance, TinyWasmInstance, TinyWasmModule, VALTYPE_I32, VALTYPE_I64,
        WASIP1_FDSTAT_RIGHTS_BASE_OFFSET, WASIP1_FILESTAT_FILETYPE_OFFSET,
        WASIP1_FILESTAT_SIZE_OFFSET, WASIP1_FILETYPE_DIRECTORY, WASIP1_FILETYPE_REGULAR_FILE,
        WASIP1_IMPORT_MODULE, WASIP1_PRESTAT_DIR_NAME_LEN_OFFSET, WasmError,
    };
    use crate::{
        choreography::protocol::{
            BudgetExpired, BudgetRun, EngineReq, EngineRet, FdRead, FdRequest, FdWrite, GpioSet,
            MemBorrow, MemFence, MemFenceReason, StdoutChunk, TimerSleepDone, TimerSleepUntil,
        },
        kernel::{
            features::Wasip1HandlerSet,
            wasi::{MemoryLeaseError, MemoryLeaseTable},
        },
    };
    use std::vec::Vec;

    #[derive(Clone, Copy)]
    enum TestWasmArg {
        I32(u32),
        I64(u64),
    }

    fn push_test_u32(out: &mut Vec<u8>, mut value: u32) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn push_test_i32(out: &mut Vec<u8>, value: u32) {
        let mut value = value as i32;
        loop {
            let byte = (value as u8) & 0x7f;
            value >>= 7;
            let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
            out.push(if done { byte } else { byte | 0x80 });
            if done {
                break;
            }
        }
    }

    fn push_test_i64(out: &mut Vec<u8>, value: u64) {
        let mut value = value as i64;
        loop {
            let byte = (value as u8) & 0x7f;
            value >>= 7;
            let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
            out.push(if done { byte } else { byte | 0x80 });
            if done {
                break;
            }
        }
    }

    fn push_test_name(out: &mut Vec<u8>, name: &[u8]) {
        push_test_u32(out, name.len() as u32);
        out.extend_from_slice(name);
    }

    fn push_test_section(out: &mut Vec<u8>, id: u8, section: &[u8]) {
        out.push(id);
        push_test_u32(out, section.len() as u32);
        out.extend_from_slice(section);
    }

    fn core_wasip1_single_import_module(
        import_name: &[u8],
        import_params: &[u8],
        import_results: &[u8],
        args: &[TestWasmArg],
        needs_memory: bool,
    ) -> Vec<u8> {
        let mut module = Vec::new();
        module.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

        let mut types = Vec::new();
        push_test_u32(&mut types, 2);
        types.push(0x60);
        push_test_u32(&mut types, import_params.len() as u32);
        types.extend_from_slice(import_params);
        push_test_u32(&mut types, import_results.len() as u32);
        types.extend_from_slice(import_results);
        types.push(0x60);
        push_test_u32(&mut types, 0);
        push_test_u32(&mut types, 0);
        push_test_section(&mut module, SECTION_TYPE, &types);

        let mut imports = Vec::new();
        push_test_u32(&mut imports, 1);
        push_test_name(&mut imports, WASIP1_IMPORT_MODULE);
        push_test_name(&mut imports, import_name);
        imports.push(EXTERNAL_KIND_FUNC);
        push_test_u32(&mut imports, 0);
        push_test_section(&mut module, SECTION_IMPORT, &imports);

        let mut functions = Vec::new();
        push_test_u32(&mut functions, 1);
        push_test_u32(&mut functions, 1);
        push_test_section(&mut module, SECTION_FUNCTION, &functions);

        if needs_memory {
            push_test_section(&mut module, SECTION_MEMORY, &[0x01, 0x00, 0x01]);
        }

        let mut exports = Vec::new();
        push_test_u32(&mut exports, 1);
        push_test_name(&mut exports, b"_start");
        exports.push(EXTERNAL_KIND_FUNC);
        push_test_u32(&mut exports, 1);
        push_test_section(&mut module, SECTION_EXPORT, &exports);

        let mut body = Vec::new();
        push_test_u32(&mut body, 0);
        for arg in args {
            match *arg {
                TestWasmArg::I32(value) => {
                    body.push(OPCODE_I32_CONST);
                    push_test_i32(&mut body, value);
                }
                TestWasmArg::I64(value) => {
                    body.push(OPCODE_I64_CONST);
                    push_test_i64(&mut body, value);
                }
            }
        }
        body.push(OPCODE_CALL);
        push_test_u32(&mut body, 0);
        if !import_results.is_empty() {
            body.push(OPCODE_DROP);
        }
        body.push(OPCODE_END);

        let mut code = Vec::new();
        push_test_u32(&mut code, 1);
        push_test_u32(&mut code, body.len() as u32);
        code.extend_from_slice(&body);
        push_test_section(&mut module, SECTION_CODE, &code);
        module
    }

    fn core_test_module(
        body_instrs: &[u8],
        memory: bool,
        table_min: Option<u32>,
        data_section: Option<&[u8]>,
        element_section: Option<&[u8]>,
    ) -> Vec<u8> {
        let mut module = Vec::new();
        module.extend_from_slice(&[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);

        let mut types = Vec::new();
        push_test_u32(&mut types, 1);
        types.push(0x60);
        push_test_u32(&mut types, 0);
        push_test_u32(&mut types, 0);
        push_test_section(&mut module, SECTION_TYPE, &types);

        let mut functions = Vec::new();
        push_test_u32(&mut functions, 1);
        push_test_u32(&mut functions, 0);
        push_test_section(&mut module, SECTION_FUNCTION, &functions);

        if let Some(min) = table_min {
            let mut table = Vec::new();
            push_test_u32(&mut table, 1);
            table.push(super::VALTYPE_FUNCREF);
            table.push(0x00);
            push_test_u32(&mut table, min);
            push_test_section(&mut module, super::SECTION_TABLE, &table);
        }

        if memory {
            push_test_section(&mut module, SECTION_MEMORY, &[0x01, 0x00, 0x01]);
        }

        let mut exports = Vec::new();
        push_test_u32(&mut exports, 1);
        push_test_name(&mut exports, b"_start");
        exports.push(EXTERNAL_KIND_FUNC);
        push_test_u32(&mut exports, 0);
        push_test_section(&mut module, SECTION_EXPORT, &exports);

        if let Some(elements) = element_section {
            push_test_section(&mut module, super::SECTION_ELEMENT, elements);
        }

        let mut code = Vec::new();
        let mut body = Vec::new();
        push_test_u32(&mut body, 0);
        body.extend_from_slice(body_instrs);
        body.push(OPCODE_END);
        push_test_u32(&mut code, 1);
        push_test_u32(&mut code, body.len() as u32);
        code.extend_from_slice(&body);
        push_test_section(&mut module, SECTION_CODE, &code);

        if let Some(data) = data_section {
            push_test_section(&mut module, super::SECTION_DATA, data);
        }

        module
    }

    #[test]
    fn demo_wasm_module_parses() {
        let _module = TinyWasmModule::parse(DEMO_WASM_GUEST).expect("parse demo wasm");
    }

    #[test]
    fn core_wasm_engine_surfaces_imports_without_wasi_authority() {
        static CORE_IMPORT_GUEST: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x23, 0x01, 0x16, b'w', b'a', b's', b'i', b'_', b's',
            b'n', b'a', b'p', b's', b'h', b'o', b't', b'_', b'p', b'r', b'e', b'v', b'i', b'e',
            b'w', b'1', 0x08, b'f', b'd', b'_', b'w', b'r', b'i', b't', b'e', 0x00, 0x00, 0x03,
            0x02, 0x01, 0x01, 0x07, 0x0a, 0x01, 0x06, b'_', b's', b't', b'a', b'r', b't', 0x00,
            0x01, 0x0a, 0x08, 0x01, 0x06, 0x00, 0x41, 0x07, 0x10, 0x00, 0x0b,
        ];
        let mut core = CoreWasmInstance::new(CORE_IMPORT_GUEST).expect("instantiate core wasm");

        let CoreWasmTrap::HostImport(import) = core.resume().expect("resume to generic import")
        else {
            panic!("expected generic host import trap");
        };
        assert_eq!(import.import.module, b"wasi_snapshot_preview1");
        assert_eq!(import.import.name, b"fd_write");
        assert_eq!(import.args(), &[CoreWasmValue::I32(7)]);
        core.complete_host_import(&[])
            .expect("complete generic import without syscall handling");
        assert_eq!(core.resume().expect("resume to done"), CoreWasmTrap::Done);
    }

    #[test]
    fn core_wasm_memory_grow_is_generic_engine_event_not_lease_policy() {
        static CORE_MEMORY_GROW_GUEST: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x05, 0x04, 0x01, 0x01, 0x01, 0x02, 0x07, 0x0a, 0x01, 0x06,
            b'_', b's', b't', b'a', b'r', b't', 0x00, 0x00, 0x0a, 0x09, 0x01, 0x07, 0x00, 0x41,
            0x01, 0x40, 0x00, 0x1a, 0x0b,
        ];
        let mut core =
            CoreWasmInstance::new(CORE_MEMORY_GROW_GUEST).expect("instantiate core wasm");
        assert_eq!(core.memory_pages(), 1);

        let CoreWasmTrap::MemoryGrow(event) = core.resume().expect("resume to memory.grow event")
        else {
            panic!("expected memory.grow core event");
        };
        assert_eq!(event.previous_pages, 1);
        assert_eq!(event.requested_pages, 1);
        assert_eq!(event.new_pages, Some(2));
        assert_eq!(core.memory_pages(), 2);
        assert_eq!(
            core.complete_memory_grow_event()
                .expect("host observes memory.grow"),
            event
        );
        assert_eq!(core.resume().expect("resume to done"), CoreWasmTrap::Done);
    }

    #[test]
    fn core_wasm_engine_runs_local_function_calls_without_syscall_features() {
        static CORE_LOCAL_CALL_GUEST: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x00, 0x01,
            0x7f, 0x60, 0x00, 0x00, 0x03, 0x03, 0x02, 0x00, 0x01, 0x07, 0x0a, 0x01, 0x06, b'_',
            b's', b't', b'a', b'r', b't', 0x00, 0x01, 0x0a, 0x0c, 0x02, 0x04, 0x00, 0x41, 0x2a,
            0x0b, 0x05, 0x00, 0x10, 0x00, 0x1a, 0x0b,
        ];
        let mut core =
            CoreWasmInstance::new(CORE_LOCAL_CALL_GUEST).expect("instantiate local-call core wasm");

        assert_eq!(
            core.resume().expect("local call reaches done"),
            CoreWasmTrap::Done
        );
    }

    #[test]
    fn core_wasm_engine_executes_if_else_and_block_results() {
        let mut body = Vec::new();
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 1);
        body.push(super::OPCODE_IF);
        body.push(VALTYPE_I32);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 42);
        body.push(super::OPCODE_ELSE);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 7);
        body.push(OPCODE_END);
        body.push(super::OPCODE_I32_STORE);
        push_test_u32(&mut body, 2);
        push_test_u32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 4);
        body.push(super::OPCODE_BLOCK);
        body.push(VALTYPE_I32);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 9);
        body.push(OPCODE_END);
        body.push(super::OPCODE_I32_STORE);
        push_test_u32(&mut body, 2);
        push_test_u32(&mut body, 0);

        let module = core_test_module(&body, true, None, None, None);
        let mut core = CoreWasmInstance::new(&module).expect("instantiate if/block core wasm");
        assert_eq!(
            core.resume().expect("if/block reaches done"),
            CoreWasmTrap::Done
        );
        assert_eq!(core.read_memory_u32(0).expect("if result"), 42);
        assert_eq!(core.read_memory_u32(4).expect("block result"), 9);
    }

    #[test]
    fn core_wasm_engine_executes_sign_extension_ops() {
        let mut body = Vec::new();
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0x80);
        body.push(super::OPCODE_I32_EXTEND8_S);
        body.push(super::OPCODE_I32_STORE);
        push_test_u32(&mut body, 2);
        push_test_u32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 8);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0x8000);
        body.push(super::OPCODE_I64_EXTEND_I32_U);
        body.push(super::OPCODE_I64_EXTEND16_S);
        body.push(super::OPCODE_I64_STORE);
        push_test_u32(&mut body, 3);
        push_test_u32(&mut body, 0);

        let module = core_test_module(&body, true, None, None, None);
        let mut core = CoreWasmInstance::new(&module).expect("instantiate sign-extension wasm");
        assert_eq!(
            core.resume().expect("sign-extension reaches done"),
            CoreWasmTrap::Done
        );
        assert_eq!(core.read_memory_u32(0).expect("i32.extend8_s"), 0xffff_ff80);
        let mut out = [0u8; 8];
        core.read_memory(8, &mut out).expect("i64.extend16_s");
        assert_eq!(u64::from_le_bytes(out), 0xffff_ffff_ffff_8000);
    }

    #[test]
    fn core_wasm_engine_executes_passive_data_memory_init() {
        let mut data = Vec::new();
        push_test_u32(&mut data, 1);
        push_test_u32(&mut data, 1);
        push_test_u32(&mut data, 6);
        data.extend_from_slice(b"hibana");

        let mut body = Vec::new();
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 16);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 1);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 3);
        body.push(super::OPCODE_MISC);
        push_test_u32(&mut body, 8);
        push_test_u32(&mut body, 0);
        body.push(0x00);
        body.push(super::OPCODE_MISC);
        push_test_u32(&mut body, 9);
        push_test_u32(&mut body, 0);

        let module = core_test_module(&body, true, None, Some(&data), None);
        let mut core = CoreWasmInstance::new(&module).expect("instantiate passive data wasm");
        assert_eq!(
            core.resume().expect("passive data reaches done"),
            CoreWasmTrap::Done
        );
        let mut out = [0u8; 3];
        core.read_memory(16, &mut out).expect("memory.init bytes");
        assert_eq!(&out, b"iba");
    }

    #[test]
    fn core_wasm_engine_executes_float_basics() {
        let mut body = Vec::new();
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(super::OPCODE_F32_CONST);
        body.extend_from_slice(&1.5f32.to_bits().to_le_bytes());
        body.push(super::OPCODE_F32_CONST);
        body.extend_from_slice(&2.25f32.to_bits().to_le_bytes());
        body.push(super::OPCODE_F32_ADD);
        body.push(super::OPCODE_I32_REINTERPRET_F32);
        body.push(super::OPCODE_I32_STORE);
        push_test_u32(&mut body, 2);
        push_test_u32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 8);
        body.push(super::OPCODE_F64_CONST);
        body.extend_from_slice(&4.0f64.to_bits().to_le_bytes());
        body.push(super::OPCODE_F64_SQRT);
        body.push(super::OPCODE_I64_REINTERPRET_F64);
        body.push(super::OPCODE_I64_STORE);
        push_test_u32(&mut body, 3);
        push_test_u32(&mut body, 0);

        let module = core_test_module(&body, true, None, None, None);
        let mut core = CoreWasmInstance::new(&module).expect("instantiate float wasm");
        assert_eq!(
            core.resume().expect("float reaches done"),
            CoreWasmTrap::Done
        );
        assert_eq!(
            f32::from_bits(core.read_memory_u32(0).expect("f32 add")),
            3.75
        );
        let mut out = [0u8; 8];
        core.read_memory(8, &mut out).expect("f64 sqrt");
        assert_eq!(f64::from_bits(u64::from_le_bytes(out)), 2.0);
    }

    #[test]
    fn core_wasm_engine_executes_table_ref_basics() {
        let mut body = Vec::new();
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(super::OPCODE_REF_FUNC);
        push_test_u32(&mut body, 0);
        body.push(super::OPCODE_TABLE_SET);
        push_test_u32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 0);
        body.push(super::OPCODE_TABLE_GET);
        push_test_u32(&mut body, 0);
        body.push(super::OPCODE_REF_IS_NULL);
        body.push(super::OPCODE_I32_STORE);
        push_test_u32(&mut body, 2);
        push_test_u32(&mut body, 0);
        body.push(super::OPCODE_REF_NULL);
        body.push(super::VALTYPE_FUNCREF);
        body.push(OPCODE_I32_CONST);
        push_test_i32(&mut body, 1);
        body.push(super::OPCODE_MISC);
        push_test_u32(&mut body, 15);
        push_test_u32(&mut body, 0);
        body.push(OPCODE_DROP);

        let module = core_test_module(&body, true, Some(1), None, None);
        let mut core = CoreWasmInstance::new(&module).expect("instantiate table/ref wasm");
        assert_eq!(
            core.resume().expect("table/ref reaches done"),
            CoreWasmTrap::Done
        );
        assert_eq!(core.read_memory_u32(0).expect("ref.is_null"), 0);
    }

    #[test]
    fn core_wasip1_trampoline_maps_fd_write_only_when_handler_is_enabled() {
        static CORE_WASIP1_FD_WRITE_GUEST: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x0c, 0x02, 0x60, 0x04, 0x7f,
            0x7f, 0x7f, 0x7f, 0x01, 0x7f, 0x60, 0x00, 0x00, 0x02, 0x23, 0x01, 0x16, b'w', b'a',
            b's', b'i', b'_', b's', b'n', b'a', b'p', b's', b'h', b'o', b't', b'_', b'p', b'r',
            b'e', b'v', b'i', b'e', b'w', b'1', 0x08, b'f', b'd', b'_', b'w', b'r', b'i', b't',
            b'e', 0x00, 0x00, 0x03, 0x02, 0x01, 0x01, 0x05, 0x03, 0x01, 0x00, 0x01, 0x07, 0x0a,
            0x01, 0x06, b'_', b's', b't', b'a', b'r', b't', 0x00, 0x01, 0x0a, 0x16, 0x01, 0x14,
            0x00, 0x41, 0x00, 0x41, 0x31, 0x3a, 0x00, 0x00, 0x41, 0x03, 0x41, 0x00, 0x41, 0x00,
            0x41, 0x01, 0x10, 0x00, 0x1a, 0x0b,
        ];

        assert!(matches!(
            CoreWasip1Instance::new(CORE_WASIP1_FD_WRITE_GUEST, Wasip1HandlerSet::EMPTY),
            Err(WasmError::Unsupported(
                "wasip1 fd_write disabled by feature profile"
            ))
        ));

        let mut guest =
            CoreWasip1Instance::new(CORE_WASIP1_FD_WRITE_GUEST, Wasip1HandlerSet::PICO_MIN)
                .expect("instantiate core wasip1 fd_write guest");
        let CoreWasip1Trap::FdWrite(write) = guest.resume().expect("fd_write trampoline trap")
        else {
            panic!("expected fd_write trap");
        };
        assert_eq!(write.fd(), 3);
        assert_eq!(
            guest
                .fd_write_payload(write)
                .expect("payload comes from core memory")
                .as_bytes(),
            b"1"
        );
        guest.complete_host_call(0).expect("return errno to core");
        assert_eq!(
            guest.resume().expect("done after fd_write"),
            CoreWasip1Trap::Done
        );

        let mut slot = core::mem::MaybeUninit::uninit();
        let guest = CoreWasip1Instance::write_new_in_place(
            CORE_WASIP1_FD_WRITE_GUEST,
            Wasip1HandlerSet::PICO_MIN,
            &mut slot,
        )
        .expect("instantiate core wasip1 fd_write guest in place");
        let CoreWasip1Trap::FdWrite(write) =
            guest.resume().expect("in-place fd_write trampoline trap")
        else {
            panic!("expected fd_write trap from in-place guest");
        };
        assert_eq!(write.fd(), 3);
    }

    #[test]
    fn core_wasip1_trampoline_maps_full_feature_syscall_surface() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                {
                    let fd_read = core_wasip1_single_import_module(
                        b"fd_read",
                        &[VALTYPE_I32, VALTYPE_I32, VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(16),
                            TestWasmArg::I32(1),
                            TestWasmArg::I32(40),
                        ],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&fd_read, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 fd_read disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&fd_read, Wasip1HandlerSet::FULL)
                        .expect("fd_read full");
                    let CoreWasip1Trap::FdRead(read) = guest.resume().expect("fd_read trap") else {
                        panic!("expected fd_read");
                    };
                    assert_eq!(read.fd(), 0);
                    assert_eq!(read.iovs(), 16);
                    assert_eq!(read.iovs_len(), 1);
                    assert_eq!(read.nread(), 40);
                    guest.complete_host_call(0).expect("complete fd_read errno");
                }

                {
                    let fdstat = core_wasip1_single_import_module(
                        b"fd_fdstat_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(1), TestWasmArg::I32(80)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&fdstat, Wasip1HandlerSet::FULL)
                        .expect("fdstat full");
                    let CoreWasip1Trap::FdFdstatGet(stat) = guest.resume().expect("fdstat trap")
                    else {
                        panic!("expected fd_fdstat_get");
                    };
                    assert_eq!(stat.fd(), 1);
                    assert_eq!(stat.out_ptr(), 80);
                    guest
                        .complete_fd_fdstat_get(
                            stat,
                            CoreWasip1FdStat::new(WASIP1_FILETYPE_REGULAR_FILE, 0, 0b11, 0),
                            0,
                        )
                        .expect("complete fdstat");
                    let mut rights = [0u8; 8];
                    guest
                        .read_memory(80 + WASIP1_FDSTAT_RIGHTS_BASE_OFFSET, &mut rights)
                        .expect("fdstat rights memory");
                    assert_eq!(u64::from_le_bytes(rights), 0b11);
                }

                {
                    let fd_close = core_wasip1_single_import_module(
                        b"fd_close",
                        &[VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(7)],
                        false,
                    );
                    let mut guest = CoreWasip1Instance::new(&fd_close, Wasip1HandlerSet::FULL)
                        .expect("fd_close full");
                    let CoreWasip1Trap::FdClose(close) = guest.resume().expect("fd_close trap")
                    else {
                        panic!("expected fd_close");
                    };
                    assert_eq!(close.fd(), 7);
                    guest.complete_host_call(0).expect("complete fd_close");
                }

                {
                    let fd_prestat_get = core_wasip1_single_import_module(
                        b"fd_prestat_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(3), TestWasmArg::I32(128)],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&fd_prestat_get, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 path-minimal disabled by feature profile"
                        ))
                    ));
                    let mut guest =
                        CoreWasip1Instance::new(&fd_prestat_get, Wasip1HandlerSet::FULL)
                            .expect("fd_prestat_get full");
                    let CoreWasip1Trap::PathMinimal(path) =
                        guest.resume().expect("fd_prestat_get trap")
                    else {
                        panic!("expected path-minimal trap");
                    };
                    assert_eq!(path.kind(), CoreWasip1PathKind::FdPrestatGet);
                    assert_eq!(
                        path.args(),
                        &[CoreWasmValue::I32(3), CoreWasmValue::I32(128)]
                    );
                    guest
                        .complete_path_minimal(path, 52)
                        .expect("complete fd_prestat_get as ENOSYS");
                    assert_eq!(
                        guest.resume().expect("done after fd_prestat_get"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let path_open = core_wasip1_single_import_module(
                        b"path_open",
                        &[
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I64,
                            VALTYPE_I64,
                            VALTYPE_I32,
                            VALTYPE_I32,
                        ],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(3),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(160),
                            TestWasmArg::I32(4),
                            TestWasmArg::I32(0),
                            TestWasmArg::I64(1),
                            TestWasmArg::I64(1),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(196),
                        ],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&path_open, Wasip1HandlerSet::FULL)
                        .expect("path_open full");
                    let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("path_open trap")
                    else {
                        panic!("expected path-minimal path_open trap");
                    };
                    assert_eq!(path.kind(), CoreWasip1PathKind::PathOpen);
                    assert_eq!(path.args().len(), 9);
                    assert_eq!(path.args()[5], CoreWasmValue::I64(1));
                    guest
                        .complete_path_minimal(path, 52)
                        .expect("complete path_open as ENOSYS");
                    assert_eq!(
                        guest.resume().expect("done after path_open"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let fd_seek = core_wasip1_single_import_module(
                        b"fd_seek",
                        &[VALTYPE_I32, VALTYPE_I64, VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(3),
                            TestWasmArg::I64(42),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(224),
                        ],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&fd_seek, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 path-full disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&fd_seek, Wasip1HandlerSet::FULL)
                        .expect("fd_seek full");
                    let CoreWasip1Trap::PathFull(path) = guest.resume().expect("fd_seek trap")
                    else {
                        panic!("expected path-full fd_seek trap");
                    };
                    assert_eq!(path.kind(), CoreWasip1PathKind::FdSeek);
                    assert_eq!(
                        path.args(),
                        &[
                            CoreWasmValue::I32(3),
                            CoreWasmValue::I64(42),
                            CoreWasmValue::I32(0),
                            CoreWasmValue::I32(224),
                        ]
                    );
                    guest
                        .complete_path_full(path, 52)
                        .expect("complete fd_seek as ENOSYS");
                    assert_eq!(
                        guest.resume().expect("done after fd_seek"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let fd_fdstat_set_rights = core_wasip1_single_import_module(
                        b"fd_fdstat_set_rights",
                        &[VALTYPE_I32, VALTYPE_I64, VALTYPE_I64],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(3),
                            TestWasmArg::I64(0x10),
                            TestWasmArg::I64(0x20),
                        ],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&fd_fdstat_set_rights, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 path-full disabled by feature profile"
                        ))
                    ));
                    let mut guest =
                        CoreWasip1Instance::new(&fd_fdstat_set_rights, Wasip1HandlerSet::FULL)
                            .expect("fd_fdstat_set_rights full");
                    let CoreWasip1Trap::PathFull(path) =
                        guest.resume().expect("fd_fdstat_set_rights trap")
                    else {
                        panic!("expected path-full fd_fdstat_set_rights trap");
                    };
                    assert_eq!(path.kind(), CoreWasip1PathKind::FdFdstatSetRights);
                    assert_eq!(
                        path.args(),
                        &[
                            CoreWasmValue::I32(3),
                            CoreWasmValue::I64(0x10),
                            CoreWasmValue::I64(0x20),
                        ]
                    );
                    guest
                        .complete_path_full(path, 52)
                        .expect("complete fd_fdstat_set_rights as ENOSYS");
                    assert_eq!(
                        guest.resume().expect("done after fd_fdstat_set_rights"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let path_link = core_wasip1_single_import_module(
                        b"path_link",
                        &[
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                        ],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(3),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(240),
                            TestWasmArg::I32(4),
                            TestWasmArg::I32(5),
                            TestWasmArg::I32(264),
                            TestWasmArg::I32(6),
                        ],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&path_link, Wasip1HandlerSet::FULL)
                        .expect("path_link full");
                    let CoreWasip1Trap::PathFull(path) = guest.resume().expect("path_link trap")
                    else {
                        panic!("expected path-full path_link trap");
                    };
                    assert_eq!(path.kind(), CoreWasip1PathKind::PathLink);
                    assert_eq!(path.args().len(), 7);
                    guest
                        .complete_path_full(path, 52)
                        .expect("complete path_link as ENOSYS");
                }

                {
                    let sock_send = core_wasip1_single_import_module(
                        b"sock_send",
                        &[
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                        ],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(30),
                            TestWasmArg::I32(128),
                            TestWasmArg::I32(1),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(152),
                        ],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&sock_send, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 NetworkObject imports disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&sock_send, Wasip1HandlerSet::FULL)
                        .expect("sock_send full");
                    let CoreWasip1Trap::Socket(sock) = guest.resume().expect("sock_send trap")
                    else {
                        panic!("expected socket sock_send trap");
                    };
                    assert_eq!(sock.kind(), CoreWasip1SocketKind::SockSend);
                    assert_eq!(
                        sock.args(),
                        &[
                            CoreWasmValue::I32(30),
                            CoreWasmValue::I32(128),
                            CoreWasmValue::I32(1),
                            CoreWasmValue::I32(0),
                            CoreWasmValue::I32(152),
                        ]
                    );
                    assert_eq!(
                        guest
                            .sock_send_payload(sock)
                            .expect("sock_send payload")
                            .as_bytes(),
                        b""
                    );
                    assert_eq!(
                        guest
                            .socket_as_engine_req(sock, 8)
                            .expect("sock_send as fd_write"),
                        EngineReq::FdWrite(FdWrite::new_with_lease(30, 8, b"").expect("fd_write"))
                    );
                    guest
                        .complete_sock_send(sock, 4, 0)
                        .expect("complete sock_send");
                    assert_eq!(guest.read_memory_u32(152).expect("sock_send nwritten"), 4);
                    assert_eq!(
                        guest.resume().expect("done after sock_send"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let sock_recv = core_wasip1_single_import_module(
                        b"sock_recv",
                        &[
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                            VALTYPE_I32,
                        ],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(31),
                            TestWasmArg::I32(160),
                            TestWasmArg::I32(1),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(184),
                            TestWasmArg::I32(188),
                        ],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&sock_recv, Wasip1HandlerSet::FULL)
                        .expect("sock_recv full");
                    let CoreWasip1Trap::Socket(sock) = guest.resume().expect("sock_recv trap")
                    else {
                        panic!("expected socket sock_recv trap");
                    };
                    assert_eq!(sock.kind(), CoreWasip1SocketKind::SockRecv);
                    assert_eq!(sock.args().len(), 6);
                    assert_eq!(
                        guest
                            .socket_as_engine_req(sock, 9)
                            .expect("sock_recv as fd_read"),
                        EngineReq::FdRead(FdRead::new_with_lease(31, 9, 0).expect("fd_read"))
                    );
                    guest
                        .complete_sock_recv(sock, b"", 2, 0)
                        .expect("complete sock_recv");
                    assert_eq!(guest.read_memory_u32(184).expect("sock_recv nread"), 0);
                    assert_eq!(guest.read_memory_u32(188).expect("sock_recv flags"), 2);
                }

                {
                    let sock_accept = core_wasip1_single_import_module(
                        b"sock_accept",
                        &[VALTYPE_I32, VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(32),
                            TestWasmArg::I32(0),
                            TestWasmArg::I32(196),
                        ],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&sock_accept, Wasip1HandlerSet::FULL)
                        .expect("sock_accept full");
                    let CoreWasip1Trap::Socket(sock) = guest.resume().expect("sock_accept trap")
                    else {
                        panic!("expected socket sock_accept trap");
                    };
                    assert_eq!(sock.kind(), CoreWasip1SocketKind::SockAccept);
                    assert_eq!(sock.args().len(), 3);
                    assert!(matches!(
                        guest.socket_as_engine_req(sock, 0),
                        Err(WasmError::Unsupported(
                            "sock_accept requires explicit network accept route"
                        ))
                    ));
                    guest
                        .complete_sock_accept(sock, 44, 0)
                        .expect("complete sock_accept");
                    assert_eq!(guest.read_memory_u32(196).expect("accepted fd"), 44);
                }

                {
                    let sock_shutdown = core_wasip1_single_import_module(
                        b"sock_shutdown",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(33), TestWasmArg::I32(3)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&sock_shutdown, Wasip1HandlerSet::FULL)
                        .expect("sock_shutdown full");
                    let CoreWasip1Trap::Socket(sock) = guest.resume().expect("sock_shutdown trap")
                    else {
                        panic!("expected socket sock_shutdown trap");
                    };
                    assert_eq!(sock.kind(), CoreWasip1SocketKind::SockShutdown);
                    assert_eq!(
                        sock.args(),
                        &[CoreWasmValue::I32(33), CoreWasmValue::I32(3)]
                    );
                    assert_eq!(
                        guest
                            .socket_as_engine_req(sock, 0)
                            .expect("sock_shutdown as fd_close"),
                        EngineReq::FdClose(FdRequest::new(33))
                    );
                    assert!(matches!(
                        guest.complete_socket(sock, 0),
                        Err(WasmError::Invalid(
                            "socket success requires typed socket completion"
                        ))
                    ));
                    guest
                        .complete_sock_shutdown(sock, 0)
                        .expect("complete sock_shutdown");
                }

                {
                    let clock_res = core_wasip1_single_import_module(
                        b"clock_res_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(1), TestWasmArg::I32(88)],
                        true,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&clock_res, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 clock_res_get disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&clock_res, Wasip1HandlerSet::FULL)
                        .expect("clock_res_get full");
                    let CoreWasip1Trap::ClockResGet(clock) =
                        guest.resume().expect("clock_res_get trap")
                    else {
                        panic!("expected clock_res_get");
                    };
                    assert_eq!(clock.clock_id(), 1);
                    assert_eq!(clock.resolution_ptr(), 88);
                    guest
                        .complete_clock_res_get(clock, 1_000_000, 0)
                        .expect("complete clock_res_get");
                    let mut resolution = [0u8; 8];
                    guest
                        .read_memory(88, &mut resolution)
                        .expect("read clock resolution result");
                    assert_eq!(u64::from_le_bytes(resolution), 1_000_000);
                }

                {
                    let clock = core_wasip1_single_import_module(
                        b"clock_time_get",
                        &[VALTYPE_I32, VALTYPE_I64, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[
                            TestWasmArg::I32(1),
                            TestWasmArg::I64(1_000),
                            TestWasmArg::I32(96),
                        ],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&clock, Wasip1HandlerSet::FULL)
                        .expect("clock full");
                    let CoreWasip1Trap::ClockTimeGet(clock) = guest.resume().expect("clock trap")
                    else {
                        panic!("expected clock_time_get");
                    };
                    assert_eq!(clock.clock_id(), 1);
                    assert_eq!(clock.precision(), 1_000);
                    assert_eq!(clock.time_ptr(), 96);
                    guest
                        .complete_clock_time_get(clock, 123_456_789, 0)
                        .expect("complete clock");
                    let mut nanos = [0u8; 8];
                    guest
                        .read_memory(96, &mut nanos)
                        .expect("read clock result");
                    assert_eq!(u64::from_le_bytes(nanos), 123_456_789);
                }

                {
                    let random = core_wasip1_single_import_module(
                        b"random_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(112), TestWasmArg::I32(4)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&random, Wasip1HandlerSet::FULL)
                        .expect("random full");
                    let CoreWasip1Trap::RandomGet(random) = guest.resume().expect("random trap")
                    else {
                        panic!("expected random_get");
                    };
                    assert_eq!(random.buf(), 112);
                    assert_eq!(random.buf_len(), 4);
                    guest
                        .complete_random_get(random, b"RAND", 0)
                        .expect("complete random");
                    let mut random_out = [0u8; 4];
                    guest
                        .read_memory(112, &mut random_out)
                        .expect("read random result");
                    assert_eq!(&random_out, b"RAND");
                }

                {
                    let sched_yield = core_wasip1_single_import_module(
                        b"sched_yield",
                        &[],
                        &[VALTYPE_I32],
                        &[],
                        false,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&sched_yield, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 sched_yield disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&sched_yield, Wasip1HandlerSet::FULL)
                        .expect("sched_yield full");
                    assert_eq!(
                        guest.resume().expect("sched_yield trap"),
                        CoreWasip1Trap::SchedYield
                    );
                    guest.complete_sched_yield(0).expect("complete sched_yield");
                    assert_eq!(
                        guest.resume().expect("done after sched_yield"),
                        CoreWasip1Trap::Done
                    );
                }

                {
                    let proc_raise = core_wasip1_single_import_module(
                        b"proc_raise",
                        &[VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(9)],
                        false,
                    );
                    assert!(matches!(
                        CoreWasip1Instance::new(&proc_raise, Wasip1HandlerSet::PICO_MIN),
                        Err(WasmError::Unsupported(
                            "wasip1 proc_raise disabled by feature profile"
                        ))
                    ));
                    let mut guest = CoreWasip1Instance::new(&proc_raise, Wasip1HandlerSet::FULL)
                        .expect("proc_raise full");
                    assert_eq!(
                        guest.resume().expect("proc_raise trap"),
                        CoreWasip1Trap::ProcRaise(9)
                    );
                    guest
                        .complete_proc_raise(52)
                        .expect("complete proc_raise as ENOSYS");
                    assert_eq!(
                        guest.resume().expect("done after proc_raise"),
                        CoreWasip1Trap::Done
                    );
                }
            })
            .expect("spawn wasm test")
            .join()
            .expect("wasm test joins");
    }

    #[test]
    fn core_wasip1_path_helpers_write_meaningful_choreofs_results() {
        let path_open = core_wasip1_single_import_module(
            b"path_open",
            &[
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I64,
                VALTYPE_I64,
                VALTYPE_I32,
                VALTYPE_I32,
            ],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(3),
                TestWasmArg::I32(0),
                TestWasmArg::I32(160),
                TestWasmArg::I32(10),
                TestWasmArg::I32(0),
                TestWasmArg::I64(1),
                TestWasmArg::I64(1),
                TestWasmArg::I32(0),
                TestWasmArg::I32(196),
            ],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&path_open, Wasip1HandlerSet::FULL).expect("path_open full");
        guest
            .write_memory(160, b"app/config")
            .expect("write path bytes into guest memory");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("path_open trap") else {
            panic!("expected path_open trap");
        };
        assert_eq!(path.kind(), CoreWasip1PathKind::PathOpen);
        assert_eq!(
            guest.path_bytes(path).expect("read path bytes").as_bytes(),
            b"app/config"
        );
        guest
            .complete_path_open(path, 44, 0)
            .expect("complete path_open with minted fd");
        assert_eq!(guest.read_memory_u32(196).expect("opened fd"), 44);
        assert_eq!(
            guest.resume().expect("done after path_open"),
            CoreWasip1Trap::Done
        );

        let fd_readdir = core_wasip1_single_import_module(
            b"fd_readdir",
            &[
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I64,
                VALTYPE_I32,
            ],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(44),
                TestWasmArg::I32(224),
                TestWasmArg::I32(32),
                TestWasmArg::I64(0),
                TestWasmArg::I32(260),
            ],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&fd_readdir, Wasip1HandlerSet::FULL).expect("readdir full");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("fd_readdir trap") else {
            panic!("expected fd_readdir trap");
        };
        assert_eq!(path.kind(), CoreWasip1PathKind::FdReaddir);
        guest
            .complete_fd_readdir(path, b"config\nstate\n", 0)
            .expect("complete fd_readdir with manifest bytes");
        let mut bytes = [0u8; 13];
        guest
            .read_memory(224, &mut bytes)
            .expect("read fd_readdir bytes");
        assert_eq!(&bytes, b"config\nstate\n");
        assert_eq!(guest.read_memory_u32(260).expect("bufused"), 13);
        assert_eq!(
            guest.resume().expect("done after fd_readdir"),
            CoreWasip1Trap::Done
        );
    }

    #[test]
    fn core_wasip1_path_helpers_write_std_prestat_and_filestat_results() {
        let fd_prestat_get = core_wasip1_single_import_module(
            b"fd_prestat_get",
            &[VALTYPE_I32, VALTYPE_I32],
            &[VALTYPE_I32],
            &[TestWasmArg::I32(3), TestWasmArg::I32(96)],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&fd_prestat_get, Wasip1HandlerSet::FULL).expect("prestat");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("prestat trap") else {
            panic!("expected fd_prestat_get");
        };
        guest
            .complete_fd_prestat_get(path, 3, 0)
            .expect("complete prestat");
        assert_eq!(
            guest
                .read_memory_u32(96 + WASIP1_PRESTAT_DIR_NAME_LEN_OFFSET)
                .unwrap(),
            3
        );

        let fd_prestat_dir_name = core_wasip1_single_import_module(
            b"fd_prestat_dir_name",
            &[VALTYPE_I32, VALTYPE_I32, VALTYPE_I32],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(3),
                TestWasmArg::I32(128),
                TestWasmArg::I32(8),
            ],
            true,
        );
        let mut guest = CoreWasip1Instance::new(&fd_prestat_dir_name, Wasip1HandlerSet::FULL)
            .expect("prestat dir name");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("prestat name trap") else {
            panic!("expected fd_prestat_dir_name");
        };
        guest
            .complete_fd_prestat_dir_name(path, b"app", 0)
            .expect("complete prestat dir name");
        let mut name = [0u8; 3];
        guest.read_memory(128, &mut name).expect("read name");
        assert_eq!(&name, b"app");

        let fd_filestat_get = core_wasip1_single_import_module(
            b"fd_filestat_get",
            &[VALTYPE_I32, VALTYPE_I32],
            &[VALTYPE_I32],
            &[TestWasmArg::I32(4), TestWasmArg::I32(160)],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&fd_filestat_get, Wasip1HandlerSet::FULL).expect("fd filestat");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("fd filestat trap") else {
            panic!("expected fd_filestat_get");
        };
        guest
            .complete_fd_filestat_get(
                path,
                CoreWasip1FileStat::new(WASIP1_FILETYPE_REGULAR_FILE, 42),
                0,
            )
            .expect("complete fd filestat");
        let mut filetype = [0u8; 1];
        guest
            .read_memory(160 + WASIP1_FILESTAT_FILETYPE_OFFSET, &mut filetype)
            .expect("read filetype");
        assert_eq!(filetype[0], WASIP1_FILETYPE_REGULAR_FILE);
        let mut size = [0u8; 8];
        guest
            .read_memory(160 + WASIP1_FILESTAT_SIZE_OFFSET, &mut size)
            .expect("read size");
        assert_eq!(u64::from_le_bytes(size), 42);

        let path_filestat_get = core_wasip1_single_import_module(
            b"path_filestat_get",
            &[
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
            ],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(3),
                TestWasmArg::I32(0),
                TestWasmArg::I32(192),
                TestWasmArg::I32(3),
                TestWasmArg::I32(224),
            ],
            true,
        );
        let mut guest = CoreWasip1Instance::new(&path_filestat_get, Wasip1HandlerSet::FULL)
            .expect("path filestat");
        guest.write_memory(192, b"app").expect("path bytes");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("path filestat trap") else {
            panic!("expected path_filestat_get");
        };
        assert_eq!(
            guest.path_bytes(path).expect("path bytes").as_bytes(),
            b"app"
        );
        guest
            .complete_path_filestat_get(
                path,
                CoreWasip1FileStat::new(WASIP1_FILETYPE_DIRECTORY, 0),
                0,
            )
            .expect("complete path filestat");
        let mut filetype = [0u8; 1];
        guest
            .read_memory(224 + WASIP1_FILESTAT_FILETYPE_OFFSET, &mut filetype)
            .expect("read path filetype");
        assert_eq!(filetype[0], WASIP1_FILETYPE_DIRECTORY);

        let path_readlink = core_wasip1_single_import_module(
            b"path_readlink",
            &[
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
            ],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(3),
                TestWasmArg::I32(256),
                TestWasmArg::I32(4),
                TestWasmArg::I32(288),
                TestWasmArg::I32(16),
                TestWasmArg::I32(320),
            ],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&path_readlink, Wasip1HandlerSet::FULL).expect("readlink");
        guest.write_memory(256, b"link").expect("link path bytes");
        let CoreWasip1Trap::PathMinimal(path) = guest.resume().expect("readlink trap") else {
            panic!("expected path_readlink");
        };
        guest
            .complete_path_readlink(path, b"target", 0)
            .expect("complete readlink");
        let mut target = [0u8; 6];
        guest.read_memory(288, &mut target).expect("read target");
        assert_eq!(&target, b"target");
        assert_eq!(guest.read_memory_u32(320).expect("bufused"), 6);
    }

    #[test]
    fn core_wasip1_path_full_helpers_write_offset_and_iovec_results() {
        let fd_seek = core_wasip1_single_import_module(
            b"fd_seek",
            &[VALTYPE_I32, VALTYPE_I64, VALTYPE_I32, VALTYPE_I32],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(4),
                TestWasmArg::I64(12),
                TestWasmArg::I32(0),
                TestWasmArg::I32(96),
            ],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&fd_seek, Wasip1HandlerSet::FULL).expect("fd_seek full");
        let CoreWasip1Trap::PathFull(path) = guest.resume().expect("fd_seek trap") else {
            panic!("expected fd_seek trap");
        };
        guest
            .complete_fd_seek(path, 12, 0)
            .expect("complete fd_seek");
        let mut offset = [0u8; 8];
        guest
            .read_memory(96, &mut offset)
            .expect("read fd_seek offset");
        assert_eq!(u64::from_le_bytes(offset), 12);

        let fd_pread = core_wasip1_single_import_module(
            b"fd_pread",
            &[
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I32,
                VALTYPE_I64,
                VALTYPE_I32,
            ],
            &[VALTYPE_I32],
            &[
                TestWasmArg::I32(4),
                TestWasmArg::I32(128),
                TestWasmArg::I32(1),
                TestWasmArg::I64(0),
                TestWasmArg::I32(160),
            ],
            true,
        );
        let mut guest =
            CoreWasip1Instance::new(&fd_pread, Wasip1HandlerSet::FULL).expect("fd_pread full");
        guest
            .write_memory(128, &192u32.to_le_bytes())
            .expect("write iovec ptr");
        guest
            .write_memory(132, &8u32.to_le_bytes())
            .expect("write iovec len");
        let CoreWasip1Trap::PathFull(path) = guest.resume().expect("fd_pread trap") else {
            panic!("expected fd_pread trap");
        };
        guest
            .complete_fd_pread(path, b"hello", 0)
            .expect("complete fd_pread");
        let mut bytes = [0u8; 5];
        guest.read_memory(192, &mut bytes).expect("read fd_pread");
        assert_eq!(&bytes, b"hello");
        assert_eq!(guest.read_memory_u32(160).expect("nread"), 5);
    }

    #[test]
    fn core_wasip1_trampoline_maps_args_and_environment_imports() {
        std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(|| {
                {
                    let args_sizes = core_wasip1_single_import_module(
                        b"args_sizes_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(16), TestWasmArg::I32(20)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&args_sizes, Wasip1HandlerSet::FULL)
                        .expect("args sizes full");
                    let CoreWasip1Trap::ArgsSizesGet(call) =
                        guest.resume().expect("args sizes trap")
                    else {
                        panic!("expected args_sizes_get");
                    };
                    guest
                        .complete_args_sizes_get(call, 2, 12, 0)
                        .expect("complete args sizes");
                    assert_eq!(guest.read_memory_u32(16).expect("argc"), 2);
                    assert_eq!(guest.read_memory_u32(20).expect("argv size"), 12);
                }

                {
                    let args_get = core_wasip1_single_import_module(
                        b"args_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(32), TestWasmArg::I32(64)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&args_get, Wasip1HandlerSet::FULL)
                        .expect("args get full");
                    let CoreWasip1Trap::ArgsGet(call) = guest.resume().expect("args get trap")
                    else {
                        panic!("expected args_get");
                    };
                    guest
                        .complete_args_get(call, &[b"hibana", b"pico"], 0)
                        .expect("complete args get");
                    assert_eq!(guest.read_memory_u32(32).expect("argv0 ptr"), 64);
                    assert_eq!(guest.read_memory_u32(36).expect("argv1 ptr"), 71);
                    let mut arg_bytes = [0u8; 12];
                    guest
                        .read_memory(64, &mut arg_bytes)
                        .expect("read args bytes");
                    assert_eq!(&arg_bytes, b"hibana\0pico\0");
                }

                {
                    let env_sizes = core_wasip1_single_import_module(
                        b"environ_sizes_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(128), TestWasmArg::I32(132)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&env_sizes, Wasip1HandlerSet::FULL)
                        .expect("env sizes full");
                    let CoreWasip1Trap::EnvironSizesGet(call) =
                        guest.resume().expect("env sizes trap")
                    else {
                        panic!("expected environ_sizes_get");
                    };
                    guest
                        .complete_environ_sizes_get(call, 1, 10, 0)
                        .expect("complete env sizes");
                    assert_eq!(guest.read_memory_u32(128).expect("env count"), 1);
                    assert_eq!(guest.read_memory_u32(132).expect("env size"), 10);
                }

                {
                    let env_get = core_wasip1_single_import_module(
                        b"environ_get",
                        &[VALTYPE_I32, VALTYPE_I32],
                        &[VALTYPE_I32],
                        &[TestWasmArg::I32(140), TestWasmArg::I32(160)],
                        true,
                    );
                    let mut guest = CoreWasip1Instance::new(&env_get, Wasip1HandlerSet::FULL)
                        .expect("env get full");
                    let CoreWasip1Trap::EnvironGet(call) = guest.resume().expect("env get trap")
                    else {
                        panic!("expected environ_get");
                    };
                    guest
                        .complete_environ_get(call, &[(b"MODE", b"test")], 0)
                        .expect("complete env get");
                    assert_eq!(guest.read_memory_u32(140).expect("env ptr"), 160);
                    let mut env_bytes = [0u8; 10];
                    guest
                        .read_memory(160, &mut env_bytes)
                        .expect("read env bytes");
                    assert_eq!(&env_bytes, b"MODE=test\0");
                }
            })
            .expect("spawn args/env wasm test")
            .join()
            .expect("args/env wasm test joins");
    }

    #[test]
    fn core_wasip1_trampoline_maps_proc_exit_as_app_termination() {
        static CORE_WASIP1_PROC_EXIT_GUEST: &[u8] = &[
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x60, 0x01, 0x7f,
            0x00, 0x60, 0x00, 0x00, 0x02, 0x24, 0x01, 0x16, b'w', b'a', b's', b'i', b'_', b's',
            b'n', b'a', b'p', b's', b'h', b'o', b't', b'_', b'p', b'r', b'e', b'v', b'i', b'e',
            b'w', b'1', 0x09, b'p', b'r', b'o', b'c', b'_', b'e', b'x', b'i', b't', 0x00, 0x00,
            0x03, 0x02, 0x01, 0x01, 0x07, 0x0a, 0x01, 0x06, b'_', b's', b't', b'a', b'r', b't',
            0x00, 0x01, 0x0a, 0x08, 0x01, 0x06, 0x00, 0x41, 0x07, 0x10, 0x00, 0x0b,
        ];
        let mut guest =
            CoreWasip1Instance::new(CORE_WASIP1_PROC_EXIT_GUEST, Wasip1HandlerSet::PICO_MIN)
                .expect("instantiate core wasip1 proc_exit guest");

        assert_eq!(
            guest.resume().expect("proc_exit trampoline trap"),
            CoreWasip1Trap::ProcExit(7)
        );
        assert_eq!(
            guest.resume().expect("proc_exit terminates app"),
            CoreWasip1Trap::Done
        );
    }

    #[test]
    fn wasip1_engine_profile_rejects_disabled_syscall_imports_before_choreography() {
        let bytes = b"\0asm\x01\0\0\0wasi_snapshot_preview1 fd_write poll_oneoff";

        assert!(matches!(
            TinyWasip1TrafficLightInstance::from_wasip1_app_with_handlers(
                bytes,
                Wasip1HandlerSet::EMPTY
            ),
            Err(WasmError::Unsupported(
                "wasip1 fd_write disabled by feature profile"
            ))
        ));
    }

    #[test]
    fn demo_wasm_emits_expected_host_calls() {
        let mut guest = TinyWasmInstance::new(DEMO_WASM_GUEST).expect("instantiate guest");

        assert_eq!(
            guest.resume().expect("resume to log"),
            GuestTrap::HostCall(EngineReq::LogU32(0x4849_4241))
        );
        guest
            .complete_host_call(EngineRet::Logged(0x4849_4241))
            .expect("complete log call");

        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete yield call");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
        assert_eq!(guest.resume().expect("resume after done"), GuestTrap::Done);
    }

    #[test]
    fn guest_rejects_unexpected_reply() {
        let mut guest = TinyWasmInstance::new(DEMO_WASM_GUEST).expect("instantiate guest");
        let trap = guest.resume().expect("resume to first call");
        assert_eq!(trap, GuestTrap::HostCall(EngineReq::LogU32(0x4849_4241)));
        assert_eq!(
            guest.complete_host_call(EngineRet::Yielded),
            Err(WasmError::UnexpectedReply)
        );
    }

    #[test]
    fn route_wasm_guest_emits_two_samples_then_yield() {
        let mut guest = TinyWasmInstance::new(ROUTE_WASM_GUEST).expect("instantiate route guest");

        assert_eq!(
            guest.resume().expect("resume to normal sample"),
            GuestTrap::HostCall(EngineReq::LogU32(ROUTE_WASM_NORMAL_VALUE))
        );
        guest
            .complete_host_call(EngineRet::Logged(ROUTE_WASM_NORMAL_VALUE))
            .expect("complete normal sample");

        assert_eq!(
            guest.resume().expect("resume to alert sample"),
            GuestTrap::HostCall(EngineReq::LogU32(ROUTE_WASM_ALERT_VALUE))
        );
        guest
            .complete_host_call(EngineRet::Logged(ROUTE_WASM_ALERT_VALUE))
            .expect("complete alert sample");

        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete yield");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    }

    #[test]
    fn normal_wasm_guest_emits_normal_sample_then_yield() {
        let mut guest = TinyWasmInstance::new(NORMAL_WASM_GUEST).expect("instantiate normal guest");

        assert_eq!(
            guest.resume().expect("resume to normal sample"),
            GuestTrap::HostCall(EngineReq::LogU32(ROUTE_WASM_NORMAL_VALUE))
        );
        guest
            .complete_host_call(EngineRet::Logged(ROUTE_WASM_NORMAL_VALUE))
            .expect("complete normal sample");

        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete yield");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    }

    #[test]
    fn bad_route_guest_emits_yield_before_any_sample() {
        let mut guest =
            TinyWasmInstance::new(BAD_ROUTE_EARLY_YIELD_WASM_GUEST).expect("instantiate bad guest");

        assert_eq!(
            guest.resume().expect("resume to early yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete early yield");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    }

    #[test]
    fn sleep_wasm_guest_emits_timer_sleep_then_yield() {
        let mut guest = TinyWasmInstance::new(SLEEP_WASM_GUEST).expect("instantiate sleep guest");

        assert_eq!(
            guest.resume().expect("resume to sleep"),
            GuestTrap::HostCall(EngineReq::TimerSleepUntil(TimerSleepUntil::new(42)))
        );
        guest
            .complete_host_call(EngineRet::TimerSleepDone(TimerSleepDone::new(42)))
            .expect("complete sleep");

        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete yield");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    }

    #[test]
    fn gpio_wasm_guest_emits_gpio_set_then_yield() {
        let mut guest = TinyWasmInstance::new(GPIO_WASM_GUEST).expect("instantiate gpio guest");
        let set = GpioSet::new(25, true);

        assert_eq!(
            guest.resume().expect("resume to gpio set"),
            GuestTrap::HostCall(EngineReq::GpioSet(set))
        );
        guest
            .complete_host_call(EngineRet::GpioSetDone(set))
            .expect("complete gpio set");

        assert_eq!(
            guest.resume().expect("resume to yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
        guest
            .complete_host_call(EngineRet::Yielded)
            .expect("complete yield");

        assert_eq!(guest.resume().expect("resume to done"), GuestTrap::Done);
    }

    #[test]
    fn trap_guest_fails_closed_and_next_guest_can_run() {
        let mut trapped = TinyWasmInstance::new(TRAP_WASM_GUEST).expect("instantiate trap guest");
        assert_eq!(trapped.resume(), Err(WasmError::Trap));

        let mut next = TinyWasmInstance::new(NORMAL_WASM_GUEST).expect("instantiate next guest");
        assert_eq!(
            next.resume().expect("resume next guest"),
            GuestTrap::HostCall(EngineReq::LogU32(ROUTE_WASM_NORMAL_VALUE))
        );
        next.complete_host_call(EngineRet::Logged(ROUTE_WASM_NORMAL_VALUE))
            .expect("complete next log");
        assert_eq!(
            next.resume().expect("resume next yield"),
            GuestTrap::HostCall(EngineReq::Yield)
        );
    }

    #[test]
    fn trap_guest_revokes_outstanding_memory_leases() {
        let mut leases: MemoryLeaseTable<2> = MemoryLeaseTable::new(4096, 1);
        let grant = leases
            .grant_read(MemBorrow::new(1024, 8, 1))
            .expect("grant read");
        let chunk = StdoutChunk::new_with_lease(grant.lease_id(), b"hibana").expect("chunk");

        let mut trapped = TinyWasmInstance::new(TRAP_WASM_GUEST).expect("instantiate trap guest");
        assert_eq!(trapped.resume(), Err(WasmError::Trap));
        leases.fence(MemFence::new(MemFenceReason::Trap, 2));

        assert_eq!(
            leases.validate_read_chunk(&chunk),
            Err(MemoryLeaseError::UnknownLease)
        );
        assert_eq!(
            leases.grant_read(MemBorrow::new(1024, 8, 1)),
            Err(MemoryLeaseError::EpochMismatch)
        );
    }

    #[test]
    fn fuel_exhaustion_guest_is_bounded() {
        let mut guest =
            TinyWasmInstance::new(FUEL_EXHAUSTION_WASM_GUEST).expect("instantiate fuel guest");
        assert_eq!(guest.resume_with_fuel(8), Err(WasmError::FuelExhausted));
    }

    #[test]
    fn fuel_exhaustion_becomes_budget_expired_event() {
        let mut guest =
            TinyWasmInstance::new(FUEL_EXHAUSTION_WASM_GUEST).expect("instantiate fuel guest");
        let run = BudgetRun::new(11, 4, 8, 100);

        assert_eq!(
            guest.resume_with_budget(run),
            Ok(BudgetedGuestTrap::BudgetExpired(BudgetExpired::new(11, 4)))
        );
    }

    #[test]
    fn budgeted_resume_preserves_normal_guest_host_calls() {
        let mut guest = TinyWasmInstance::new(NORMAL_WASM_GUEST).expect("instantiate normal guest");
        let run = BudgetRun::new(12, 1, 32, 100);

        assert_eq!(
            guest.resume_with_budget(run),
            Ok(BudgetedGuestTrap::Guest(GuestTrap::HostCall(
                EngineReq::LogU32(ROUTE_WASM_NORMAL_VALUE)
            )))
        );
    }
}
