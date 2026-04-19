/*
 * RP2040 SIO block
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#ifndef HW_MISC_RP2040_SIO_H
#define HW_MISC_RP2040_SIO_H

#include "hw/arm/armv7m.h"
#include "hw/core/sysbus.h"
#include "qom/object.h"

#define TYPE_RP2040_SIO "rp2040-sio"
OBJECT_DECLARE_SIMPLE_TYPE(RP2040SIOState, RP2040_SIO)

#define RP2040_SIO_FIFO_DEPTH 8

typedef struct RP2040Mailbox {
    uint32_t words[RP2040_SIO_FIFO_DEPTH];
    uint8_t head;
    uint8_t len;
} RP2040Mailbox;

struct RP2040SIOState {
    SysBusDevice parent_obj;

    MemoryRegion iomem;
    qemu_irq irq[2];
    ARMv7MState *cpu[2];

    uint32_t gpio_out;
    uint32_t gpio_oe;
    bool sticky_wof[2];
    bool sticky_roe[2];
    RP2040Mailbox fifo[2];
};

#endif
