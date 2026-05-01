use core::ptr::{read_volatile, write_volatile};

use hibana_pico::{
    choreography::protocol::GpioSet, machine::rp2040::baker_link::apply_baker_link_led_bank_set,
};

const SIO_BASE: usize = 0xD000_0000;
const IO_BANK0_BASE: usize = 0x4001_4000;
const PADS_BANK0_BASE: usize = 0x4001_c000;
const RESETS_BASE: usize = 0x4000_c000;
const RESETS_RESET_CLR: *mut u32 = (RESETS_BASE + 0x3000) as *mut u32;
const RESETS_RESET_DONE: *const u32 = (RESETS_BASE + 0x08) as *const u32;
const RESETS_IO_BANK0: u32 = 1 << 5;
const RESETS_PADS_BANK0: u32 = 1 << 8;
const GPIO_OUT_SET: *mut u32 = (SIO_BASE + 0x14) as *mut u32;
const GPIO_OUT_CLR: *mut u32 = (SIO_BASE + 0x18) as *mut u32;
const GPIO_OE_SET: *mut u32 = (SIO_BASE + 0x24) as *mut u32;
const GPIO_FUNC_SIO: u32 = 5;
const GPIO_PAD_DEFAULT: u32 = 0x56;

fn gpio_ctrl(pin: u8) -> *mut u32 {
    (IO_BANK0_BASE + 0x04 + (pin as usize * 8)) as *mut u32
}

fn gpio_pad(pin: u8) -> *mut u32 {
    (PADS_BANK0_BASE + 0x04 + (pin as usize * 4)) as *mut u32
}

pub fn rp2040_gpio_bank_init() {
    let reset_mask = RESETS_IO_BANK0 | RESETS_PADS_BANK0;
    unsafe {
        write_volatile(RESETS_RESET_CLR, reset_mask);
        while read_volatile(RESETS_RESET_DONE) & reset_mask != reset_mask {
            core::hint::spin_loop();
        }
    }
}

pub fn rp2040_gpio_init_output(pin: u8, initial_high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        write_volatile(gpio_pad(pin), GPIO_PAD_DEFAULT);
        write_volatile(gpio_ctrl(pin), GPIO_FUNC_SIO);
        if initial_high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
        write_volatile(GPIO_OE_SET, mask);
    }
}

fn rp2040_gpio_write(pin: u8, high: bool) {
    let mask = 1u32 << pin;
    unsafe {
        if high {
            write_volatile(GPIO_OUT_SET, mask);
        } else {
            write_volatile(GPIO_OUT_CLR, mask);
        }
    }
}

pub fn rp2040_gpio_apply_baker_led_set(set: GpioSet) {
    apply_baker_link_led_bank_set(rp2040_gpio_write, set);
}
