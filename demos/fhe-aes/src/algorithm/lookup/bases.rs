use tfhe::core_crypto::fpga::lookup_vector::LookupVector;

use super::FheAesLookup;

impl FheAesLookup<'_> {
    pub fn mul2_base(&self) -> Vec<LookupVector> {
        vec![
            self.mul2_lo_hi_hi(),
            self.mul2_lo_hi_lo(),
            self.mul2_lo_lo_hi(),
            self.mul2_lo_lo_lo(),
            self.mul2_hi_hi_hi(),
            self.mul2_hi_hi_lo(),
            self.mul2_hi_lo_hi(),
            self.mul2_hi_lo_lo(),
        ]
    }

    pub fn mul9_base(&self) -> Vec<LookupVector> {
        vec![
            self.mul9_lo_lo_lo(),
            self.mul9_lo_lo_hi(),
            self.mul9_lo_hi_lo(),
            self.mul9_lo_hi_hi(),
            self.mul9_hi_lo_lo(),
            self.mul9_hi_lo_hi(),
            self.mul9_hi_hi_lo(),
            self.mul9_hi_hi_hi(),
        ]
    }

    pub fn mul11_base(&self) -> Vec<LookupVector> {
        vec![
            self.mul11_lo_lo_lo(),
            self.mul11_lo_lo_hi(),
            self.mul11_lo_hi_lo(),
            self.mul11_lo_hi_hi(),
            self.mul11_hi_lo_lo(),
            self.mul11_hi_lo_hi(),
            self.mul11_hi_hi_lo(),
            self.mul11_hi_hi_hi(),
        ]
    }

    pub fn mul13_base(&self) -> Vec<LookupVector> {
        vec![
            self.mul13_lo_lo_lo(),
            self.mul13_lo_lo_hi(),
            self.mul13_lo_hi_lo(),
            self.mul13_lo_hi_hi(),
            self.mul13_hi_lo_lo(),
            self.mul13_hi_lo_hi(),
            self.mul13_hi_hi_lo(),
            self.mul13_hi_hi_hi(),
        ]
    }

    pub fn mul14_base(&self) -> Vec<LookupVector> {
        vec![
            self.mul14_lo_lo_lo(),
            self.mul14_lo_lo_hi(),
            self.mul14_lo_hi_lo(),
            self.mul14_lo_hi_hi(),
            self.mul14_hi_lo_lo(),
            self.mul14_hi_lo_hi(),
            self.mul14_hi_hi_lo(),
            self.mul14_hi_hi_hi(),
        ]
    }

    pub fn fwd_base(&self) -> Vec<LookupVector> {
        vec![
            self.fwd_lo_lo_lo(),
            self.fwd_lo_lo_hi(),
            self.fwd_lo_hi_lo(),
            self.fwd_lo_hi_hi(),
            self.fwd_hi_lo_lo(),
            self.fwd_hi_lo_hi(),
            self.fwd_hi_hi_lo(),
            self.fwd_hi_hi_hi(),
        ]
    }

    pub fn inv_base(&self) -> Vec<LookupVector> {
        [
            self.inv_lo_lo_lo(),
            self.inv_lo_lo_hi(),
            self.inv_lo_hi_lo(),
            self.inv_lo_hi_hi(),
            self.inv_hi_lo_lo(),
            self.inv_hi_lo_hi(),
            self.inv_hi_hi_lo(),
            self.inv_hi_hi_hi(),
        ]
        .to_vec()
    }

    pub fn inv_with_affine_base(&self) -> Vec<LookupVector> {
        vec![
            self.inv_lo_lo_lo_with_affine(),
            self.inv_lo_lo_hi_with_affine(),
            self.inv_lo_hi_lo_with_affine(),
            self.inv_lo_hi_hi_with_affine(),
            self.inv_hi_lo_lo_with_affine(),
            self.inv_hi_lo_hi_with_affine(),
            self.inv_hi_hi_lo_with_affine(),
            self.inv_hi_hi_hi_with_affine(),
        ]
    }

    pub fn inv_with_affine_and_xor_base(&self, round_constant: u64) -> Vec<LookupVector> {
        vec![
            self.inv_lo_lo_lo_with_affine_and_xor(round_constant),
            self.inv_lo_lo_hi_with_affine_and_xor(round_constant),
            self.inv_lo_hi_lo_with_affine_and_xor(round_constant),
            self.inv_lo_hi_hi_with_affine_and_xor(round_constant),
            self.inv_hi_lo_lo_with_affine(),
            self.inv_hi_lo_hi_with_affine(),
            self.inv_hi_hi_lo_with_affine(),
            self.inv_hi_hi_hi_with_affine(),
        ]
    }

    pub fn fwd_with_inv_affine_base(&self) -> Vec<LookupVector> {
        vec![
            self.fwd_lo_lo_lo_with_inv_affine(),
            self.fwd_lo_lo_hi_with_inv_affine(),
            self.fwd_lo_hi_lo_with_inv_affine(),
            self.fwd_lo_hi_hi_with_inv_affine(),
            self.fwd_hi_lo_lo_with_inv_affine(),
            self.fwd_hi_lo_hi_with_inv_affine(),
            self.fwd_hi_hi_lo_with_inv_affine(),
            self.fwd_hi_hi_hi_with_inv_affine(),
        ]
    }
}
