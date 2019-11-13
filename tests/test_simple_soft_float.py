# SPDX-License-Identifier: LGPL-2.1-or-later
# See Notices.txt for copyright information

import simple_soft_float as ssf
import unittest


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
                         {cls.DefaultIgnoreExactUnderflow,
                          cls.DefaultSignalExactUnderflow})


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

    def test_smoke_test(self):
        rounding_mode = ssf.RoundingMode.TiesToEven
        status_flags = ssf.StatusFlags(0)
        exception_handling_mode = ssf.ExceptionHandlingMode \
            .DefaultIgnoreExactUnderflow
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
            + "ExceptionHandlingMode.DefaultIgnoreExactUnderflow, "
            + "tininess_detection_mode=TininessDetectionMode.AfterRounding)")


class TestFloatProperties(unittest.TestCase):
    maxDiff = None

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
        self.assertEqual(
            repr(obj),
            "FloatProperties.standard(32, "
            + "PlatformProperties(canonical_nan_sign=Sign.Positive, "
            + "canonical_nan_mantissa_msb=True, "
            + "canonical_nan_mantissa_second_to_msb=False, "
            + "canonical_nan_mantissa_rest=False, "
            + "std_bin_ops_nan_propagation_mode="
            + "BinaryNaNPropagationMode.AlwaysCanonical, "
            + "fma_nan_propagation_mode="
            + "TernaryNaNPropagationMode.AlwaysCanonical, "
            + "fma_inf_zero_qnan_result="
            + "FMAInfZeroQNaNResult.CanonicalAndGenerateInvalid, "
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
            + "UnaryNaNPropagationMode.AlwaysCanonical))")


class TestPlatformProperties(unittest.TestCase):
    maxDiff = None

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

    def test_construct(self):
        # FIXME: finish
        pass


if __name__ == '__main__':
    unittest.main()
