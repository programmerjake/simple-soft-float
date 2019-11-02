#!/bin/bash
# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information
set -e

if [[ -z "$SOFTFLOAT_VERIFY" ]]; then
    SOFTFLOAT_VERIFY="`which softfloat-verify`"
fi
if [[ -z "$SOFTFLOAT_VERIFY" ]]; then
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
    local rounding_mode="$4"
    local tininess_detection_mode="$5"
    printf -v value1 "0x%04X" $((value1))
    printf -v value2 "0x%04X" $((value2))
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
    input+=" $value1 $value2 f16_$op"
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
        printf -v flags "%s | " "${decoded_flags[@]}"
        flags="${flags%% | }"
    else
        flags="StatusFlags::empty()"
    fi
    if (((result & 0x7C00) == 0x7C00 && (result & 0x3FF) != 0)); then
        result=0x7E00
    fi
    printf -v result "0x%04X" $((result))
    echo "    binary_test_case!($value1, $value2, $op, $rounding_mode, $tininess_detection_mode, $result, $flags);"
}

test_case_list=(0x0000 0x0001 0x03FF 0x0400 0x3C00 0x3C01 0x7BFF 0x7C00 0x7C01 0x7DFF 0x7E00 0x7FFF)
test_case_list+=(0x8000 0x8001 0x83FF 0x8400 0xBC00 0xBC01 0xFBFF 0xFC00 0xFC01 0xFDFF 0xFE00 0xFFFF)

total=0
for op in add sub; do
    for rounding_mode in TiesToEven TowardZero TowardNegative TowardPositive TiesToAway; do
        for tininess_detection_mode in BeforeRounding AfterRounding; do
            for value1 in "${test_case_list[@]}"; do
                ((total += 1))
            done
        done
    done
done

step=0
for op in add sub; do
    for rounding_mode in TiesToEven TowardZero TowardNegative TowardPositive TiesToAway; do
        echo "#[test]"
        lc_rounding_mode="`echo "$rounding_mode" | sed 's/\([^A-Z]\)\([A-Z]\)/\1_\2/g; s/.*/\L&/'`"
        echo "fn test_${op}_$lc_rounding_mode() {"
        first=1
        for tininess_detection_mode in BeforeRounding AfterRounding; do
            for value1 in "${test_case_list[@]}"; do
                if ((first)); then
                    first=0
                else
                    echo
                fi
                echo "    // testing F16::$op($value1, X) with $rounding_mode $tininess_detection_mode"
                for value2 in "${test_case_list[@]}"; do
                    write_test_case $value1 $value2 $op $rounding_mode $tininess_detection_mode
                done
                ((step += 1))
                [ -t 1 ] || printf "\rworking %i%%...\x1b[K" $(((step * 100) / total)) >&2
            done
        done
        echo "}"
        echo
    done
done
[ -t 1 ] || printf "\rdone.\x1b[K\n" >&2
