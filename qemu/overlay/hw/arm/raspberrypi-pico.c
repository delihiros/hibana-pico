/*
 * Raspberry Pi Pico machine
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#include "qemu/osdep.h"
#include "qapi/error.h"
#include "hw/arm/boot.h"
#include "hw/arm/machines-qom.h"
#include "hw/arm/rp2040.h"
#include "hw/core/boards.h"
#include "hw/core/qdev-clock.h"
#include "hw/core/qdev-properties.h"
#include "system/system.h"
#include "qemu/error-report.h"

typedef struct PicoMachineState {
    MachineState parent_obj;

    RP2040State soc;
} PicoMachineState;

#define TYPE_RASPBERRYPI_PICO MACHINE_TYPE_NAME("raspberrypi-pico")
OBJECT_DECLARE_SIMPLE_TYPE(PicoMachineState, RASPBERRYPI_PICO)

static void raspberrypi_pico_init(MachineState *machine)
{
    PicoMachineState *s = RASPBERRYPI_PICO(machine);
    Clock *sysclk;

    if (!machine->kernel_filename) {
        error_report("raspberrypi-pico requires -kernel");
        exit(1);
    }

    sysclk = clock_new(OBJECT(machine), "SYSCLK");
    clock_set_hz(sysclk, 125000000ULL);

    object_initialize_child(OBJECT(machine), "soc", &s->soc, TYPE_RP2040_SOC);
    qdev_prop_set_chr(DEVICE(&s->soc), "serial0", serial_hd(0));
    qdev_prop_set_uint32(DEVICE(&s->soc), "num-cpus", machine->smp.cpus);
    qdev_connect_clock_in(DEVICE(&s->soc), "sysclk", sysclk);
    sysbus_realize(SYS_BUS_DEVICE(&s->soc), &error_fatal);

    armv7m_load_kernel(s->soc.cpu[0].cpu, machine->kernel_filename,
                       RP2040_XIP_BASE, RP2040_XIP_SIZE);
    armv7m_load_kernel(s->soc.cpu[1].cpu, NULL, 0, 0);
}

static void raspberrypi_pico_machine_class_init(ObjectClass *oc,
                                                const void *data)
{
    MachineClass *mc = MACHINE_CLASS(oc);
    static const char *const valid_cpu_types[] = {
        ARM_CPU_TYPE_NAME("cortex-m0"),
        NULL,
    };

    mc->desc = "Raspberry Pi Pico (RP2040)";
    mc->init = raspberrypi_pico_init;
    mc->valid_cpu_types = valid_cpu_types;
    mc->max_cpus = RP2040_NUM_CORES;
    mc->default_cpus = RP2040_NUM_CORES;
    mc->default_ram_size = 0;
}

static const TypeInfo raspberrypi_pico_machine_type = {
    .name = TYPE_RASPBERRYPI_PICO,
    .parent = TYPE_MACHINE,
    .instance_size = sizeof(PicoMachineState),
    .class_init = raspberrypi_pico_machine_class_init,
    .interfaces = arm_machine_interfaces,
};

static void raspberrypi_pico_machine_init(void)
{
    type_register_static(&raspberrypi_pico_machine_type);
}

type_init(raspberrypi_pico_machine_init)
