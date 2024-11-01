#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// re-export the symbols for the C API because rust will loose them for wasm
// because the crate type needs to be cdylib
// https://github.com/rust-lang/rfcs/issues/2771#issuecomment-363695407
#[doc(hidden)]
#[macro_export]
macro_rules! export_c_symbol {
    (fn $name:ident($( $arg:ident : $type:ty ),*) -> $ret:ty) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [< _$name >]($( $arg : $type),*) -> $ret {
                $crate::$name($( $arg ),*)
            }
        }
    };
    (fn $name:ident($( $arg:ident : $type:ty ),*)) => {
        export_c_symbol!(fn $name($( $arg : $type),*) -> ());
    }
}

/// As a workaround for rust-lang/rust#6342, you can use this macro to make sure
/// the symbols for `ffi_utils`'s error handling are correctly exported in your
/// `cdylib`.
#[macro_export]
macro_rules! export_om_fileformatc_symbols {
    () => {
    export_c_symbol!(fn fpxenc32(in_: *mut u32, n: usize, out: *mut ::std::os::raw::c_uchar, start: u32) -> usize);
    // export_c_symbol!(fn last_error_length() -> ::libc::c_int);
    // export_c_symbol!(fn last_error_length_utf16() -> ::libc::c_int);
    // export_c_symbol!(fn error_message_utf8(buf: *mut ::libc::c_char, length: ::libc::c_int) -> ::libc::c_int);
    // export_c_symbol!(fn error_message_utf16(buf: *mut u16, length: ::libc::c_int) -> ::libc::c_int);
    };
}

#[cfg(test)]
mod tests {

    export_om_fileformatc_symbols!();

    #[test]
    fn test_round_trip_p4n() {
        const n: usize = 3;
        let mut nums = vec![33_u16, 44, 77];
        let mut compressed = vec![0_u8; 1000];
        // TODO: p4bound buffer sizes!
        let mut recovered = vec![0_u16; n + 200];
        unsafe {
            crate::p4nzenc128v16(nums.as_mut_ptr(), 3, compressed.as_mut_ptr());
            crate::p4nzdec128v16(compressed.as_mut_ptr(), n, recovered.as_mut_ptr());
        }
        assert_eq!(recovered[..n], nums[..n]);
    }

    #[test]
    fn test_round_trip_fp32() {
        const n: usize = 3;
        let mut nums = vec![33_u32, 44, 77];
        let mut compressed = vec![0_u8; 1000];
        let mut recovered = vec![0_u32; n];
        unsafe {
            let compressed_size = crate::fpxenc32(nums.as_mut_ptr(), 3, compressed.as_mut_ptr(), 0);
            let decompressed_size =
                crate::fpxdec32(compressed.as_mut_ptr(), n, recovered.as_mut_ptr(), 0);
            assert_eq!(compressed_size, decompressed_size);
        }
        assert_eq!(recovered, nums);
    }

    #[test]
    fn test_round_trip_fp32_with_very_short_length() {
        let data: Vec<f32> = vec![10.0, 22.0, 23.0, 24.0];
        let length = 1; //data.len();

        // create buffers for compression and decompression!
        let mut compressed = vec![0; 1000];
        let mut decompressed = vec![0.0; length];

        // compress data
        let compressed_size = unsafe {
            crate::fpxenc32(
                data.as_ptr() as *mut u32,
                length,
                compressed.as_mut_ptr(),
                0,
            )
        };
        if compressed_size >= compressed.len() {
            panic!("Compress Buffer too small");
        }

        // decompress data
        let decompressed_size = unsafe {
            crate::fpxdec32(
                compressed.as_mut_ptr(),
                length,
                decompressed.as_mut_ptr() as *mut u32,
                0,
            )
        };

        // this should be equal (we check it in the reader)
        // here we have a problem if length is only 1 and the exponent of the
        // float is greater than 0 (e.g. the value is greater than 10)
        // NOTE: This fails with 4 != 5 in the original turbo-pfor library
        assert_eq!(decompressed_size, compressed_size);
        assert_eq!(data[..length], decompressed[..length]);
    }

    #[test]
    fn test_delta2d_decode() {
        let mut buffer: Vec<i16> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        unsafe { crate::delta2d_decode(2, 5, buffer.as_mut_ptr()) };
        assert_eq!(buffer, vec![1, 2, 3, 4, 5, 7, 9, 11, 13, 15]);
    }

    #[test]
    fn test_delta2d_encode() {
        let mut buffer: Vec<i16> = vec![1, 2, 3, 4, 5, 7, 9, 11, 13, 15];
        unsafe { crate::delta2d_encode(2, 5, buffer.as_mut_ptr()) };
        assert_eq!(buffer, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_delta2d_decode_xor() {
        let mut buffer: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        unsafe { crate::delta2d_decode_xor(2, 5, buffer.as_mut_ptr()) };
        let expected: Vec<f32> = vec![
            1.0,
            2.0,
            3.0,
            4.0,
            5.0,
            2.5521178e38,
            2.0571151e-38,
            3.526483e-38,
            5.2897246e-38,
            4.7019774e-38,
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_delta2d_encode_xor() {
        let mut buffer: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 7.0, 5.0, 11.0, 12.0, 15.0];
        unsafe { crate::delta2d_encode_xor(2, 5, buffer.as_mut_ptr()) };
        let expected: Vec<f32> = vec![
            1.0,
            2.0,
            3.0,
            4.0,
            5.0,
            2.9774707e38,
            1.469368e-38,
            4.4081038e-38,
            7.052966e-38,
            7.6407133e-38,
        ];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_delta2d_xor_roundtrip() {
        let mut buffer: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        unsafe {
            crate::delta2d_decode_xor(2, 5, buffer.as_mut_ptr());
            crate::delta2d_encode_xor(2, 5, buffer.as_mut_ptr());
        }
        let expected: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        assert_eq!(buffer, expected);
    }

    #[test]
    fn test_delta2d_roundtrip() {
        let mut buffer: Vec<i16> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        unsafe {
            crate::delta2d_decode(2, 5, buffer.as_mut_ptr());
            crate::delta2d_encode(2, 5, buffer.as_mut_ptr());
        }
        let expected: Vec<i16> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        assert_eq!(buffer, expected);
    }
}
