use std::slice;

pub fn decompress(bs: &[u8], size: usize) -> Vec<u8> {
    let blen = bs.len();
    let mut out: Vec<u8> = Vec::with_capacity(blen);
    let mut pos = 0;
    while pos < blen && out.len() < size {
        let control = bs[pos];
        pos += 1;
        for i in 0..8 {
            match control & (1 << i) == 0 {
                true => {
                    out.push(bs[pos]);
                    pos += 1;
                }
                false => {
                    let length = ((bs[pos] >> 2) + 3) as usize;
                    let distance = ((bs[pos] & 0b11) as usize) << 8 | bs[pos+1] as usize;
                    pos += 2;
                    let backref = (out[out.len()-distance..]).to_vec();
                    for _ in 0..(length / distance) {
                        out.extend_from_slice(backref.as_slice());
                    }
                    out.extend_from_slice(&backref.as_slice()[..(length % distance)])
                }
            }
        }
    }
    out.truncate(size);
    out
}

#[repr(C)]
pub struct DecompressionResult {
    size: usize,
    data: *const u8,
}

#[no_mangle]
pub extern fn decompress_external(bs_ptr: *const u8, bs_size: usize, size: usize) -> DecompressionResult {
    let bs;
    unsafe {
        bs = slice::from_raw_parts(bs_ptr, bs_size);
    }
    let out = decompress(bs, size);
    DecompressionResult{size: out.len(), data: out.as_ptr()}
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
        let data = decompress(compressed.as_slice(), decompressed.len());
        assert_eq!(data, decompressed);
        Ok(())
    }
}
