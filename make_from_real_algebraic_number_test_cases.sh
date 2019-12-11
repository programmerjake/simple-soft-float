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

function write_test_cases() {
    local mantissa_start="$1"
    local mantissa_end="$2"
    local mantissa_step="$3"
    local exponent="$4"
    local rounding_mode="$5"
    local exception_handling_mode="$6"
    local tininess_detection_mode="$7"
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
    local mantissa
    for((mantissa=mantissa_start; mantissa<=mantissa_end; mantissa+=mantissa_step)); do
        local input make_input_value
        printf -v make_input_value "0x%X i64_to_f64 0x%X i64_to_f64 f64_div" $((mantissa << (exponent + 32))) $((1 << 32))
        local input="softfloat_round_$sf_rounding_mode softfloat_roundingMode_write_helper"
        input+=" 0x0400 f16_to_f64 $make_input_value f64_to_f16 f16_to_f64 f64_le"
        input+=" $make_input_value f64_to_f16 f16_to_f64 0x8400 f16_to_f64 f64_le"
        input+=" $make_input_value"
        input+=" 0 softfloat_exceptionFlags_write_helper"
        input+=" softfloat_tininess_${tininess_detection_mode,} softfloat_detectTininess_write_helper"
        input+=" f64_to_f16"
        input+=" softfloat_exceptionFlags_read_helper"
        input+=" softfloat_flag_inexact"
        input+=" softfloat_flag_underflow"
        input+=" softfloat_flag_overflow"
        input+=" softfloat_flag_infinite"
        input+=" softfloat_flag_invalid"
        local output
        output=(`echo "$input" | "$SOFTFLOAT_VERIFY"`) || fail $'softfloat-verify failed. input:\n'"$input"
        ((${#output[@]} == 9)) || fail $'softfloat-verify returned invalid number of outputs. input:\n'"$input"
        local gt_min_normal="${output[0]}"
        local le_neg_min_normal="${output[1]}"
        local result="${output[2]}"
        local flags="${output[3]}"
        local flag_inexact="${output[4]}"
        local flag_underflow="${output[5]}"
        local flag_overflow="${output[6]}"
        local flag_infinite="${output[7]}"
        local flag_invalid="${output[8]}"
        if [[ "$exception_handling_mode" != "IgnoreExactUnderflow" ]] && ((mantissa != 0 && !gt_min_normal && !le_neg_min_normal)); then
            ((flags |= flag_underflow))
        fi
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
        local mantissa_str
        if ((mantissa < 0)); then
            printf -v mantissa_str "%s0x%X" '-' $((-mantissa))
        else
            printf -v mantissa_str "0x%X" $mantissa
        fi
        printf -v result "0x%04X" $((result))
        echo "$mantissa_str $exponent $rounding_mode $exception_handling_mode $tininess_detection_mode $result $flags"
    done
}

exec > "test_data/from_real_algebraic_number.txt"
for rounding_mode in TiesToEven TowardZero TowardNegative TowardPositive TiesToAway; do
    lc_rounding_mode="`echo "$rounding_mode" | sed 's/\([^A-Z]\)\([A-Z]\)/\1_\2/g; s/.*/\L&/'`"
    for tininess_detection_mode in BeforeRounding AfterRounding; do
        for exception_handling_mode in IgnoreExactUnderflow SignalExactUnderflow; do
            echo "# test the values right around zero for $rounding_mode $exception_handling_mode $tininess_detection_mode"
            write_test_cases -0x20 0x20 0x4 -28 $rounding_mode $exception_handling_mode $tininess_detection_mode
            echo
            echo "# test the values at the transition between subnormal and normal for $rounding_mode $exception_handling_mode $tininess_detection_mode"
            write_test_cases 0x3FE0 0x4020 0x2 -28 $rounding_mode $exception_handling_mode $tininess_detection_mode
            write_test_cases -0x4020 -0x3FE0 0x2 -28 $rounding_mode $exception_handling_mode $tininess_detection_mode
            echo
            echo -n '.' >&2
        done
    done
    echo "# test the values right around 1 and -1 for $rounding_mode"
    write_test_cases -0x4020 -0x3FE0 0x4 -14 $rounding_mode IgnoreExactUnderflow BeforeRounding
    write_test_cases 0x3FE0 0x4020 0x4 -14 $rounding_mode IgnoreExactUnderflow BeforeRounding
    echo
    echo "# test the values right around max normal for $rounding_mode"
    write_test_cases -0x4010 -0x3FF0 0x2 2 $rounding_mode IgnoreExactUnderflow BeforeRounding
    write_test_cases 0x3FF0 0x4010 0x2 2 $rounding_mode IgnoreExactUnderflow BeforeRounding
    echo
    echo "# test the values much larger than max normal for $rounding_mode"
    write_test_cases -1 -1 1 20 $rounding_mode IgnoreExactUnderflow BeforeRounding
    write_test_cases 1 1 1 20 $rounding_mode IgnoreExactUnderflow BeforeRounding
    echo
done
echo >&2
