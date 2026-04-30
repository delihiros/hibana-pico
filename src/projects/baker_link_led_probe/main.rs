#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_std)]
#![cfg_attr(all(target_arch = "arm", target_os = "none"), no_main)]

#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::{
    arch::{asm, naked_asm},
    ptr::{read_volatile, write_volatile},
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(link_section = ".boot2")]
#[used]
static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" {
    static __stack_top: u32;
    static __data_load_start: u8;
    static mut __data_start: u8;
    static mut __data_end: u8;
    static mut __bss_start: u8;
    static mut __bss_end: u8;
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
const SIO_BASE: usize = 0xD000_0000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OUT: *const u32 = (SIO_BASE + 0x10) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x14) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x18) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OE: *const u32 = (SIO_BASE + 0x20) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x24) as *mut u32;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const IO_BANK0_BASE: usize = 0x4001_4000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PADS_BANK0_BASE: usize = 0x4001_c000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_BASE: usize = 0x4000_c000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_RESET_CLR: *mut u32 = (RESETS_BASE + 0x3000) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_IO_BANK0: u32 = 1 << 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const RESETS_PADS_BANK0: u32 = 1 << 8;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_FUNC_SIO: u32 = 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const GPIO_PAD_DEFAULT: u32 = 0x56;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_START: u32 = 0x484c_0001;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ALL_HIGH: u32 = 0x484c_0100;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_ALL_LOW: u32 = 0x484c_0200;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_CHASE_HIGH: u32 = 0x484c_1000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const STAGE_CHASE_LOW: u32 = 0x484c_2000;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const CANDIDATE_LED_PINS: [u8; 3] = [22, 21, 20];
#[cfg(all(target_arch = "arm", target_os = "none"))]
const VISIBLE_DELAY_CYCLES: u32 = 120_000_000;

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_STAGE: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_PIN: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_LEVEL: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_GPIO_OUT: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_GPIO_OE: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_CTRL: [u32; 3] = [0; 3];
#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(no_mangle)]
static mut HIBANA_LED_PROBE_PAD: [u32; 3] = [0; 3];

