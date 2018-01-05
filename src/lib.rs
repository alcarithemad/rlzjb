use std::mem;
use std::slice;
use std::vec;

pub fn decompress(bs: &[u8], size: usize) -> Option<Vec<u8>> {
    let blen = bs.len();
    let mut out: Vec<u8> = Vec::with_capacity(blen);
    let mut pos = 0;
    while pos < blen && out.len() < size {
        let control = bs.get(pos)?;
        pos += 1;
        for i in 0..8 {
            if ! (pos < blen) {
                break
            }
            if control & (1 << i) == 0 {
                out.push(*bs.get(pos)?);
                pos += 1;
            } else {
                let length = ((bs.get(pos)? >> 2) + 3 as u8) as usize;
                let distance = ((bs.get(pos)? & 0b11) as usize) << 8 | *bs.get(pos+1)? as usize;
                pos += 2;
                let pivot = out.len();
                out.resize(pivot+length, 0);

                let (first, end) = out.split_at_mut(pivot);

                let backref = &first[first.len()-distance..];

                for i in 0..(length / distance) {
                    end[(i*distance)..((i+1)*distance)].copy_from_slice(backref);
                }

                let final_stride = length % distance;
                let endl = end.len();
                end[endl-final_stride..].copy_from_slice(&backref[..final_stride]);
            }
        }
    }
    out.truncate(size);
    Some(out)
}

#[repr(C)]
pub struct DecompressionResult {
    success: bool,
    size: usize,
    data: *const u8,
    capacity: usize,
}

#[no_mangle]
pub extern fn decompress_external(bs_ptr: *const u8, bs_size: usize, size: usize) -> DecompressionResult {
    let bs;
    unsafe {
        if bs_ptr.is_null() {
            return DecompressionResult{success: false, size: 0, data: bs_ptr, capacity: 0 }
        }
        bs = slice::from_raw_parts(bs_ptr, bs_size);
    }
    let out = decompress(bs, size);
    let ret = match out {
        Some(ref v) => DecompressionResult{success: true, size: v.len(), data: v.as_ptr(), capacity: v.capacity() },
        None => DecompressionResult{success: false, size: 0, data: bs_ptr, capacity: 0 }
    };
    mem::forget(out);
    ret
}

#[no_mangle]
pub extern fn free_result(result: DecompressionResult) {
    unsafe {
        vec::Vec::from_raw_parts(result.data as (*mut u8), result.size, result.capacity)
    };
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io;
    use std::io::prelude::*;
    use super::decompress;

    #[test]
    fn decompress_test() {
        decompress_file("001-in", "001-out").unwrap();
        decompress_file("107-in", "107-out").unwrap();
    }

    fn decompress_file(name_in: &str, name_out: &str) -> io::Result<()> {
        let mut infile = fs::File::open(name_in)?;
        let mut compressed: Vec<u8> = Vec::new();
        infile.read_to_end(&mut compressed)?;
        let mut outfile = fs::File::open(name_out)?;
        let mut decompressed: Vec<u8> = Vec::new();
        outfile.read_to_end(&mut decompressed)?;
        let data = decompress(compressed.as_slice(), decompressed.len()).unwrap();
        assert_eq!(data, decompressed);
        Ok(())
    }
}
