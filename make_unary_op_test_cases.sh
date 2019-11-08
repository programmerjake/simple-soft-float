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
    local op="$2"
    local rounding_mode="$3"
    local tininess_detection_mode="$4"
    printf -v value1 "0x%04X" $((value1))
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
    if [[ "$op" == "round_to_integral" ]]; then
        input+=" $value1 softfloat_round_$sf_rounding_mode 0 f16_roundToInt"
    elif [[ "$op" == "round_to_integral_exact" ]]; then
        input+=" $value1 softfloat_round_$sf_rounding_mode 1 f16_roundToInt"
    elif [[ "$op" == "rsqrt" ]]; then
        # known to work for cases in test_case_list
        input+=" 1 i32_to_f128 $value1 f16_to_f128 f128_sqrt f128_div f128_to_f16"
    else
        input+=" $value1 f16_$op"
    fi
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
    if (((result & 0x7C00) == 0x7C00 && (result & 0x3FF) != 0)); then
        result=0x7E00
    fi
    printf -v result "0x%04X" $((result))
    echo "$value1 $rounding_mode $tininess_detection_mode $result $flags"
}

test_case_list=(0x0000 0x0001 0x03FF 0x0400 0x3C00 0x3C01 0x7BFF 0x7C00 0x7C01 0x7DFF 0x7E00 0x7FFF)
test_case_list+=(0x8000 0x8001 0x83FF 0x8400 0xBC00 0xBC01 0xFBFF 0xFC00 0xFC01 0xFDFF 0xFE00 0xFFFF)
test_case_list+=(0x3400 0x3800 0x3A00 0x3C00 0x3D00 0x3E00 0x3F00 0x4000 0x4080 0x4100 0x4180 0x4200)
test_case_list+=(0xB400 0xB800 0xBA00 0xBC00 0xBD00 0xBE00 0xBF00 0xC000 0xC080 0xC100 0xC180 0xC200)
ops=(round_to_integral round_to_integral_exact sqrt rsqrt)
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
            for value1 in "${test_case_list[@]}"; do
                write_test_case $value1 $op $rounding_mode $tininess_detection_mode
            done
            printf "." >&2
        done
    done &
done
wait
echo >&2
