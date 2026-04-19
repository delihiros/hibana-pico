/*
 * Raspberry Pi RP2040 SoC
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#ifndef HW_ARM_RP2040_H
#define HW_ARM_RP2040_H

#include "hw/arm/armv7m.h"
#include "hw/char/pl011.h"
#include "hw/core/clock.h"
#include "hw/core/sysbus.h"
#include "hw/misc/rp2040_sio.h"
#include "qom/object.h"

#define TYPE_RP2040_SOC "rp2040-soc"
OBJECT_DECLARE_SIMPLE_TYPE(RP2040State, RP2040_SOC)

#define RP2040_NUM_CORES 2

#define RP2040_BOOT_BASE 0x00000000
#define RP2040_XIP_BASE 0x10000000
#define RP2040_XIP_SIZE (16 * 1024 * 1024)
#define RP2040_SRAM_BASE 0x20000000
#define RP2040_SRAM_SIZE (264 * 1024)

#define RP2040_IO_BANK0_BASE 0x40014000
#define RP2040_RESETS_BASE 0x4000c000
#define RP2040_PSM_BASE 0x40010000
#define RP2040_UART0_BASE 0x40034000
#define RP2040_TIMER_BASE 0x40054000
#define RP2040_SIO_BASE 0xd0000000

#define RP2040_SIO_IRQ_PROC0 15
#define RP2040_SIO_IRQ_PROC1 16
#define RP2040_UART0_IRQ 20

struct RP2040State {
    SysBusDevice parent_obj;

    ARMv7MState cpu[RP2040_NUM_CORES];
    PL011State uart0;
    RP2040SIOState sio;

    MemoryRegion xip;
    MemoryRegion xip_alias;
    MemoryRegion sram;
    MemoryRegion cpu_memory[RP2040_NUM_CORES];

    Clock *sysclk;
    uint32_t num_cpus;
};

#endif