#[cfg(all(target_arch = "arm", target_os = "none"))]
type IrqHandler = unsafe extern "C" fn();

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[repr(C)]
struct VectorTable {
    initial_stack_pointer: *const u32,
    reset: unsafe extern "C" fn() -> !,
    exceptions: [IrqHandler; 14],
    external_irqs: [IrqHandler; 32],
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe impl Sync for VectorTable {}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(link_section = ".vector_table.reset_vector")]
#[used]
static VECTOR_TABLE: VectorTable = VectorTable {
    initial_stack_pointer: core::ptr::addr_of!(__stack_top) as *const u32,
    reset,
    exceptions: [default_irq_handler; 14],
    external_irqs: [default_irq_handler; 32],
};

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[unsafe(naked)]
#[unsafe(export_name = "Reset")]
pub unsafe extern "C" fn reset() -> ! {
    naked_asm!("ldr r0, ={entry}", "bx r0", entry = sym reset_entry);
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn reset_entry() -> ! {
    init_ram();
    led_probe_main()
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
unsafe extern "C" fn default_irq_handler() {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn init_ram() {
    unsafe {
        let data_src = core::ptr::addr_of!(__data_load_start);
        let data_start = core::ptr::addr_of_mut!(__data_start);
        let data_end = core::ptr::addr_of_mut!(__data_end);
        let data_len = data_end as usize - data_start as usize;
        core::ptr::copy_nonoverlapping(data_src, data_start, data_len);

        let bss_start = core::ptr::addr_of_mut!(__bss_start);
        let bss_end = core::ptr::addr_of_mut!(__bss_end);
        let bss_len = bss_end as usize - bss_start as usize;
        core::ptr::write_bytes(bss_start, 0, bss_len);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn mark(stage: u32, pin: u8, high: bool) {
    unsafe {
        write_volatile(core::ptr::addr_of_mut!(HIBANA_LED_PROBE_STAGE), stage);
        write_volatile(core::ptr::addr_of_mut!(HIBANA_LED_PROBE_PIN), pin as u32);
        write_volatile(core::ptr::addr_of_mut!(HIBANA_LED_PROBE_LEVEL), high as u32);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn snapshot_gpio() {
    unsafe {
        write_volatile(
            core::ptr::addr_of_mut!(HIBANA_LED_PROBE_GPIO_OUT),
            read_volatile(GPIO_OUT),
        );
        write_volatile(
            core::ptr::addr_of_mut!(HIBANA_LED_PROBE_GPIO_OE),
            read_volatile(GPIO_OE),
        );
        let ctrl = core::ptr::addr_of_mut!(HIBANA_LED_PROBE_CTRL) as *mut u32;
        let pad = core::ptr::addr_of_mut!(HIBANA_LED_PROBE_PAD) as *mut u32;
        let mut idx = 0usize;
        while idx < CANDIDATE_LED_PINS.len() {
            let pin = CANDIDATE_LED_PINS[idx];
            write_volatile(ctrl.add(idx), read_volatile(gpio_ctrl(pin)));
            write_volatile(pad.add(idx), read_volatile(gpio_pad(pin)));
            idx += 1;
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_ctrl(pin: u8) -> *mut u32 {
    (IO_BANK0_BASE + 0x04 + (pin as usize * 8)) as *mut u32
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_pad(pin: u8) -> *mut u32 {
    (PADS_BANK0_BASE + 0x04 + (pin as usize * 4)) as *mut u32
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_bank_init() {
    let reset_mask = RESETS_IO_BANK0 | RESETS_PADS_BANK0;
    unsafe {
        write_volatile(RESETS_RESET_CLR, reset_mask);
        while read_volatile(RESETS_RESET_DONE) & reset_mask != reset_mask {
            core::hint::spin_loop();
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_init_output(pin: u8, high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        write_volatile(gpio_pad(pin), GPIO_PAD_DEFAULT);
        write_volatile(gpio_ctrl(pin), GPIO_FUNC_SIO);
        if high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
        write_volatile(GPIO_OE_SET, mask);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn gpio_write(pin: u8, high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        if high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn write_all(high: bool) {
    for pin in CANDIDATE_LED_PINS {
        gpio_write(pin, high);
    }
    snapshot_gpio();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn one_hot(pin: u8, active_high: bool) {
    write_all(!active_high);
    gpio_write(pin, active_high);
    snapshot_gpio();
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn delay_visible() {
    let mut cycles = VISIBLE_DELAY_CYCLES;
    while cycles != 0 {
        unsafe {
            asm!("nop", options(nomem, nostack, preserves_flags));
        }
        cycles = cycles.wrapping_sub(1);
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn led_probe_main() -> ! {
    mark(STAGE_START, 0, false);
    gpio_bank_init();
    for pin in CANDIDATE_LED_PINS {
        gpio_init_output(pin, false);
    }
    snapshot_gpio();
    mark(STAGE_ALL_LOW, 0, false);

    loop {
        mark(STAGE_ALL_LOW, 0, false);
        write_all(false);
        delay_visible();

        mark(STAGE_ALL_HIGH, 0, true);
        write_all(true);
        delay_visible();

        for pin in CANDIDATE_LED_PINS {
            mark(STAGE_CHASE_HIGH | pin as u32, pin, true);
            one_hot(pin, true);
            delay_visible();
        }

        for pin in CANDIDATE_LED_PINS {
            mark(STAGE_CHASE_LOW | pin as u32, pin, false);
            one_hot(pin, false);
            delay_visible();
        }
    }
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo<'_>) -> ! {
    loop {
        unsafe {
            asm!("wfi");
        }
    }
}

#[cfg(not(all(target_arch = "arm", target_os = "none")))]
fn main() {}
