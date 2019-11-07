#!/bin/bash
# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information
set -e

test_case_list=(0x0000 0x0001 0x03FF 0x0400 0x3C00 0x3C01 0x7BFF 0x7C00 0x7C01 0x7DFF 0x7E00 0x7FFF)
test_case_list+=(0x8000 0x8001 0x83FF 0x8400 0xBC00 0xBC01 0xFBFF 0xFC00 0xFC01 0xFDFF 0xFE00 0xFFFF)
test_case_list+=(0x3400 0x3800 0x3A00 0x3C00 0x3D00 0x3E00 0x3F00 0x4000 0x4080 0x4100 0x4180 0x4200)
test_case_list+=(0xB400 0xB800 0xBA00 0xBC00 0xBD00 0xBE00 0xBF00 0xC000 0xC080 0xC100 0xC180 0xC200)

exec > "test_data/next_up_or_down.txt"
for value in "${test_case_list[@]}"; do
    status_flags="(empty)"
    if (((value & 0x7C00) == 0x7C00 && (value & 0x3FF) != 0)); then
        if (((value & 0x200) == 0)); then
            status_flags="INVALID_OPERATION"
        fi
        down_result=0x7E00
        up_result=0x7E00
    elif ((value == 0x7C00)); then
        down_result=0x7BFF
        up_result=0x7C00
    elif ((value == 0xFC00)); then
        down_result=0xFC00
        up_result=0xFBFF
    elif ((value == 0x8000 || value == 0x0000)); then
        down_result=0x8001
        up_result=0x0001
    elif ((value & 0x8000)); then
        down_result=$((value + 1))
        up_result=$((value - 1))
    else
        down_result=$((value - 1))
        up_result=$((value + 1))
    fi
    printf -v down_result "0x%04X" "$down_result"
    printf -v up_result "0x%04X" "$up_result"
    echo "$value $up_result $status_flags $down_result $status_flags"
done
