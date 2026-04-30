/*
 * Raspberry Pi RP2350 SoC
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "qemu/osdep.h"
#include "qapi/error.h"
#include "hw/arm/rp2350.h"
#include "hw/core/qdev-clock.h"
#include "hw/core/qdev-properties.h"
#include "hw/misc/cyw43439_wifi.h"
#include "hw/misc/unimp.h"
#include "hw/ssi/ssi.h"
#include "qemu/module.h"
#include "system/address-spaces.h"

#define RP2350_CPU_ADDRESS_SPACE_SIZE (UINT64_C(1) << 32)

static void rp2350_soc_init(Object *obj)
{
    RP2350State *s = RP2350_SOC(obj);

    for (int i = 0; i < RP2350_NUM_CORES; i++) {
        object_initialize_child(obj, "armv7m[*]", &s->cpu[i], TYPE_ARMV7M);
    }

    object_initialize_child(obj, "uart0", &s->uart0, TYPE_PL011);
    object_initialize_child(obj, "spi0", &s->spi0, TYPE_PL022);
    object_initialize_child(obj, "sio", &s->sio, TYPE_RP2040_SIO);
    object_property_add_alias(obj, "serial0", OBJECT(&s->uart0), "chardev");

    s->sysclk = qdev_init_clock_in(DEVICE(obj), "sysclk", NULL, NULL, 0);
}

static void rp2350_soc_realize(DeviceState *dev, Error **errp)
{
    RP2350State *s = RP2350_SOC(dev);
    MemoryRegion *system_memory = get_system_memory();
    DeviceState *armv7m;
    DeviceState *cyw;
    uint32_t cores = s->num_cpus ? s->num_cpus : RP2350_NUM_CORES;

    if (cores != RP2350_NUM_CORES) {
        error_setg(errp, "rp2350: only dual-core Arm mode is currently supported");
        return;
    }
    if (!clock_has_source(s->sysclk)) {
        error_setg(errp, "rp2350: sysclk must be connected");
        return;
    }

    if (!memory_region_init_ram(&s->xip, OBJECT(dev), "rp2350.xip",
                                RP2350_XIP_SIZE, errp)) {
        return;
    }
    memory_region_init_alias(&s->xip_alias, OBJECT(dev), "rp2350.xip.alias",
                             &s->xip, 0, RP2350_XIP_SIZE);
    if (!memory_region_init_ram(&s->sram, OBJECT(dev), "rp2350.sram",
                                RP2350_SRAM_SIZE, errp)) {
        return;
    }

    memory_region_add_subregion(system_memory, RP2350_BOOT_BASE, &s->xip_alias);
    memory_region_add_subregion(system_memory, RP2350_XIP_BASE, &s->xip);
    memory_region_add_subregion(system_memory, RP2350_SRAM_BASE, &s->sram);

    for (uint32_t i = 0; i < cores; i++) {
        memory_region_init_alias(&s->cpu_memory[i], OBJECT(dev),
                                 "rp2350.cpu-memory", system_memory, 0,
                                 RP2350_CPU_ADDRESS_SPACE_SIZE);

        armv7m = DEVICE(&s->cpu[i]);
        qdev_prop_set_uint32(armv7m, "num-irq", 64);
        qdev_prop_set_string(armv7m, "cpu-type",
                             ARM_CPU_TYPE_NAME("cortex-m33"));
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
    sysbus_mmio_map(SYS_BUS_DEVICE(&s->uart0), 0, RP2350_UART0_BASE);
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->uart0), 0,
                       qdev_get_gpio_in(DEVICE(&s->cpu[0]), RP2350_UART0_IRQ));

    if (!sysbus_realize(SYS_BUS_DEVICE(&s->spi0), errp)) {
        return;
    }
    sysbus_mmio_map(SYS_BUS_DEVICE(&s->spi0), 0, RP2350_SPI0_BASE);
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->spi0), 0,
                       qdev_get_gpio_in(DEVICE(&s->cpu[0]), RP2350_SPI0_IRQ));

    cyw = qdev_new(TYPE_CYW43439_WIFI);
    object_property_set_link(OBJECT(cyw), "cpu0", OBJECT(&s->cpu[0]),
                             &error_abort);
    object_property_set_link(OBJECT(cyw), "cpu1", OBJECT(&s->cpu[1]),
                             &error_abort);
    if (!ssi_realize_and_unref(cyw, s->spi0.ssi, errp)) {
        return;
    }

    object_property_set_link(OBJECT(&s->sio), "cpu0", OBJECT(&s->cpu[0]),
                             &error_abort);
    object_property_set_link(OBJECT(&s->sio), "cpu1", OBJECT(&s->cpu[1]),
                             &error_abort);
    if (!sysbus_realize(SYS_BUS_DEVICE(&s->sio), errp)) {
        return;
    }
    sysbus_mmio_map(SYS_BUS_DEVICE(&s->sio), 0, RP2350_SIO_BASE);
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->sio), 0,
                       qdev_get_gpio_in(DEVICE(&s->cpu[0]),
                                        RP2350_SIO_IRQ_PROC0));
    sysbus_connect_irq(SYS_BUS_DEVICE(&s->sio), 1,
                       qdev_get_gpio_in(DEVICE(&s->cpu[1]),
                                        RP2350_SIO_IRQ_PROC1));

    create_unimplemented_device("rp2350.sysinfo", RP2350_SYSINFO_BASE, 0x1000);
    create_unimplemented_device("rp2350.syscfg", RP2350_SYSCFG_BASE, 0x1000);
    create_unimplemented_device("rp2350.clocks", RP2350_CLOCKS_BASE, 0x1000);
    create_unimplemented_device("rp2350.psm", RP2350_PSM_BASE, 0x1000);
    create_unimplemented_device("rp2350.resets", RP2350_RESETS_BASE, 0x1000);
    create_unimplemented_device("rp2350.io_bank0", RP2350_IO_BANK0_BASE, 0x1000);
    create_unimplemented_device("rp2350.io_qspi", RP2350_IO_QSPI_BASE, 0x1000);
    create_unimplemented_device("rp2350.pads_bank0", RP2350_PADS_BANK0_BASE,
                                0x1000);
    create_unimplemented_device("rp2350.pads_qspi", RP2350_PADS_QSPI_BASE,
                                0x1000);
    create_unimplemented_device("rp2350.timer0", RP2350_TIMER0_BASE, 0x1000);
    create_unimplemented_device("rp2350.timer1", RP2350_TIMER1_BASE, 0x1000);
    create_unimplemented_device("rp2350.watchdog", RP2350_WATCHDOG_BASE,
                                0x1000);
    create_unimplemented_device("rp2350.bootram", RP2350_BOOTRAM_BASE, 0x1000);
}

static const Property rp2350_soc_properties[] = {
    DEFINE_PROP_UINT32("num-cpus", RP2350State, num_cpus, RP2350_NUM_CORES),
};

static void rp2350_soc_class_init(ObjectClass *klass, const void *data)
{
    DeviceClass *dc = DEVICE_CLASS(klass);

    dc->realize = rp2350_soc_realize;
    device_class_set_props(dc, rp2350_soc_properties);
}

static const TypeInfo rp2350_soc_type_info = {
    .name = TYPE_RP2350_SOC,
    .parent = TYPE_SYS_BUS_DEVICE,
    .instance_size = sizeof(RP2350State),
    .instance_init = rp2350_soc_init,
    .class_init = rp2350_soc_class_init,
};

static void rp2350_register_types(void)
{
    type_register_static(&rp2350_soc_type_info);
}

type_init(rp2350_register_types)
