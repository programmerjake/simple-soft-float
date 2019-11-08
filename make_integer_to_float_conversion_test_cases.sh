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
    local value="$1"
    local op="$2"
    local rounding_mode="$3"
    local tininess_detection_mode="$4"
    local src_width
    local dest_width
    local src_sign_mask=0
    local sf_op="$op"
    case "$op" in
    i32_to_f16)
        src_width=32
        dest_width=16
        src_sign_mask=0x80000000
        ;;
    u32_to_f16)
        src_width=32
        dest_width=16
        sf_op=ui32_to_f16
        ;;
    i64_to_f16)
        src_width=64
        dest_width=16
        src_sign_mask=0x8000000000000000
        ;;
    u64_to_f16)
        src_width=64
        dest_width=16
        sf_op=ui64_to_f16
        ;;
    i32_to_f32)
        src_width=32
        dest_width=32
        src_sign_mask=0x80000000
        ;;
    u32_to_f32)
        src_width=32
        dest_width=32
        sf_op=ui32_to_f32
        ;;
    i64_to_f32)
        src_width=64
        dest_width=32
        src_sign_mask=0x8000000000000000
        ;;
    u64_to_f32)
        src_width=64
        dest_width=32
        sf_op=ui64_to_f32
        ;;
    *)
        fail "op not implemented: $op"
        ;;
    esac
    local src_mask
    case "$src_width" in
    32)
        src_mask=0xFFFFFFFF
        ;;
    64)
        src_mask=0xFFFFFFFFFFFFFFFF
        ;;
    *)
        fail "src_width not implemented: $src_width"
        ;;
    esac
    local hex_value
    printf -v hex_value "0x%X" $((value & src_mask))
    if ((value & src_sign_mask)); then
        printf -v value "%s0x%X" '-' $((-value & src_mask))
    else
        printf -v value "0x%X" $((value & src_mask))
    fi
    local sf_rounding_mode
    case "$rounding_mode" in
    TiesToEven)
        sf_rounding_mode=near_even
        ;;
    TowardZero)
        sf_rounding_mode=minMag
        ;;
    TowardNegative)
        sf_rounding_mode=min
        ;;
    TowardPositive)
        sf_rounding_mode=max
        ;;
    TiesToAway)
        sf_rounding_mode=near_maxMag
        ;;
    *)
        fail "invalid rounding mode: $rounding_mode"
        ;;
    esac
    local input="softfloat_round_$sf_rounding_mode softfloat_roundingMode_write_helper"
    input+=" 0 softfloat_exceptionFlags_write_helper"
    input+=" softfloat_tininess_${tininess_detection_mode,} softfloat_detectTininess_write_helper"
    input+=" $hex_value"
    input+=" $sf_op"
    input+=" softfloat_exceptionFlags_read_helper"
    input+=" softfloat_flag_inexact"
    input+=" softfloat_flag_underflow"
    input+=" softfloat_flag_overflow"
    input+=" softfloat_flag_infinite"
    input+=" softfloat_flag_invalid"
    local output
    output=(`echo "$input" | "$SOFTFLOAT_VERIFY"`) || fail $'softfloat-verify failed. input:\n'"$input"
    ((${#output[@]} == 7)) || fail $'softfloat-verify returned invalid number of outputs. input:\n'"$input"
    local result="${output[0]}"
    local flags="${output[1]}"
    local flag_inexact="${output[2]}"
    local flag_underflow="${output[3]}"
    local flag_overflow="${output[4]}"
    local flag_infinite="${output[5]}"
    local flag_invalid="${output[6]}"
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
    case "$dest_width" in
    16)
        if (((result & 0x7C00) == 0x7C00 && (result & 0x3FF) != 0)); then
            result=0x7E00
        fi
        ;;
    32)
        if (((result & 0x7F800000) == 0x7F800000 && (result & 0x7FFFFF) != 0)); then
            result=0x7FC00000
        fi
        ;;
    *)
        fail "dest_width not implemented: $dest_width"
        ;;
    esac
    printf -v result "0x%0*X" $((dest_width / 4)) $((result))
    echo "$value $rounding_mode $tininess_detection_mode $result $flags"
}

test_case_list=(0x0)
test_case_list+=(0x1 0x2 0x3 0x4 0x5 0x6 0x7 0x7FF 0x800 0x801 0xFFE0 0xFFE1 0xFFEF 0xFFF0 0xFFF1)
test_case_list+=(0x7FFFFFFF 0x80000000 0x7FFFFFFFFFFFFFFF 0x8000000000000000)
test_case_list+=(-0x1 -0x2 -0x3 -0x4 -0x5 -0x6 -0x7 -0x7FF -0x800 -0x801 -0xFFE0 -0xFFE1 -0xFFEF -0xFFF0 -0xFFF1)
test_case_list+=(-0x7FFFFFFF -0x80000000 -0x7FFFFFFFFFFFFFFF -0x8000000000000000)
ops=(i32_to_f16 u32_to_f16 i64_to_f16 u64_to_f16 i32_to_f32 u32_to_f32 i64_to_f32 u64_to_f32)
rounding_modes=(TiesToEven TowardZero TowardNegative TowardPositive TiesToAway)
tininess_detection_modes=(BeforeRounding AfterRounding)

for op in "${ops[@]}"; do
    exec > "test_data/$op.txt"
    first=1
    for rounding_mode in "${rounding_modes[@]}"; do
        for tininess_detection_mode in "${tininess_detection_modes[@]}"; do
            if ((first)); then
                first=0
            else
                echo
            fi
            echo "# testing $op with $rounding_mode $tininess_detection_mode"
            for value in "${test_case_list[@]}"; do
                write_test_case $value $op $rounding_mode $tininess_detection_mode
            done
            printf "." >&2
        done
    done &
done
wait
echo >&2
