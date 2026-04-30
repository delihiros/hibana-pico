/*
 * Raspberry Pi Pico 2 W machine
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "qemu/osdep.h"
#include "qapi/error.h"
#include "hw/arm/boot.h"
#include "hw/arm/machines-qom.h"
#include "hw/arm/rp2350.h"
#include "hw/core/boards.h"
#include "hw/core/qdev-clock.h"
#include "hw/core/qdev-properties.h"
#include "system/system.h"
#include "qemu/error-report.h"

typedef struct Pico2WMachineState {
    MachineState parent_obj;

    RP2350State soc;
} Pico2WMachineState;

#define TYPE_RASPBERRYPI_PICO2W MACHINE_TYPE_NAME("raspberrypi-pico2w")
OBJECT_DECLARE_SIMPLE_TYPE(Pico2WMachineState, RASPBERRYPI_PICO2W)

static void raspberrypi_pico2w_init(MachineState *machine)
{
    Pico2WMachineState *s = RASPBERRYPI_PICO2W(machine);
    Clock *sysclk;

    if (!machine->kernel_filename) {
        error_report("raspberrypi-pico2w requires -kernel");
        exit(1);
    }

    sysclk = clock_new(OBJECT(machine), "SYSCLK");
    clock_set_hz(sysclk, 150000000ULL);

    object_initialize_child(OBJECT(machine), "soc", &s->soc, TYPE_RP2350_SOC);
    qdev_prop_set_chr(DEVICE(&s->soc), "serial0", serial_hd(0));
    qdev_prop_set_uint32(DEVICE(&s->soc), "num-cpus", machine->smp.cpus);
    qdev_connect_clock_in(DEVICE(&s->soc), "sysclk", sysclk);
    sysbus_realize(SYS_BUS_DEVICE(&s->soc), &error_fatal);

    armv7m_load_kernel(s->soc.cpu[0].cpu, machine->kernel_filename,
                       RP2350_XIP_BASE, RP2350_PICO2W_FLASH_SIZE);
    armv7m_load_kernel(s->soc.cpu[1].cpu, NULL, 0, 0);
}

static void raspberrypi_pico2w_machine_class_init(ObjectClass *oc,
                                                  const void *data)
{
    MachineClass *mc = MACHINE_CLASS(oc);
    static const char *const valid_cpu_types[] = {
        ARM_CPU_TYPE_NAME("cortex-m33"),
        NULL,
    };

    mc->desc = "Raspberry Pi Pico 2 W (RP2350 + CYW43439)";
    mc->init = raspberrypi_pico2w_init;
    mc->valid_cpu_types = valid_cpu_types;
    mc->max_cpus = RP2350_NUM_CORES;
    mc->default_cpus = RP2350_NUM_CORES;
    mc->default_ram_size = 0;
}

static const TypeInfo raspberrypi_pico2w_machine_type = {
    .name = TYPE_RASPBERRYPI_PICO2W,
    .parent = TYPE_MACHINE,
    .instance_size = sizeof(Pico2WMachineState),
    .class_init = raspberrypi_pico2w_machine_class_init,
    .interfaces = arm_machine_interfaces,
};

static void raspberrypi_pico2w_machine_init(void)
{
    type_register_static(&raspberrypi_pico2w_machine_type);
}

type_init(raspberrypi_pico2w_machine_init)
