// Copyright 2012 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/*!

Simple compression

*/

#[crate_id = "flate#0.10-pre"];
#[crate_type = "rlib"];
#[crate_type = "dylib"];
#[license = "MIT/ASL2"];
#[allow(missing_doc)];

extern crate extra;
use std::libc::{c_void, size_t, c_int};
use std::libc;
use extra::c_vec::CVec;

pub mod rustrt {
    use std::libc::{c_int, c_void, size_t};

    #[link(name = "miniz", kind = "static")]
    extern {
        pub fn tdefl_compress_mem_to_heap(psrc_buf: *c_void,
                                          src_buf_len: size_t,
                                          pout_len: *mut size_t,
                                          flags: c_int)
                                          -> *mut c_void;

        pub fn tinfl_decompress_mem_to_heap(psrc_buf: *c_void,
                                            src_buf_len: size_t,
                                            pout_len: *mut size_t,
                                            flags: c_int)
                                            -> *mut c_void;
    }
}

static LZ_NORM : c_int = 0x80;  // LZ with 128 probes, "normal"
static TINFL_FLAG_PARSE_ZLIB_HEADER : c_int = 0x1; // parse zlib header and adler32 checksum
static TDEFL_WRITE_ZLIB_HEADER : c_int = 0x01000; // write zlib header and adler32 checksum

fn deflate_bytes_internal(bytes: &[u8], flags: c_int) -> CVec<u8> {
    unsafe {
        let mut outsz : size_t = 0;
        let res = rustrt::tdefl_compress_mem_to_heap(bytes.as_ptr() as *c_void,
                                                     bytes.len() as size_t,
                                                     &mut outsz,
                                                     flags);
        assert!(!res.is_null());
        CVec::new_with_dtor(res as *mut u8, outsz as uint, proc() libc::free(res))
    }
}

pub fn deflate_bytes(bytes: &[u8]) -> CVec<u8> {
    deflate_bytes_internal(bytes, LZ_NORM)
}

pub fn deflate_bytes_zlib(bytes: &[u8]) -> CVec<u8> {
    deflate_bytes_internal(bytes, LZ_NORM | TDEFL_WRITE_ZLIB_HEADER)
}

fn inflate_bytes_internal(bytes: &[u8], flags: c_int) -> CVec<u8> {
    unsafe {
        let mut outsz : size_t = 0;
        let res = rustrt::tinfl_decompress_mem_to_heap(bytes.as_ptr() as *c_void,
                                                       bytes.len() as size_t,
                                                       &mut outsz,
                                                       flags);
        assert!(!res.is_null());
        CVec::new_with_dtor(res as *mut u8, outsz as uint, proc() libc::free(res))
    }
}

pub fn inflate_bytes(bytes: &[u8]) -> CVec<u8> {
    inflate_bytes_internal(bytes, 0)
}

pub fn inflate_bytes_zlib(bytes: &[u8]) -> CVec<u8> {
    inflate_bytes_internal(bytes, TINFL_FLAG_PARSE_ZLIB_HEADER)
}

#[cfg(test)]
mod tests {
    use super::{inflate_bytes, deflate_bytes};
    use std::rand;
    use std::rand::Rng;

    #[test]
    fn test_flate_round_trip() {
        let mut r = rand::rng();
        let mut words = ~[];
        for _ in range(0, 20) {
            let range = r.gen_range(1u, 10);
            words.push(r.gen_vec::<u8>(range));
        }
        for _ in range(0, 20) {
            let mut input = ~[];
            for _ in range(0, 2000) {
                input.push_all(r.choose(words));
            }
            debug!("de/inflate of {} bytes of random word-sequences",
                   input.len());
            let cmp = deflate_bytes(input);
            let out = inflate_bytes(cmp.as_slice());
            debug!("{} bytes deflated to {} ({:.1f}% size)",
                   input.len(), cmp.len(),
                   100.0 * ((cmp.len() as f64) / (input.len() as f64)));
            assert_eq!(input.as_slice(), out.as_slice());
        }
    }

    #[test]
    fn test_zlib_flate() {
        let bytes = ~[1, 2, 3, 4, 5];
        let deflated = deflate_bytes(bytes);
        let inflated = inflate_bytes(deflated.as_slice());
        assert_eq!(inflated.as_slice(), bytes.as_slice());
    }
}