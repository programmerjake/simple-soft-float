# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information

import simple_soft_float as ssf
import unittest
import operator
import inspect


def check_signatures(test_case, cls):
    test_case.assertIsNotNone(inspect.signature(cls))
    for name, member in cls.__dict__.items():
        if name.startswith("_") or not callable(member):
            continue
        test_case.assertIsNotNone(inspect.signature(member))


class TestBinaryNaNPropagationMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.BinaryNaNPropagationMode
        self.assertEqual(set(cls),
                         {cls.AlwaysCanonical,
                          cls.FirstSecond,
                          cls.SecondFirst,
                          cls.FirstSecondPreferringSNaN,
                          cls.SecondFirstPreferringSNaN})


class TestExceptionHandlingMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.ExceptionHandlingMode
        self.assertEqual(set(cls),
                         {cls.IgnoreExactUnderflow,
                          cls.SignalExactUnderflow})


class TestFMAInfZeroQNaNResult(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.FMAInfZeroQNaNResult
        self.assertEqual(set(cls),
                         {cls.FollowNaNPropagationMode,
                          cls.CanonicalAndGenerateInvalid,
                          cls.PropagateAndGenerateInvalid})


class TestFloatClass(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.FloatClass
        self.assertEqual(set(cls),
                         {cls.NegativeInfinity,
                          cls.NegativeNormal,
                          cls.NegativeSubnormal,
                          cls.NegativeZero,
                          cls.PositiveInfinity,
                          cls.PositiveNormal,
                          cls.PositiveSubnormal,
                          cls.PositiveZero,
                          cls.QuietNaN,
                          cls.SignalingNaN})


class TestFloatToFloatConversionNaNPropagationMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.FloatToFloatConversionNaNPropagationMode
        self.assertEqual(set(cls),
                         {cls.AlwaysCanonical,
                          cls.RetainMostSignificantBits})


class TestQuietNaNFormat(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.QuietNaNFormat
        self.assertEqual(set(cls),
                         {cls.Standard,
                          cls.MIPSLegacy})


class TestRoundingMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.RoundingMode
        self.assertEqual(set(cls),
                         {cls.TiesToEven,
                          cls.TowardZero,
                          cls.TowardNegative,
                          cls.TowardPositive,
                          cls.TiesToAway})


class TestSign(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.Sign
        self.assertEqual(set(cls),
                         {cls.Positive,
                          cls.Negative})


class TestTernaryNaNPropagationMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.TernaryNaNPropagationMode
        self.assertEqual(set(cls),
                         {cls.AlwaysCanonical,
                          cls.FirstSecondThird,
                          cls.FirstThirdSecond,
                          cls.SecondFirstThird,
                          cls.SecondThirdFirst,
                          cls.ThirdFirstSecond,
                          cls.ThirdSecondFirst,
                          cls.FirstSecondThirdPreferringSNaN,
                          cls.FirstThirdSecondPreferringSNaN,
                          cls.SecondFirstThirdPreferringSNaN,
                          cls.SecondThirdFirstPreferringSNaN,
                          cls.ThirdFirstSecondPreferringSNaN,
                          cls.ThirdSecondFirstPreferringSNaN})


class TestTininessDetectionMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.TininessDetectionMode
        self.assertEqual(set(cls),
                         {cls.BeforeRounding,
                          cls.AfterRounding})


class TestUnaryNaNPropagationMode(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.UnaryNaNPropagationMode
        self.assertEqual(set(cls),
                         {cls.AlwaysCanonical,
                          cls.First})


class TestUpOrDown(unittest.TestCase):
    maxDiff = None

    def test_enumerants(self):
        cls = ssf.UpOrDown
        self.assertEqual(set(cls),
                         {cls.Up,
                          cls.Down})


class TestFPState(unittest.TestCase):
    maxDiff = None

    def test_signatures(self):
        check_signatures(self, ssf.FPState)

    def test_smoke_test(self):
        rounding_mode = ssf.RoundingMode.TiesToEven
        status_flags = ssf.StatusFlags(0)
        exception_handling_mode = ssf.ExceptionHandlingMode \
            .IgnoreExactUnderflow
        tininess_detection_mode = ssf.TininessDetectionMode.AfterRounding
        obj = ssf.FPState(rounding_mode=rounding_mode,
                          status_flags=status_flags,
                          exception_handling_mode=exception_handling_mode,
                          tininess_detection_mode=tininess_detection_mode)
        obj = obj.merge(obj)
        self.assertEqual(obj.rounding_mode, rounding_mode)
        self.assertEqual(obj.status_flags, status_flags)
        self.assertEqual(obj.exception_handling_mode, exception_handling_mode)
        self.assertEqual(obj.tininess_detection_mode, tininess_detection_mode)
        self.assertEqual(
            repr(obj),
            "PlatformProperties(rounding_mode=RoundingMode.TiesToEven, "
            + "status_flags=StatusFlags(0), "
            + "exception_handling_mode="
            + "ExceptionHandlingMode.IgnoreExactUnderflow, "
            + "tininess_detection_mode=TininessDetectionMode.AfterRounding)")


class TestFloatProperties(unittest.TestCase):
    maxDiff = None

    def test_signatures(self):
        check_signatures(self, ssf.FloatProperties)

    def test_smoke_test(self):
        obj = ssf.FloatProperties(
            exponent_width=8,
            mantissa_width=23,
            has_implicit_leading_bit=True,
            has_sign_bit=True,
            platform_properties=ssf.PlatformProperties_RISC_V)
        obj = ssf.FloatProperties.standard(32)
        self.assertEqual(obj.is_standard, True)
        self.assertEqual(obj.exponent_width, 8)
        self.assertEqual(obj.mantissa_width, 23)
        self.assertEqual(obj.has_implicit_leading_bit, True)
        self.assertEqual(obj.has_sign_bit, True)
        self.assertEqual(obj.platform_properties,
                         ssf.PlatformProperties_RISC_V)
        self.assertEqual(obj.quiet_nan_format, ssf.QuietNaNFormat.Standard)
        self.assertEqual(obj.width, 32)
        self.assertEqual(obj.fraction_width, 23)
        self.assertEqual(obj.sign_field_shift, 31)
        self.assertEqual(obj.sign_field_mask, 0x80000000)
        self.assertEqual(obj.exponent_field_shift, 23)
        self.assertEqual(obj.exponent_field_mask, 0x7F800000)
        self.assertEqual(obj.mantissa_field_shift, 0)
        self.assertEqual(obj.mantissa_field_mask, 0x007FFFFF)
        self.assertEqual(obj.mantissa_field_max, 0x007FFFFF)
        self.assertEqual(obj.mantissa_field_normal_min, 0x00000000)
        self.assertEqual(obj.mantissa_field_msb_shift, 22)
        self.assertEqual(obj.mantissa_field_msb_mask, 0x00400000)
        self.assertEqual(obj.exponent_bias, 0x7F)
        self.assertEqual(obj.exponent_inf_nan, 0xFF)
        self.assertEqual(obj.exponent_zero_subnormal, 0)
        self.assertEqual(obj.exponent_min_normal, 1)
        self.assertEqual(obj.exponent_max_normal, 0xFE)
        self.assertEqual(obj.overall_mask, 0xFFFFFFFF)
        self.assertEqual(repr(obj),
                         "FloatProperties.standard(32, "
                         + "PlatformProperties_RISC_V)")


class TestPlatformProperties(unittest.TestCase):
    maxDiff = None

    def test_signatures(self):
        check_signatures(self, ssf.PlatformProperties)

    def test_constructor_signature(self):
        cls = ssf.PlatformProperties
        signature = inspect.signature(ssf.PlatformProperties)
        parameters = list(signature.parameters.values())
        parameters_set = set()
        for i in range(len(parameters)):
            parameter = parameters[i]
            if i == 0:
                self.assertEqual(parameter.name, "value")
                self.assertEqual(parameter.kind,
                                 inspect.Parameter.POSITIONAL_OR_KEYWORD)
                self.assertIsNone(parameter.default)
            else:
                parameters_set.add(parameter.name)
                self.assertEqual(parameter.kind,
                                 inspect.Parameter.KEYWORD_ONLY)
                self.assertIsNone(parameter.default)
        members_set = set()
        for name, member in cls.__dict__.items():
            if name.startswith("_") or callable(member):
                continue
            members_set.add(name)
        members_set.discard("quiet_nan_format")
        self.assertEqual(parameters_set, members_set)

    def test_smoke_test(self):
        self.assertIsInstance(ssf.PlatformProperties_ARM,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_RISC_V,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_POWER,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_MIPS_2008,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_X86_SSE,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_SPARC,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_HPPA,
                              ssf.PlatformProperties)
        self.assertIsInstance(ssf.PlatformProperties_MIPS_LEGACY,
                              ssf.PlatformProperties)
        obj = ssf.PlatformProperties_RISC_V
        self.assertEqual(obj.fma_inf_zero_qnan_result,
                         ssf.FMAInfZeroQNaNResult.CanonicalAndGenerateInvalid)
        obj = ssf.PlatformProperties(
            obj,
            fma_inf_zero_qnan_result=ssf
            .FMAInfZeroQNaNResult.FollowNaNPropagationMode)
        self.assertEqual(obj.canonical_nan_sign, ssf.Sign.Positive)
        self.assertEqual(obj.canonical_nan_mantissa_msb, True)
        self.assertEqual(obj.canonical_nan_mantissa_second_to_msb, False)
        self.assertEqual(obj.canonical_nan_mantissa_rest, False)
        self.assertEqual(obj.std_bin_ops_nan_propagation_mode,
                         ssf.BinaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.fma_nan_propagation_mode,
                         ssf.TernaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.fma_inf_zero_qnan_result,
                         ssf.FMAInfZeroQNaNResult.FollowNaNPropagationMode)
        self.assertEqual(obj.round_to_integral_nan_propagation_mode,
                         ssf.UnaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.next_up_or_down_nan_propagation_mode,
                         ssf.UnaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.scale_b_nan_propagation_mode,
                         ssf.UnaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.sqrt_nan_propagation_mode,
                         ssf.UnaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(
            obj.float_to_float_conversion_nan_propagation_mode,
            ssf.FloatToFloatConversionNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.rsqrt_nan_propagation_mode,
                         ssf.UnaryNaNPropagationMode.AlwaysCanonical)
        self.assertEqual(obj.quiet_nan_format,
                         ssf.QuietNaNFormat.Standard)
        self.assertEqual(
            repr(obj),
            "PlatformProperties(canonical_nan_sign=Sign.Positive, "
            + "canonical_nan_mantissa_msb=True, "
            + "canonical_nan_mantissa_second_to_msb=False, "
            + "canonical_nan_mantissa_rest=False, "
            + "std_bin_ops_nan_propagation_mode="
            + "BinaryNaNPropagationMode.AlwaysCanonical, "
            + "fma_nan_propagation_mode="
            + "TernaryNaNPropagationMode.AlwaysCanonical, "
            + "fma_inf_zero_qnan_result="
            + "FMAInfZeroQNaNResult.FollowNaNPropagationMode, "
            + "round_to_integral_nan_propagation_mode="
            + "UnaryNaNPropagationMode.AlwaysCanonical, "
            + "next_up_or_down_nan_propagation_mode="
            + "UnaryNaNPropagationMode.AlwaysCanonical, "
            + "scale_b_nan_propagation_mode="
            + "UnaryNaNPropagationMode.AlwaysCanonical, "
            + "sqrt_nan_propagation_mode="
            + "UnaryNaNPropagationMode.AlwaysCanonical, "
            + "float_to_float_conversion_nan_propagation_mode="
            + "FloatToFloatConversionNaNPropagationMode.AlwaysCanonical, "
            + "rsqrt_nan_propagation_mode="
            + "UnaryNaNPropagationMode.AlwaysCanonical)")


class TestStatusFlags(unittest.TestCase):
    maxDiff = None

    def test_smoke_test(self):
        self.assertIsInstance(ssf.StatusFlags.INVALID_OPERATION,
                              ssf.StatusFlags)
        self.assertIsInstance(ssf.StatusFlags.DIVISION_BY_ZERO,
                              ssf.StatusFlags)
        self.assertIsInstance(ssf.StatusFlags.OVERFLOW,
                              ssf.StatusFlags)
        self.assertIsInstance(ssf.StatusFlags.UNDERFLOW,
                              ssf.StatusFlags)
        self.assertIsInstance(ssf.StatusFlags.INEXACT,
                              ssf.StatusFlags)
        self.assertIsInstance(ssf.StatusFlags(0),
                              ssf.StatusFlags)
        self.assertEqual(
            ssf.StatusFlags.INVALID_OPERATION
            | ssf.StatusFlags.DIVISION_BY_ZERO
            | ssf.StatusFlags.OVERFLOW
            | ssf.StatusFlags.UNDERFLOW
            | ssf.StatusFlags.INEXACT,
            ssf.StatusFlags(31))
        self.assertEqual(repr(ssf.StatusFlags(0)),
                         "StatusFlags(0)")
        self.assertEqual(repr(ssf.StatusFlags(31)),
                         "StatusFlags.INVALID_OPERATION | "
                         + "StatusFlags.DIVISION_BY_ZERO | "
                         + "StatusFlags.OVERFLOW | "
                         + "StatusFlags.UNDERFLOW | "
                         + "StatusFlags.INEXACT")


class TestDynamicFloat(unittest.TestCase):
    maxDiff = None
    properties = ssf.FloatProperties.standard(32,
                                              ssf.PlatformProperties_RISC_V)

    def test_signatures(self):
        check_signatures(self, ssf.DynamicFloat)

    def test_construct(self):
        obj = ssf.DynamicFloat(properties=self.properties)
        self.assertEqual(obj.properties, self.properties)
        self.assertEqual(obj.bits, 0)
        self.assertEqual(obj.fp_state, ssf.FPState())
        obj = ssf.DynamicFloat(obj, bits=0x1)
        self.assertEqual(obj.properties, self.properties)
        self.assertEqual(obj.bits, 0x1)
        self.assertEqual(obj.fp_state, ssf.FPState())
        obj = ssf.DynamicFloat(properties=self.properties, bits=0x2)
        self.assertEqual(obj.properties, self.properties)
        self.assertEqual(obj.bits, 0x2)
        self.assertEqual(obj.fp_state, ssf.FPState())
        obj = ssf.DynamicFloat(
            properties=self.properties,
            bits=0x3,
            fp_state=ssf.FPState(status_flags=ssf.StatusFlags.INEXACT))
        self.assertEqual(obj.properties, self.properties)
        self.assertEqual(obj.bits, 0x3)
        self.assertEqual(obj.fp_state,
                         ssf.FPState(status_flags=ssf.StatusFlags.INEXACT))

    def test_constants(self):
        cls = ssf.DynamicFloat
        obj = cls.positive_zero(self.properties)
        self.assertEqual(obj.bits, 0x00000000)
        obj = cls.negative_zero(self.properties)
        self.assertEqual(obj.bits, 0x80000000)
        obj = cls.signed_zero(ssf.Sign.Positive, self.properties)
        self.assertEqual(obj.bits, 0x00000000)
        obj = cls.signed_zero(ssf.Sign.Negative, self.properties)
        self.assertEqual(obj.bits, 0x80000000)
        obj = cls.positive_infinity(self.properties)
        self.assertEqual(obj.bits, 0x7F800000)
        obj = cls.negative_infinity(self.properties)
        self.assertEqual(obj.bits, 0xFF800000)
        obj = cls.signed_infinity(ssf.Sign.Positive, self.properties)
        self.assertEqual(obj.bits, 0x7F800000)
        obj = cls.signed_infinity(ssf.Sign.Negative, self.properties)
        self.assertEqual(obj.bits, 0xFF800000)
        obj = cls.quiet_nan(self.properties)
        self.assertEqual(obj.bits, 0x7FC00000)
        obj = cls.signaling_nan(self.properties)
        self.assertEqual(obj.bits, 0x7F800001)
        obj = obj.to_quiet_nan()
        self.assertEqual(obj.bits & 0x7FC00000, 0x7FC00000)
        obj = cls.signed_max_normal(ssf.Sign.Positive, self.properties)
        self.assertEqual(obj.bits, 0x7F7FFFFF)
        obj = cls.signed_max_normal(ssf.Sign.Negative, self.properties)
        self.assertEqual(obj.bits, 0xFF7FFFFF)
        obj = cls.signed_min_subnormal(ssf.Sign.Positive, self.properties)
        self.assertEqual(obj.bits, 0x00000001)
        obj = cls.signed_min_subnormal(ssf.Sign.Negative, self.properties)
        self.assertEqual(obj.bits, 0x80000001)
        self.assertIsNone(getattr(cls, "from_real_algebraic_number", None))

    def handle_binary_op(self, op_name, python_op, bits, status_flags):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = getattr(arg, op_name)(arg, rounding_mode)
        self.assertEqual(obj.bits, bits)
        self.assertEqual(obj.fp_state.status_flags, status_flags)
        if python_op is not None:
            obj = python_op(arg, arg)
            self.assertEqual(obj.bits, bits)
            self.assertEqual(obj.fp_state.status_flags, status_flags)

    def test_add(self):
        self.handle_binary_op("add", operator.add,
                              0x00000000, ssf.StatusFlags(0))

    def test_sub(self):
        self.handle_binary_op("sub", operator.sub,
                              0x00000000, ssf.StatusFlags(0))

    def test_mul(self):
        self.handle_binary_op("mul", operator.mul,
                              0x00000000, ssf.StatusFlags(0))

    def test_div(self):
        self.handle_binary_op("div", operator.truediv,
                              0x7FC00000, ssf.StatusFlags.INVALID_OPERATION)

    def test_ieee754_remainder(self):
        self.handle_binary_op("ieee754_remainder", None,
                              0x7FC00000, ssf.StatusFlags.INVALID_OPERATION)

    def test_fused_mul_add(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.fused_mul_add(arg, arg, rounding_mode)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_round_to_integer(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.round_to_integer(exact=True, rounding_mode=rounding_mode)
        self.assertEqual(obj[0], 0)
        self.assertEqual(obj[1], ssf.FPState())

    def test_round_to_integral(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.round_to_integral(exact=True, rounding_mode=rounding_mode)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_next_up_or_down(self):
        cls = ssf.DynamicFloat
        arg = cls.positive_zero(self.properties)
        obj = arg.next_up_or_down(ssf.UpOrDown.Up)
        self.assertEqual(obj.bits, 0x00000001)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = arg.next_up_or_down(ssf.UpOrDown.Down)
        self.assertEqual(obj.bits, 0x80000001)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = arg.next_up()
        self.assertEqual(obj.bits, 0x00000001)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = arg.next_down()
        self.assertEqual(obj.bits, 0x80000001)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_log_b(self):
        cls = ssf.DynamicFloat
        arg = cls.positive_zero(self.properties)
        obj = arg.log_b()
        self.assertEqual(obj[0], None)
        self.assertEqual(
            obj[1],
            ssf.FPState(status_flags=ssf.StatusFlags.INVALID_OPERATION))

    def test_scale_b(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.scale_b(5, rounding_mode)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_sqrt(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.sqrt(rounding_mode)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_convert_to_dynamic_float(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.convert_to_dynamic_float(rounding_mode, self.properties)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_abs(self):
        cls = ssf.DynamicFloat
        arg = cls.positive_zero(self.properties)
        obj = arg.abs()
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = abs(arg)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_neg(self):
        cls = ssf.DynamicFloat
        arg = cls.positive_zero(self.properties)
        obj = arg.neg()
        self.assertEqual(obj.bits, 0x80000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = -arg
        self.assertEqual(obj.bits, 0x80000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_copy_sign(self):
        cls = ssf.DynamicFloat
        arg = cls.positive_zero(self.properties)
        obj = arg.copy_sign(arg)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_compare(self):
        cls = ssf.DynamicFloat
        zero = cls.positive_zero(self.properties)
        inf = cls.positive_infinity(self.properties)
        nan = cls.quiet_nan(self.properties)
        obj = zero.compare_quiet(zero)
        self.assertEqual(obj[0], 0)
        self.assertEqual(obj[1], ssf.FPState())
        obj = zero.compare_quiet(inf)
        self.assertEqual(obj[0], -1)
        self.assertEqual(obj[1], ssf.FPState())
        obj = inf.compare_quiet(zero)
        self.assertEqual(obj[0], 1)
        self.assertEqual(obj[1], ssf.FPState())
        obj = nan.compare_quiet(nan)
        self.assertIsNone(obj[0])
        self.assertEqual(obj[1], ssf.FPState())
        obj = nan.compare_signaling(nan)
        self.assertIsNone(obj[0])
        self.assertEqual(
            obj[1],
            ssf.FPState(status_flags=ssf.StatusFlags.INVALID_OPERATION))
        obj = zero.compare(zero, quiet=False)
        self.assertEqual(obj[0], 0)
        self.assertEqual(obj[1], ssf.FPState())

    def test_from_int(self):
        cls = ssf.DynamicFloat
        obj = cls.from_int(0, self.properties)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))
        obj = cls.from_int(1, self.properties,
                           rounding_mode=ssf.RoundingMode.TiesToEven,
                           fp_state=ssf.FPState())
        self.assertEqual(obj.bits, 0x3F800000)
        self.assertEqual(obj.fp_state.status_flags, ssf.StatusFlags(0))

    def test_to_int(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.to_int(exact=False, rounding_mode=rounding_mode)
        self.assertEqual(obj[0], 0)
        self.assertEqual(obj[1], ssf.FPState())

    def test_rsqrt(self):
        cls = ssf.DynamicFloat
        rounding_mode = ssf.RoundingMode.TiesToEven
        arg = cls.positive_zero(self.properties)
        obj = arg.rsqrt(rounding_mode)
        self.assertEqual(obj.bits, 0x7F800000)
        self.assertEqual(obj.fp_state.status_flags,
                         ssf.StatusFlags.DIVISION_BY_ZERO)

    def test_attributes(self):
        cls = ssf.DynamicFloat
        obj = cls.positive_zero(self.properties)
        self.assertEqual(obj.bits, 0x00000000)
        self.assertIsInstance(obj.fp_state, ssf.FPState)
        self.assertIsInstance(obj.properties, ssf.FloatProperties)
        self.assertEqual(obj.sign, ssf.Sign.Positive)
        self.assertEqual(obj.exponent_field, 0)
        self.assertEqual(obj.mantissa_field, 0)
        self.assertEqual(obj.mantissa_field_msb, False)
        self.assertEqual(obj.float_class, ssf.FloatClass.PositiveZero)
        self.assertEqual(obj.is_negative_infinity, False)
        self.assertEqual(obj.is_negative_normal, False)
        self.assertEqual(obj.is_negative_subnormal, False)
        self.assertEqual(obj.is_negative_zero, False)
        self.assertEqual(obj.is_positive_infinity, False)
        self.assertEqual(obj.is_positive_normal, False)
        self.assertEqual(obj.is_positive_subnormal, False)
        self.assertEqual(obj.is_positive_zero, True)
        self.assertEqual(obj.is_quiet_nan, False)
        self.assertEqual(obj.is_signaling_nan, False)
        self.assertEqual(obj.is_infinity, False)
        self.assertEqual(obj.is_normal, False)
        self.assertEqual(obj.is_subnormal, False)
        self.assertEqual(obj.is_zero, True)
        self.assertEqual(obj.is_nan, False)
        self.assertEqual(obj.is_finite, True)
        self.assertEqual(obj.is_subnormal_or_zero, True)


if __name__ == '__main__':
    unittest.main()
