/*
 * Raspberry Pi RP2040 SoC
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "qemu/osdep.h"
#include "qapi/error.h"
#include "hw/arm/rp2040.h"
#include "hw/core/qdev-clock.h"
#include "hw/core/qdev-properties.h"
#include "hw/misc/unimp.h"
#include "qemu/module.h"
#include "system/address-spaces.h"

#define RP2040_SYSCLK_HZ 125000000ULL
#define RP2040_CPU_ADDRESS_SPACE_SIZE (UINT64_C(1) << 32)

static void rp2040_soc_init(Object *obj)
{
    RP2040State *s = RP2040_SOC(obj);

    for (int i = 0; i < RP2040_NUM_CORES; i++) {
        object_initialize_child(obj, "armv7m[*]", &s->cpu[i], TYPE_ARMV7M);
    }

    object_initialize_child(obj, "uart0", &s->uart0, TYPE_PL011);
    object_initialize_child(obj, "sio", &s->sio, TYPE_RP2040_SIO);
    object_property_add_alias(obj, "serial0", OBJECT(&s->uart0), "chardev");

    s->sysclk = qdev_init_clock_in(DEVICE(obj), "sysclk", NULL, NULL, 0);
}

static void rp2040_soc_realize(DeviceState *dev, Error **errp)
{
    RP2040State *s = RP2040_SOC(dev);
    MemoryRegion *system_memory = get_system_memory();
    DeviceState *armv7m;
    uint32_t cores = s->num_cpus ? s->num_cpus : RP2040_NUM_CORES;

    if (cores != RP2040_NUM_CORES) {
        error_setg(errp, "rp2040: only dual-core mode is currently supported");
        return;
    }
    if (!clock_has_source(s->sysclk)) {
        error_setg(errp, "rp2040: sysclk must be connected");
        return;
    }

    if (!memory_region_init_ram(&s->xip, OBJECT(dev), "rp2040.xip",
                                RP2040_XIP_SIZE, errp)) {
        return;
    }
    memory_region_init_alias(&s->xip_alias, OBJECT(dev), "rp2040.xip.alias",
                             &s->xip, 0, RP2040_XIP_SIZE);
    if (!memory_region_init_ram(&s->sram, OBJECT(dev), "rp2040.sram",
                                RP2040_SRAM_SIZE, errp)) {
        return;
    }

    memory_region_add_subregion(system_memory, RP2040_BOOT_BASE, &s->xip_alias);
    memory_region_add_subregion(system_memory, RP2040_XIP_BASE, &s->xip);
    memory_region_add_subregion(system_memory, RP2040_SRAM_BASE, &s->sram);

    for (uint32_t i = 0; i < cores; i++) {
        memory_region_init_alias(&s->cpu_memory[i], OBJECT(dev),
                                 "rp2040.cpu-memory", system_memory, 0,
                                 RP2040_CPU_ADDRESS_SPACE_SIZE);

        armv7m = DEVICE(&s->cpu[i]);
        qdev_prop_set_uint32(armv7m, "num-irq", 32);
        qdev_prop_set_string(armv7m, "cpu-type",
                             ARM_CPU_TYPE_NAME("cortex-m0"));
        qdev_prop_set_bit(armv7m, "enable-bitband", false);
        object_property_set_link(OBJECT(&s->cpu[i]), "memory",
                                 OBJECT(&s->cpu_memory[i]), &error_abort);
        qdev_connect_clock_in(armv7m, "cpuclk", s->sysclk);
        if (!sysbus_realize(SYS_BUS_DEVICE(&s->cpu[i]), errp)) {
            return;
        }
    }

    if (!sysbus_realize(SYS_BUS_DEVICE(&s->uart0), errp)) {
        return;
    }
    sysbus_mmio_map(SYS_BUS_DEVICE(&s->uart0), 0, RP2040_UART0_BASE);
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->uart0), 0,
                       qdev_get_gpio_in(DEVICE(&s->cpu[0]), RP2040_UART0_IRQ));

    object_property_set_link(OBJECT(&s->sio), "cpu0", OBJECT(&s->cpu[0]),
                             &error_abort);
    object_property_set_link(OBJECT(&s->sio), "cpu1", OBJECT(&s->cpu[1]),
                             &error_abort);
    if (!sysbus_realize(SYS_BUS_DEVICE(&s->sio), errp)) {
        return;
    }
    sysbus_mmio_map(SYS_BUS_DEVICE(&s->sio), 0, RP2040_SIO_BASE);
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->sio), 0,
                       qdev_get_gpio_in(DEVICE(&s->cpu[0]),
                                        RP2040_SIO_IRQ_PROC0));
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->sio), 1,
                       qdev_get_gpio_in(DEVICE(&s->cpu[1]),
                                        RP2040_SIO_IRQ_PROC1));

    create_unimplemented_device("rp2040.resets", RP2040_RESETS_BASE, 0x1000);
    create_unimplemented_device("rp2040.psm", RP2040_PSM_BASE, 0x1000);
    create_unimplemented_device("rp2040.io_bank0", RP2040_IO_BANK0_BASE,
                                0x1000);
    create_unimplemented_device("rp2040.timer", RP2040_TIMER_BASE, 0x1000);
}

static const Property rp2040_soc_properties[] = {
    DEFINE_PROP_UINT32("num-cpus", RP2040State, num_cpus, RP2040_NUM_CORES),
};

static void rp2040_soc_class_init(ObjectClass *klass, const void *data)
{
    DeviceClass *dc = DEVICE_CLASS(klass);

    dc->realize = rp2040_soc_realize;
    device_class_set_props(dc, rp2040_soc_properties);
}

static const TypeInfo rp2040_soc_type_info = {
    .name = TYPE_RP2040_SOC,
    .parent = TYPE_SYS_BUS_DEVICE,
    .instance_size = sizeof(RP2040State),
    .instance_init = rp2040_soc_init,
    .class_init = rp2040_soc_class_init,
};

static void rp2040_register_types(void)
{
    type_register_static(&rp2040_soc_type_info);
}

type_init(rp2040_register_types)
