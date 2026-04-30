/*
 * Infineon CYW43439 Wi-Fi device model for Raspberry Pi Pico 2 W
 *
 * Copyright (c) 2026
 *
 * SPDX-License-Identifier: GPL-2.0-or-later
 */

#ifndef HW_MISC_CYW43439_WIFI_H
#define HW_MISC_CYW43439_WIFI_H

#include "hw/arm/armv7m.h"
#include "hw/ssi/ssi.h"
#include "qom/object.h"

#define TYPE_CYW43439_WIFI "cyw43439-wifi"
OBJECT_DECLARE_SIMPLE_TYPE(CYW43439WifiState, CYW43439_WIFI)

#define CYW43439_WIFI_FRAME_SIZE 128
#define CYW43439_WIFI_QUEUE_DEPTH 8
#define CYW43439_WIFI_LOAD_HEADER_SIZE 8
#define CYW43439_WIFI_LOAD_CHUNK_HEADER_SIZE 5

typedef struct CYW43439WifiFrame {
    uint8_t dst_node;
    uint8_t len;
    uint8_t bytes[CYW43439_WIFI_FRAME_SIZE];
} CYW43439WifiFrame;

struct CYW43439WifiState {
    SSIPeripheral parent_obj;

    uint8_t selected_node;
    uint8_t mode;
    uint8_t tx_dst_node;
    uint8_t tx_len;
    uint8_t tx_pos;
    uint8_t tx_buf[CYW43439_WIFI_FRAME_SIZE];
    uint8_t rx_len;
    uint8_t rx_pos;
    uint8_t rx_buf[CYW43439_WIFI_FRAME_SIZE];
    uint8_t queue_head;
    uint8_t queue_len;
    uint8_t node_role;
    uint8_t node_id;
    uint8_t node_count;
    uint8_t load_kind;
    uint8_t load_header_pos;
    uint8_t load_chunk_header_pos;
    uint8_t load_chunk_len;
    uint8_t load_chunk_pos;
    uint8_t load_header[CYW43439_WIFI_LOAD_HEADER_SIZE];
    uint8_t load_chunk_header[CYW43439_WIFI_LOAD_CHUNK_HEADER_SIZE];
    uint32_t fw_expected_len;
    uint32_t fw_expected_hash;
    uint32_t fw_received_len;
    uint32_t fw_hash;
    uint32_t clm_expected_len;
    uint32_t clm_expected_hash;
    uint32_t clm_received_len;
    uint32_t clm_hash;
    uint32_t nvram_len;
    uint32_t nvram_hash;
    bool overflow;
    bool powered;
    bool reset_asserted;
    bool probed;
    bool fw_started;
    bool fw_committed;
    bool clm_started;
    bool clm_committed;
    bool nvram_applied;
    bool fw_ready;
    bool fw_error;
    uint16_t radio_local_port;
    uint16_t radio_peer_port;
    uint16_t radio_port_base;
    int radio_fd;
    ARMv7MState *cpu[2];
    struct sockaddr_in radio_peer_addr;
    CYW43439WifiFrame queue[CYW43439_WIFI_QUEUE_DEPTH];
};

#endif
