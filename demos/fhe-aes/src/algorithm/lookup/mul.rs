use tfhe::core_crypto::fpga::lookup_vector::LookupVector;

use crate::algorithm::constants::{
    MUL2_LUT_HI, MUL2_LUT_LO, MUL9_LUT_HI, MUL9_LUT_LO, MUL11_LUT_HI, MUL11_LUT_LO, MUL13_LUT_HI,
    MUL13_LUT_LO, MUL14_LUT_HI, MUL14_LUT_LO,
};

use super::{FheAesLookup, lookup_match};

impl FheAesLookup<'_> {
    pub fn mul2_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| lookup_match(MUL2_LUT_LO, x) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_LO, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_LO, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_LO, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| lookup_match(MUL2_LUT_HI, x) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_HI, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_HI, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul2_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let func = |x| (lookup_match(MUL2_LUT_HI, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&func)
    }

    pub fn mul9_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL9_LUT_LO, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_LO, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_LO, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_LO, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL9_LUT_HI, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_HI, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_HI, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul9_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL9_LUT_HI, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL11_LUT_LO, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_LO, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_LO, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_LO, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL11_LUT_HI, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_HI, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_HI, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul11_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL11_LUT_HI, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL13_LUT_LO, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_LO, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_LO, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_LO, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL13_LUT_HI, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_HI, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_HI, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul13_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL13_LUT_HI, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL14_LUT_LO, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_LO, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_LO, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_LO, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(MUL14_LUT_HI, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_HI, x) >> 2) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_HI, x) >> 4) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn mul14_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(MUL14_LUT_HI, x) >> 6) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }
}
