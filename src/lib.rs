#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use std::ffi::c_void;

    use crate::{om_encoder_init, OmError_t_ERROR_OK};

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

    #[test]
    fn test_p4nzenc128v16_cpuarch_dependency() {
        let u8_nums = vec![02_u8, 0, 3, 0, 5, 0, 5, 0];
        let u16_len = u8_nums.len() / 2;

        let mut u16_nums = unsafe {
            // Create a new Vec<u16> with the same underlying memory
            Vec::from_raw_parts(
                u8_nums.as_ptr() as *mut u16,
                u16_len,                // len in u16s is half of u8s
                u8_nums.capacity() / 2, // capacity in u16s is half of u8s
            )
        };
        // Make sure the original vec doesn't drop the memory
        std::mem::forget(u8_nums);

        let compressed = vec![0_u8; 1000];
        let mut direct_compressed = compressed.clone();
        let mut callback_compressed = compressed.clone();

        // Test both direct function and function pointer callbacks
        let direct_result = unsafe {
            crate::p4nzenc128v16(
                u16_nums.as_mut_ptr(),
                u16_len,
                direct_compressed.as_mut_ptr(),
            )
        };

        let callback_result = unsafe {
            // Cast through raw function pointers first
            let fn_ptr = crate::p4nzenc128v16 as *const ();
            let callback: crate::om_compress_callback_t = Some(std::mem::transmute(fn_ptr));

            callback.unwrap()(
                u16_nums.as_ptr() as *const c_void,
                u16_len as u64,
                callback_compressed.as_mut_ptr() as *mut c_void,
            )
        };

        assert_eq!(direct_result, callback_result as usize);
        assert_eq!(direct_result, 4);
        assert_eq!(&direct_compressed[0..4], &callback_compressed[0..4]);
        #[cfg(target_arch = "x86_64")]
        assert_eq!(&direct_compressed[0..4], &[2, 3, 34, 0]);
        #[cfg(target_arch = "aarch64")]
        assert_eq!(&direct_compressed[0..4], &[2, 3, 34, 0]);
        // assert_eq!(&direct_compressed[0..4], &[2, 3, 34, 16]); // WHY??
    }

    #[test]
    fn test_compress_empty_data_chunk() {
        let dimensions = vec![1000, 1000];
        let chunks = vec![10, 10];
        let lut_chunk_element_count = 256;

        let data = vec![0.0; 1000];

        let array_offset = vec![0; 2];
        let array_count = vec![1000; 2];
        let chunk_index = 0;
        let chunk_offset = 0;

        println!("create encoder");

        let mut encoder = crate::OmEncoder_t {
            dimension_count: 0,
            lut_chunk_element_count: 0,
            dimensions: std::ptr::null_mut(),
            chunks: std::ptr::null_mut(),
            compress_callback: None,
            compress_filter_callback: None,
            compress_copy_callback: None,
            scale_factor: 0.0,
            add_offset: 0.0,
            bytes_per_element: 0,
            bytes_per_element_compressed: 0,
        };

        println!("init encoder");

        let error = unsafe {
            om_encoder_init(
                &mut encoder,
                1.0,
                0.0,
                1,  // p4nzdec256
                20, // float array
                dimensions.as_ptr(),
                chunks.as_ptr(),
                dimensions.len() as u64,
                lut_chunk_element_count,
            )
        };

        println!("check error");

        assert!(error == OmError_t_ERROR_OK, "Initialized with error");

        let mut compressed = vec![0u8; 1000];
        let mut chunk_buffer = vec![0u8; 1000];

        println!("compress chunk");

        let bytes_written = unsafe {
            crate::om_encoder_compress_chunk(
                &mut encoder,
                data.as_slice().as_ptr() as *const u8,
                dimensions.as_ptr(),
                array_offset.as_ptr(),
                array_count.as_ptr(),
                chunk_index,
                chunk_offset,
                compressed.as_mut_ptr(),
                chunk_buffer.as_mut_ptr(),
            )
        };

        println!("check bytes written");

        // differences on different operating systems???
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        assert_eq!(bytes_written, 11);
        #[cfg(target_os = "macos")]
        assert_eq!(bytes_written, 2);
        println!("Basically finished test...")
    }
}
