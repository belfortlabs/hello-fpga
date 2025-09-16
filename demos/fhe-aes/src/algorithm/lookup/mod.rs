use tfhe::BelfortServerKey;

use tfhe::core_crypto::fpga::lookup_vector::LookupVector;

pub mod bases;
pub mod mul;
pub mod tower_field_arithmetic;

pub fn lookup_match(table: [u64; 16], x: u64) -> u64 {
    match x {
        0x00 => table[0],
        0x01 => table[1],
        0x02 => table[2],
        0x03 => table[3],
        0x04 => table[4],
        0x05 => table[5],
        0x06 => table[6],
        0x07 => table[7],
        0x08 => table[8],
        0x09 => table[9],
        0x0A => table[10],
        0x0B => table[11],
        0x0C => table[12],
        0x0D => table[13],
        0x0E => table[14],
        0x0F => table[15],
        _ => panic!("{x} is outside the legal 4-bit input range"),
    }
}

pub struct FheAesLookup<'a> {
    pub fpga_key: &'a BelfortServerKey,
}

impl<'a> FheAesLookup<'a> {
    pub fn new(fpga_key: &'a BelfortServerKey) -> Self {
        Self { fpga_key }
    }

    pub fn lut_bitxor(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x, y| x ^ y;
        shortint_key.generate_lookup_vector_bivariate(&func)
    }
}
