#[cfg(all(target_arch = "arm", target_os = "none"))]
use core::ptr::{read_volatile, write_volatile};

#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_BASE: usize = 0x4002_4000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_CTRL: *mut u32 = XOSC_BASE as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_STATUS: *const u32 = (XOSC_BASE + 0x04) as *const u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_STARTUP: *mut u32 = (XOSC_BASE + 0x0c) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_FREQ_RANGE_1_15MHZ: u32 = 0xaa0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_ENABLE: u32 = 0xfab << 12;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const XOSC_STATUS_STABLE: u32 = 1 << 31;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLOCKS_BASE: usize = 0x4000_8000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_REF_CTRL: *mut u32 = (CLOCKS_BASE + 0x30) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_REF_DIV: *mut u32 = (CLOCKS_BASE + 0x34) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_SYS_CTRL: *mut u32 = (CLOCKS_BASE + 0x3c) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_SYS_DIV: *mut u32 = (CLOCKS_BASE + 0x40) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_PERI_CTRL: *mut u32 = (CLOCKS_BASE + 0x48) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_DIV_1: u32 = 1 << 8;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_REF_SRC_XOSC: u32 = 2;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_SYS_SRC_REF: u32 = 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_SYS_SRC_AUX: u32 = 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_SYS_AUXSRC_PLL_SYS: u32 = 0 << 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_PERI_ENABLE: u32 = 1 << 11;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const CLK_PERI_AUXSRC_CLK_SYS: u32 = 0 << 5;

#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_BASE: usize = 0x4002_8000;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_CS: *mut u32 = PLL_SYS_BASE as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_PWR: *mut u32 = (PLL_SYS_BASE + 0x04) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_FBDIV_INT: *mut u32 = (PLL_SYS_BASE + 0x08) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_PRIM: *mut u32 = (PLL_SYS_BASE + 0x0c) as *mut u32;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_CS_LOCK: u32 = 1 << 31;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_PWR_PD: u32 = 1 << 0;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_PWR_POSTDIVPD: u32 = 1 << 3;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_PWR_VCOPD: u32 = 1 << 5;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_REFDIV_1: u32 = 1;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_FBDIV_125: u32 = 125;
#[cfg(all(target_arch = "arm", target_os = "none"))]
const PLL_SYS_POSTDIV_6_2: u32 = (6 << 16) | (2 << 12);

#[cfg(all(target_arch = "arm", target_os = "none"))]
fn wait_until(mut ready: impl FnMut() -> bool) -> bool {
    let mut spins = 0u32;
    while !ready() {
        spins = spins.wrapping_add(1);
        if spins == 2_000_000 {
            return false;
        }
        core::hint::spin_loop();
    }
    true
}

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub fn init_125mhz() -> bool {
    unsafe {
        write_volatile(XOSC_STARTUP, 47);
        write_volatile(XOSC_CTRL, XOSC_FREQ_RANGE_1_15MHZ | XOSC_ENABLE);
    }
    if !wait_until(|| unsafe { read_volatile(XOSC_STATUS) & XOSC_STATUS_STABLE != 0 }) {
        return false;
    }

    unsafe {
        write_volatile(CLK_SYS_CTRL, CLK_SYS_SRC_REF);
        write_volatile(CLK_SYS_DIV, CLK_DIV_1);
        write_volatile(CLK_REF_DIV, CLK_DIV_1);
        write_volatile(CLK_REF_CTRL, CLK_REF_SRC_XOSC);

        write_volatile(PLL_SYS_PWR, PLL_PWR_PD | PLL_PWR_VCOPD | PLL_PWR_POSTDIVPD);
        write_volatile(PLL_SYS_CS, PLL_SYS_REFDIV_1);
        write_volatile(PLL_SYS_FBDIV_INT, PLL_SYS_FBDIV_125);
        write_volatile(PLL_SYS_PWR, PLL_PWR_POSTDIVPD);
    }
    if !wait_until(|| unsafe { read_volatile(PLL_SYS_CS) & PLL_CS_LOCK != 0 }) {
        return false;
    }

    unsafe {
        write_volatile(PLL_SYS_PRIM, PLL_SYS_POSTDIV_6_2);
        write_volatile(PLL_SYS_PWR, 0);
        write_volatile(CLK_SYS_CTRL, CLK_SYS_AUXSRC_PLL_SYS | CLK_SYS_SRC_AUX);
        write_volatile(CLK_PERI_CTRL, CLK_PERI_ENABLE | CLK_PERI_AUXSRC_CLK_SYS);
    }
    true
}
