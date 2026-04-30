/*
 * Raspberry Pi RP2350 SoC
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#ifndef HW_ARM_RP2350_H
#define HW_ARM_RP2350_H

#include "hw/arm/armv7m.h"
#include "hw/char/pl011.h"
#include "hw/core/clock.h"
#include "hw/core/sysbus.h"
#include "hw/misc/rp2040_sio.h"
#include "hw/ssi/pl022.h"
#include "qom/object.h"

#define TYPE_RP2350_SOC "rp2350-soc"
OBJECT_DECLARE_SIMPLE_TYPE(RP2350State, RP2350_SOC)

#define RP2350_NUM_CORES 2

#define RP2350_BOOT_BASE 0x00000000
#define RP2350_XIP_BASE 0x10000000
#define RP2350_XIP_SIZE (16 * 1024 * 1024)
#define RP2350_PICO2W_FLASH_SIZE (4 * 1024 * 1024)
#define RP2350_SRAM_BASE 0x20000000
#define RP2350_SRAM_SIZE (520 * 1024)

#define RP2350_SYSINFO_BASE 0x40000000
#define RP2350_SYSCFG_BASE 0x40008000
#define RP2350_CLOCKS_BASE 0x40010000
#define RP2350_PSM_BASE 0x40018000
#define RP2350_RESETS_BASE 0x40020000
#define RP2350_IO_BANK0_BASE 0x40028000
#define RP2350_IO_QSPI_BASE 0x40030000
#define RP2350_PADS_BANK0_BASE 0x40038000
#define RP2350_PADS_QSPI_BASE 0x40040000
#define RP2350_UART0_BASE 0x40070000
#define RP2350_UART1_BASE 0x40078000
#define RP2350_SPI0_BASE 0x40080000
#define RP2350_SPI1_BASE 0x40088000
#define RP2350_TIMER0_BASE 0x400b0000
#define RP2350_TIMER1_BASE 0x400b8000
#define RP2350_WATCHDOG_BASE 0x400d8000
#define RP2350_BOOTRAM_BASE 0x400e0000
#define RP2350_SIO_BASE 0xd0000000

#define RP2350_SIO_IRQ_PROC0 15
#define RP2350_SIO_IRQ_PROC1 16
#define RP2350_UART0_IRQ 20
#define RP2350_SPI0_IRQ 22

struct RP2350State {
    SysBusDevice parent_obj;

    ARMv7MState cpu[RP2350_NUM_CORES];
    PL011State uart0;
    PL022State spi0;
    RP2040SIOState sio;

    MemoryRegion xip;
    MemoryRegion xip_alias;
    MemoryRegion sram;
    MemoryRegion cpu_memory[RP2350_NUM_CORES];

    Clock *sysclk;
    uint32_t num_cpus;
};

#endif
