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
    local exact="$3"
    local rounding_mode="$4"
    local src_width
    local dest_width
    local sf_op
    local dest_sign_mask=0
    case "$op" in
    f16_to_u32)
        src_width=16
        dest_width=32
        sf_op=f16_to_ui32
        ;;
    f16_to_i32)
        src_width=16
        dest_width=32
        sf_op=f16_to_i32
        dest_sign_mask=0x80000000
        ;;
    f16_to_u64)
        src_width=16
        dest_width=64
        sf_op=f16_to_ui64
        ;;
    f16_to_i64)
        src_width=16
        dest_width=64
        sf_op=f16_to_i64
        dest_sign_mask=0x8000000000000000
        ;;
    f32_to_u32)
        src_width=32
        dest_width=32
        sf_op=f32_to_ui32
        ;;
    f32_to_i32)
        src_width=32
        dest_width=32
        sf_op=f32_to_i32
        dest_sign_mask=0x80000000
        ;;
    f32_to_u64)
        src_width=32
        dest_width=64
        sf_op=f32_to_ui64
        ;;
    f32_to_i64)
        src_width=32
        dest_width=64
        sf_op=f32_to_i64
        dest_sign_mask=0x8000000000000000
        ;;
    *)
        fail "op not implemented: $op"
        ;;
    esac
    printf -v value "0x%0*X" $((src_width / 4)) $((value))
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
    local sf_exact
    case "$exact" in
    false)
        sf_exact=0
        ;;
    true)
        sf_exact=1
        ;;
    *)
        fail "invalid exact flag: $exact"
        ;;
    esac
    local input="softfloat_round_$sf_rounding_mode softfloat_roundingMode_write_helper"
    input+=" 0 softfloat_exceptionFlags_write_helper"
    input+=" $value softfloat_round_$sf_rounding_mode $sf_exact $sf_op"
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
    if ((flags & flag_invalid)); then
        result="None"
    elif ((result & dest_sign_mask)); then
        printf -v result "%s0x%X" '-' $((-result))
    else
        printf -v result "0x%X" $((result))
    fi
    if (( ${#decoded_flags[@]} )); then
        printf -v flags "%s|" "${decoded_flags[@]}"
        flags="${flags%%|}"
    else
        flags="(empty)"
    fi
    echo "$value $exact $rounding_mode $result $flags"
}

f16_test_case_list=(0x0000 0x0001 0x03FF 0x0400 0x3C00 0x3C01 0x7BFF 0x7C00 0x7C01 0x7DFF 0x7E00 0x7FFF)
f16_test_case_list+=(0x8000 0x8001 0x83FF 0x8400 0xBC00 0xBC01 0xFBFF 0xFC00 0xFC01 0xFDFF 0xFE00 0xFFFF)
f16_test_case_list+=(0x3400 0x3800 0x3A00 0x3C00 0x3D00 0x3E00 0x3F00 0x4000 0x4080 0x4100 0x4180 0x4200)
f16_test_case_list+=(0xB400 0xB800 0xBA00 0xBC00 0xBD00 0xBE00 0xBF00 0xC000 0xC080 0xC100 0xC180 0xC200)
f32_test_case_list=(0x00000000 0x00000001 0x007FFFFF 0x00800000 0x3F800000 0x3F800001 0x7F7FFFFF 0x7F800000 0x7F800001 0x7FBFFFFF 0x7FC00000 0x7FFFFFFF)
f32_test_case_list+=(0x80000000 0x80000001 0x807FFFFF 0x80800000 0xBF800000 0xBF800001 0xFF7FFFFF 0xFF800000 0xFF800001 0xFFBFFFFF 0xFFC00000 0xFFFFFFFF)
f32_test_case_list+=(0x3E800000 0x3F000000 0x3F400000 0x3F800000 0x3FA00000 0x3FC00000 0x3FE00000 0x40000000 0x40100000 0x40200000 0x40300000 0x40400000)
f32_test_case_list+=(0xBE800000 0xBF000000 0xBF400000 0xBF800000 0xBFA00000 0xBFC00000 0xBFE00000 0xC0000000 0xC0100000 0xC0200000 0xC0300000 0xC0400000)
f32_test_case_list+=(0x33800000 0x387FC000 0x38800000 0x3F800000 0x3F802000 0x477FE000 0x47800000)
f32_test_case_list+=(0xB3800000 0xB87FC000 0xB8800000 0xBF800000 0xBF802000 0xC77FE000 0xC7800000)
f32_test_case_list+=(0x4EFFFFFF 0x4F000000 0x4F000001 0x4F7FFFFF 0x4F800000 0x4F800001 0x5EFFFFFF 0x5F000000 0x5F000001 0x5F7FFFFF 0x5F800000 0x5F800001)
f32_test_case_list+=(0xCEFFFFFF 0xCF000000 0xCF000001 0xCF7FFFFF 0xCF800000 0xCF800001 0xDEFFFFFF 0xDF000000 0xDF000001 0xDF7FFFFF 0xDF800000 0xDF800001)
ops=(f16_to_u32 f16_to_i32 f16_to_u64 f16_to_i64)
ops+=(f32_to_u32 f32_to_i32 f32_to_u64 f32_to_i64)
rounding_modes=(TiesToEven TowardZero TowardNegative TowardPositive TiesToAway)

for op in "${ops[@]}"; do
    exec > "test_data/$op.txt"
    first=1
    for rounding_mode in "${rounding_modes[@]}"; do
        for exact in false true; do
            if ((first)); then
                first=0
            else
                echo
            fi
            echo "# testing $op with exact=$exact $rounding_mode"
            case "$op" in
            f16_to_*)
                test_case_list=("${f16_test_case_list[@]}")
                ;;
            f32_to_*)
                test_case_list=("${f32_test_case_list[@]}")
                ;;
            *)
                fail "op not implemented: $op"
                ;;
            esac
            for value in "${test_case_list[@]}"; do
                write_test_case $value $op $exact $rounding_mode
            done
            printf "." >&2
        done
    done &
done
wait
echo >&2
