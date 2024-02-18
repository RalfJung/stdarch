//! `i686`'s Streaming SIMD Extensions 4a (`SSE4a`)

use crate::core_arch::{simd::*, x86::*};

#[cfg(test)]
use stdarch_test::assert_instr;

#[allow(improper_ctypes)]
extern "C" {
    #[link_name = "llvm.x86.sse4a.extrq"]
    fn extrq(x: i64x2, y: i8x16) -> i64x2;
    #[link_name = "llvm.x86.sse4a.insertq"]
    fn insertq(x: i64x2, y: i64x2) -> i64x2;
    #[link_name = "llvm.x86.sse4a.movnt.sd"]
    fn movntsd(x: *mut f64, y: __m128d);
    #[link_name = "llvm.x86.sse4a.movnt.ss"]
    fn movntss(x: *mut f32, y: __m128);
}

// FIXME(blocked on #248): _mm_extracti_si64(x, len, idx) // EXTRQ
// FIXME(blocked on #248): _mm_inserti_si64(x, y, len, idx) // INSERTQ

/// Extracts the bit range specified by `y` from the lower 64 bits of `x`.
///
/// The `[13:8]` bits of `y` specify the index of the bit-range to extract. The
/// `[5:0]` bits of `y` specify the length of the bit-range to extract. All
/// other bits are ignored.
///
/// If the length is zero, it is interpreted as `64`. If the length and index
/// are zero, the lower 64 bits of `x` are extracted.
///
/// If `length == 0 && index > 0` or `length + index > 64` the result is
/// undefined.
#[inline]
#[target_feature(enable = "sse4a")]
#[cfg_attr(test, assert_instr(extrq))]
#[stable(feature = "simd_x86", since = "1.27.0")]
pub unsafe fn _mm_extract_si64(x: __m128i, y: __m128i) -> __m128i {
    transmute(extrq(x.as_i64x2(), y.as_i8x16()))
}

/// Inserts the `[length:0]` bits of `y` into `x` at `index`.
///
/// The bits of `y`:
///
/// - `[69:64]` specify the `length`,
/// - `[77:72]` specify the index.
///
/// If the `length` is zero it is interpreted as `64`. If `index + length > 64`
/// or `index > 0 && length == 0` the result is undefined.
#[inline]
#[target_feature(enable = "sse4a")]
#[cfg_attr(test, assert_instr(insertq))]
#[stable(feature = "simd_x86", since = "1.27.0")]
pub unsafe fn _mm_insert_si64(x: __m128i, y: __m128i) -> __m128i {
    transmute(insertq(x.as_i64x2(), y.as_i64x2()))
}

/// Non-temporal store of `a.0` into `p`.
///
/// Writes 64-bit data to a memory location without polluting the caches.
///
/// # Safety of non-temporal stores
///
/// After using this intrinsic, but before any other access to the memory that this intrinsic
/// mutates, a call to [`_mm_sfence`] must be performed by the thread that used the intrinsic. In
/// particular, functions that call this intrinsic should generally call `_mm_sfence` before they
/// return.
///
/// See [`_mm_sfence`] for details.
#[inline]
#[target_feature(enable = "sse4a")]
#[cfg_attr(test, assert_instr(movntsd))]
#[stable(feature = "simd_x86", since = "1.27.0")]
pub unsafe fn _mm_stream_sd(p: *mut f64, a: __m128d) {
    movntsd(p, a);
}

/// Non-temporal store of `a.0` into `p`.
///
/// Writes 32-bit data to a memory location without polluting the caches.
///
/// # Safety of non-temporal stores
///
/// After using this intrinsic, but before any other access to the memory that this intrinsic
/// mutates, a call to [`_mm_sfence`] must be performed by the thread that used the intrinsic. In
/// particular, functions that call this intrinsic should generally call `_mm_sfence` before they
/// return.
///
/// See [`_mm_sfence`] for details.
#[inline]
#[target_feature(enable = "sse4a")]
#[cfg_attr(test, assert_instr(movntss))]
#[stable(feature = "simd_x86", since = "1.27.0")]
pub unsafe fn _mm_stream_ss(p: *mut f32, a: __m128) {
    movntss(p, a);
}

#[cfg(test)]
mod tests {
    use crate::core_arch::x86::*;
    use stdarch_test::simd_test;

    #[simd_test(enable = "sse4a")]
    unsafe fn test_mm_extract_si64() {
        let b = 0b0110_0000_0000_i64;
        //        ^^^^ bit range extracted
        let x = _mm_setr_epi64x(b, 0);
        let v = 0b001000___00___000100_i64;
        //        ^idx: 2^3 = 8 ^length = 2^2 = 4
        let y = _mm_setr_epi64x(v, 0);
        let e = _mm_setr_epi64x(0b0110_i64, 0);
        let r = _mm_extract_si64(x, y);
        assert_eq_m128i(r, e);
    }

    #[simd_test(enable = "sse4a")]
    unsafe fn test_mm_insert_si64() {
        let i = 0b0110_i64;
        //        ^^^^ bit range inserted
        let z = 0b1010_1010_1010i64;
        //        ^^^^ bit range replaced
        let e = 0b0110_1010_1010i64;
        //        ^^^^ replaced 1010 with 0110
        let x = _mm_setr_epi64x(z, 0);
        let expected = _mm_setr_epi64x(e, 0);
        let v = 0b001000___00___000100_i64;
        //        ^idx: 2^3 = 8 ^length = 2^2 = 4
        let y = _mm_setr_epi64x(i, v);
        let r = _mm_insert_si64(x, y);
        assert_eq_m128i(r, expected);
    }

    #[repr(align(16))]
    struct MemoryF64 {
        data: [f64; 2],
    }

    #[simd_test(enable = "sse4a")]
    // Miri cannot support this until it is clear how it fits in the Rust memory model
    // (non-temporal store)
    #[cfg_attr(miri, ignore)]
    unsafe fn test_mm_stream_sd() {
        let mut mem = MemoryF64 {
            data: [1.0_f64, 2.0],
        };
        {
            let vals = &mut mem.data;
            let d = vals.as_mut_ptr();

            let x = _mm_setr_pd(3.0, 4.0);

            _mm_stream_sd(d, x);
        }
        assert_eq!(mem.data[0], 3.0);
        assert_eq!(mem.data[1], 2.0);
    }

    #[repr(align(16))]
    struct MemoryF32 {
        data: [f32; 4],
    }

    #[simd_test(enable = "sse4a")]
    // Miri cannot support this until it is clear how it fits in the Rust memory model
    // (non-temporal store)
    #[cfg_attr(miri, ignore)]
    unsafe fn test_mm_stream_ss() {
        let mut mem = MemoryF32 {
            data: [1.0_f32, 2.0, 3.0, 4.0],
        };
        {
            let vals = &mut mem.data;
            let d = vals.as_mut_ptr();

            let x = _mm_setr_ps(5.0, 6.0, 7.0, 8.0);

            _mm_stream_ss(d, x);
        }
        assert_eq!(mem.data[0], 5.0);
        assert_eq!(mem.data[1], 2.0);
        assert_eq!(mem.data[2], 3.0);
        assert_eq!(mem.data[3], 4.0);
    }
}
