#!/bin/bash
# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information
set -e

if [[ -z "$SOFTFLOAT_VERIFY" ]] && ! SOFTFLOAT_VERIFY="`which softfloat-verify`"; then
    echo "can't find softfloat-verify in PATH" >&2
    echo "get it from https://salsa.debian.org/Kazan-team/softfloat-verify" >&2
    echo "then put built executable in PATH or set" >&2
    echo "SOFTFLOAT_VERIFY to path of executable" >&2
    exit 1
fi

function fail() {
    echo "$*">&2
    exit 1
}

function write_test_case() {
    local value1="$1"
    local value2="$2"
    local op="$3"
    printf -v value1 "0x%04X" $((value1))
    printf -v value2 "0x%04X" $((value2))
    local input=" 0 softfloat_exceptionFlags_write_helper"
    case "$op" in
    compare_quiet)
        input+=" $value1 $value2 f16_eq"
        input+=" $value1 $value2 f16_lt_quiet"
        input+=" $value2 $value1 f16_lt_quiet"
        ;;
    compare_signaling)
        input+=" $value1 $value2 f16_eq_signaling"
        input+=" $value1 $value2 f16_lt"
        input+=" $value2 $value1 f16_lt"
        ;;
    *)
        fail "op not implemented: $op"
        ;;
    esac
    input+=" softfloat_exceptionFlags_read_helper"
    input+=" softfloat_flag_inexact"
    input+=" softfloat_flag_underflow"
    input+=" softfloat_flag_overflow"
    input+=" softfloat_flag_infinite"
    input+=" softfloat_flag_invalid"
    local output
    output=(`echo "$input" | "$SOFTFLOAT_VERIFY"`) || fail $'softfloat-verify failed. input:\n'"$input"
    ((${#output[@]} == 9)) || fail $'softfloat-verify returned invalid number of outputs. input:\n'"$input"
    local result_eq="${output[0]}"
    local result_lt="${output[1]}"
    local result_gt="${output[2]}"
    local flags="${output[3]}"
    local flag_inexact="${output[4]}"
    local flag_underflow="${output[5]}"
    local flag_overflow="${output[6]}"
    local flag_infinite="${output[7]}"
    local flag_invalid="${output[8]}"
    local decoded_flags=()
    ((flags & flag_inexact)) && decoded_flags+=("INEXACT")
    ((flags & flag_underflow)) && decoded_flags+=("UNDERFLOW")
    ((flags & flag_overflow)) && decoded_flags+=("OVERFLOW")
    ((flags & flag_infinite)) && decoded_flags+=("DIVISION_BY_ZERO")
    ((flags & flag_invalid)) && decoded_flags+=("INVALID_OPERATION")
    if (( ${#decoded_flags[@]} )); then
        printf -v flags "%s|" "${decoded_flags[@]}"
        flags="${flags%%|}"
    else
        flags="(empty)"
    fi
    local result="Unordered"
    if ((result_eq)); then
        result="Equal"
    elif ((result_lt)); then
        result="Less"
    elif ((result_gt)); then
        result="Greater"
    fi
    echo "$value1 $value2 $result $flags"
}

test_case_list=(0x0000 0x0001 0x03FF 0x0400 0x3C00 0x3C01 0x7BFF 0x7C00 0x7C01 0x7DFF 0x7E00 0x7FFF)
test_case_list+=(0x8000 0x8001 0x83FF 0x8400 0xBC00 0xBC01 0xFBFF 0xFC00 0xFC01 0xFDFF 0xFE00 0xFFFF)
ops=(compare_quiet compare_signaling)

for op in "${ops[@]}"; do
    exec > "test_data/$op.txt"
    first=1
    for value1 in "${test_case_list[@]}"; do
        if ((first)); then
            first=0
        else
            echo
        fi
        echo "# testing F16::$op($value1, X)"
        for value2 in "${test_case_list[@]}"; do
            write_test_case $value1 $value2 $op
        done
        printf "." >&2
    done &
done
wait
echo >&2
