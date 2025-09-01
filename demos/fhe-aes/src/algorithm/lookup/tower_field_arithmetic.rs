use super::{FheAesLookup, lookup_match};
use crate::algorithm::constants::{
    DELTA_INV_2, FWD_HI, FWD_HI_INV, FWD_LO, FWD_LO_INV, FWD_LUT, GF2_MUL_LUT, GF2_MUL_PHI_LUT,
    INV_HI, INV_HI_AFF, INV_LO, INV_LO_AFF, INV_LUT, LAMBDA_SQ_2, XOR_FWD_LUT,
};
use tfhe::core_crypto::fpga::lookup_vector::LookupVector;

impl FheAesLookup<'_> {
    pub fn fwd_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_LO, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_HI, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_lo_lo_lo_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_LO_INV, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_lo_lo_hi_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO_INV, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_lo_hi_lo_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO_INV, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_lo_hi_hi_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_LO_INV, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_hi_lo_lo_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_HI_INV, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_lo_hi_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI_INV, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_hi_lo_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI_INV, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn fwd_hi_hi_hi_with_inv_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(FWD_HI_INV, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_lo_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_LO, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_hi_lo_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_HI, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_lo_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_hi_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_hi_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_lo_lo_lo_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_LO_AFF, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_lo_hi_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO_AFF, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_lo_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO_AFF, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_hi_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO_AFF, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_hi_lo_lo_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_HI_AFF, x) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_lo_hi_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI_AFF, x) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_hi_lo_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI_AFF, x) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_hi_hi_hi_with_affine(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_HI_AFF, x) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_lo_lo_lo_with_affine_and_xor(&self, rc: u64) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| (lookup_match(INV_LO_AFF, x) ^ rc) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_lo_hi_with_affine_and_xor(&self, rc: u64) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| ((lookup_match(INV_LO_AFF, x) ^ rc) >> 2) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_lo_with_affine_and_xor(&self, rc: u64) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| ((lookup_match(INV_LO_AFF, x) ^ rc) >> 4) & 3;
        shortint_key.generate_lookup_vector(&f)
    }
    pub fn inv_lo_hi_hi_with_affine_and_xor(&self, rc: u64) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| ((lookup_match(INV_LO_AFF, x) ^ rc) >> 6) & 3;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn bitxor_with_fwd_transf(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(XOR_FWD_LUT, x);
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_LUT, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn fwd_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(FWD_LUT, x) >> 2;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_lo(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_LUT, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn inv_hi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(INV_LUT, x) >> 2;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn gf2_mul(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x: u64, y: u64| {
            let idx = (x << 2) | y;
            lookup_match(GF2_MUL_LUT, idx)
        };
        shortint_key.generate_lookup_vector_bivariate(&f)
    }

    pub fn gf2_mul_phi(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x: u64, y: u64| {
            let idx = (x << 2) | y;
            lookup_match(GF2_MUL_PHI_LUT, idx)
        };
        shortint_key.generate_lookup_vector_bivariate(&f)
    }

    pub fn lambda_sq_lo2(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(LAMBDA_SQ_2, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn lambda_sq_hi2(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(LAMBDA_SQ_2, x) >> 2;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn delta_inv_lo2(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(DELTA_INV_2, x) & 0b11;
        shortint_key.generate_lookup_vector(&f)
    }

    pub fn delta_inv_hi2(&self) -> LookupVector {
        let shortint_key = &self.fpga_key.key.key.key;
        let f = |x| lookup_match(DELTA_INV_2, x) >> 2;
        shortint_key.generate_lookup_vector(&f)
    }
}
