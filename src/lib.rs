#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
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
    fn turbo_pfor_roundtrip() {
        let data: Vec<f32> = vec![10.0, 22.0, 23.0, 24.0];
        let length = 1; //data.len();

        // create buffers for compression and decompression!
        let compressed_buffer = vec![0; 1000];
        let compressed = compressed_buffer.as_slice();
        let decompress_buffer = vec![0.0; length];
        let decompressed = decompress_buffer.as_slice();

        // compress data
        let compressed_size = unsafe {
            crate::fpxenc32(
                data.as_ptr() as *mut u32,
                length,
                compressed.as_ptr() as *mut u8,
                0,
            )
        };
        if compressed_size >= compressed.len() {
            panic!("Compress Buffer too small");
        }

        // decompress data
        let decompressed_size = unsafe {
            crate::fpxdec32(
                compressed.as_ptr() as *mut u8,
                length,
                decompressed.as_ptr() as *mut u32,
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
}
