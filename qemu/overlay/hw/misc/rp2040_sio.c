/*
 * RP2040 SIO block
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "qemu/osdep.h"
#include "hw/misc/rp2040_sio.h"
#include "hw/core/cpu.h"
#include "hw/core/irq.h"
#include "hw/core/qdev-properties.h"
#include "migration/vmstate.h"
#include "qemu/module.h"
#include "qemu/log.h"

#define SIO_CPUID 0x000
#define SIO_GPIO_OUT 0x010
#define SIO_GPIO_OUT_SET 0x014
#define SIO_GPIO_OUT_CLR 0x018
#define SIO_GPIO_OUT_XOR 0x01c
#define SIO_GPIO_OE 0x020
#define SIO_GPIO_OE_SET 0x024
#define SIO_GPIO_OE_CLR 0x028
#define SIO_GPIO_OE_XOR 0x02c
#define SIO_FIFO_ST 0x050
#define SIO_FIFO_WR 0x054
#define SIO_FIFO_RD 0x058
#define SIO_SPINLOCK_ST 0x05c

#define SIO_FIFO_ST_VLD BIT(0)
#define SIO_FIFO_ST_RDY BIT(1)
#define SIO_FIFO_ST_WOF BIT(2)
#define SIO_FIFO_ST_ROE BIT(3)

static bool rp2040_mailbox_full(const RP2040Mailbox *fifo)
{
    return fifo->len == RP2040_SIO_FIFO_DEPTH;
}

static bool rp2040_mailbox_empty(const RP2040Mailbox *fifo)
{
    return fifo->len == 0;
}

static void rp2040_mailbox_push(RP2040Mailbox *fifo, uint32_t value)
{
    unsigned int slot = (fifo->head + fifo->len) % RP2040_SIO_FIFO_DEPTH;

    fifo->words[slot] = value;
    fifo->len++;
}

static uint32_t rp2040_mailbox_pop(RP2040Mailbox *fifo)
{
    uint32_t value = fifo->words[fifo->head];

    fifo->head = (fifo->head + 1) % RP2040_SIO_FIFO_DEPTH;
    fifo->len--;
    return value;
}

static int rp2040_sio_current_core(RP2040SIOState *s)
{
    if (s->cpu[1] && current_cpu == CPU(s->cpu[1]->cpu)) {
        return 1;
    }
    return 0;
}

static RP2040Mailbox *rp2040_sio_rx_fifo(RP2040SIOState *s, int core)
{
    return &s->fifo[1 - core];
}

static RP2040Mailbox *rp2040_sio_tx_fifo(RP2040SIOState *s, int core)
{
    return &s->fifo[core];
}

static uint32_t rp2040_sio_fifo_status(RP2040SIOState *s, int core)
{
    RP2040Mailbox *rx = rp2040_sio_rx_fifo(s, core);
    RP2040Mailbox *tx = rp2040_sio_tx_fifo(s, core);
    uint32_t status = 0;

    if (!rp2040_mailbox_empty(rx)) {
        status |= SIO_FIFO_ST_VLD;
    }
    if (!rp2040_mailbox_full(tx)) {
        status |= SIO_FIFO_ST_RDY;
    }
    if (s->sticky_wof[core]) {
        status |= SIO_FIFO_ST_WOF;
    }
    if (s->sticky_roe[core]) {
        status |= SIO_FIFO_ST_ROE;
    }
    return status;
}

static void rp2040_sio_update_irqs(RP2040SIOState *s)
{
    for (int core = 0; core < 2; core++) {
        uint32_t status = rp2040_sio_fifo_status(s, core);
        qemu_set_irq(s->irq[core], status & (SIO_FIFO_ST_VLD |
                                             SIO_FIFO_ST_WOF |
                                             SIO_FIFO_ST_ROE));
    }
}

static void rp2040_sio_signal_peer(RP2040SIOState *s, int peer)
{
    if (!s->cpu[peer]) {
        return;
    }

    armv7m_set_event(s->cpu[peer]);
}

static uint64_t rp2040_sio_read(void *opaque, hwaddr addr, unsigned int size)
{
    RP2040SIOState *s = opaque;
    int core = rp2040_sio_current_core(s);
    RP2040Mailbox *rx = rp2040_sio_rx_fifo(s, core);

    switch (addr) {
    case SIO_CPUID:
        return core;
    case SIO_GPIO_OUT:
        return s->gpio_out;
    case SIO_GPIO_OE:
        return s->gpio_oe;
    case SIO_FIFO_ST:
        return rp2040_sio_fifo_status(s, core);
    case SIO_FIFO_RD:
        if (rp2040_mailbox_empty(rx)) {
            s->sticky_roe[core] = true;
            rp2040_sio_update_irqs(s);
            return 0;
        }
        {
            uint32_t value = rp2040_mailbox_pop(rx);
            rp2040_sio_update_irqs(s);
            return value;
        }
    case SIO_SPINLOCK_ST:
        return 0;
    default:
        qemu_log_mask(LOG_GUEST_ERROR,
                      "rp2040-sio: bad read offset 0x%" HWADDR_PRIx "\n",
                      addr);
        return 0;
    }
}

static void rp2040_sio_write(void *opaque, hwaddr addr,
                             uint64_t value, unsigned int size)
{
    RP2040SIOState *s = opaque;
    int core = rp2040_sio_current_core(s);
    int peer = 1 - core;
    RP2040Mailbox *tx = rp2040_sio_tx_fifo(s, core);

    switch (addr) {
    case SIO_GPIO_OUT:
        s->gpio_out = value;
        break;
    case SIO_GPIO_OUT_SET:
        s->gpio_out |= value;
        break;
    case SIO_GPIO_OUT_CLR:
        s->gpio_out &= ~value;
        break;
    case SIO_GPIO_OUT_XOR:
        s->gpio_out ^= value;
        break;
    case SIO_GPIO_OE:
        s->gpio_oe = value;
        break;
    case SIO_GPIO_OE_SET:
        s->gpio_oe |= value;
        break;
    case SIO_GPIO_OE_CLR:
        s->gpio_oe &= ~value;
        break;
    case SIO_GPIO_OE_XOR:
        s->gpio_oe ^= value;
        break;
    case SIO_FIFO_ST:
        if (value & SIO_FIFO_ST_WOF) {
            s->sticky_wof[core] = false;
        }
        if (value & SIO_FIFO_ST_ROE) {
            s->sticky_roe[core] = false;
        }
        rp2040_sio_update_irqs(s);
        break;
    case SIO_FIFO_WR:
        if (rp2040_mailbox_full(tx)) {
            s->sticky_wof[core] = true;
        } else {
            rp2040_mailbox_push(tx, value);
            rp2040_sio_signal_peer(s, peer);
        }
        rp2040_sio_update_irqs(s);
        break;
    default:
        qemu_log_mask(LOG_GUEST_ERROR,
                      "rp2040-sio: bad write offset 0x%" HWADDR_PRIx
                      " = 0x%" PRIx64 "\n", addr, value);
        break;
    }
}

static const MemoryRegionOps rp2040_sio_ops = {
    .read = rp2040_sio_read,
    .write = rp2040_sio_write,
    .endianness = DEVICE_LITTLE_ENDIAN,
    .valid.min_access_size = 4,
    .valid.max_access_size = 4,
    .impl.min_access_size = 4,
    .impl.max_access_size = 4,
};

static void rp2040_sio_reset(DeviceState *dev)
{
    RP2040SIOState *s = RP2040_SIO(dev);

    s->gpio_out = 0;
    s->gpio_oe = 0;
    memset(s->sticky_wof, 0, sizeof(s->sticky_wof));
    memset(s->sticky_roe, 0, sizeof(s->sticky_roe));
    memset(s->fifo, 0, sizeof(s->fifo));
    rp2040_sio_update_irqs(s);
}

static void rp2040_sio_init(Object *obj)
{
    RP2040SIOState *s = RP2040_SIO(obj);
    SysBusDevice *sbd = SYS_BUS_DEVICE(obj);

    memory_region_init_io(&s->iomem, obj, &rp2040_sio_ops, s, "rp2040-sio",
                          0x1000);
    sysbus_init_mmio(sbd, &s->iomem);
    for (int i = 0; i < 2; i++) {
        sysbus_init_irq(sbd, &s->irq[i]);
    }
}

static const Property rp2040_sio_properties[] = {
    DEFINE_PROP_LINK("cpu0", RP2040SIOState, cpu[0], TYPE_ARMV7M, ARMv7MState *),
    DEFINE_PROP_LINK("cpu1", RP2040SIOState, cpu[1], TYPE_ARMV7M, ARMv7MState *),
};

static const VMStateDescription vmstate_rp2040_mailbox = {
    .name = "rp2040-sio-mailbox",
    .version_id = 1,
    .minimum_version_id = 1,
    .fields = (const VMStateField[]) {
        VMSTATE_UINT32_ARRAY(words, RP2040Mailbox, RP2040_SIO_FIFO_DEPTH),
        VMSTATE_UINT8(head, RP2040Mailbox),
        VMSTATE_UINT8(len, RP2040Mailbox),
        VMSTATE_END_OF_LIST()
    },
};

static const VMStateDescription vmstate_rp2040_sio = {
    .name = "rp2040-sio",
    .version_id = 1,
    .minimum_version_id = 1,
    .fields = (const VMStateField[]) {
        VMSTATE_UINT32(gpio_out, RP2040SIOState),
        VMSTATE_UINT32(gpio_oe, RP2040SIOState),
        VMSTATE_BOOL_ARRAY(sticky_wof, RP2040SIOState, 2),
        VMSTATE_BOOL_ARRAY(sticky_roe, RP2040SIOState, 2),
        VMSTATE_STRUCT_ARRAY(fifo, RP2040SIOState, 2, 0,
                             vmstate_rp2040_mailbox, RP2040Mailbox),
        VMSTATE_END_OF_LIST()
    },
};

static void rp2040_sio_class_init(ObjectClass *klass, const void *data)
{
    DeviceClass *dc = DEVICE_CLASS(klass);

    dc->vmsd = &vmstate_rp2040_sio;
    device_class_set_props(dc, rp2040_sio_properties);
    device_class_set_legacy_reset(dc, rp2040_sio_reset);
}

static const TypeInfo rp2040_sio_type_info = {
    .name = TYPE_RP2040_SIO,
    .parent = TYPE_SYS_BUS_DEVICE,
    .instance_size = sizeof(RP2040SIOState),
    .instance_init = rp2040_sio_init,
    .class_init = rp2040_sio_class_init,
};

static void rp2040_sio_register_types(void)
{
    type_register_static(&rp2040_sio_type_info);
}

type_init(rp2040_sio_register_types)
