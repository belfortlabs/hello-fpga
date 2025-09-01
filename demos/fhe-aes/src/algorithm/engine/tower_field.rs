use tfhe::shortint::Ciphertext;

use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::integer::fpga::BelfortServerKey;

use crate::algorithm::{
    lookup::FheAesLookup,
    utils::{
        FheAesCiphertextUtils, interleave_chunks_of_two, interleave_low_high, repeat_each,
        repeat_lut, repeat_luts_cycled, split_by_chunk_sizes, split_chunks_even_odd,
    },
};

pub struct FheAesTowerField<'a> {
    fpga_key: &'a BelfortServerKey,
    lookup: FheAesLookup<'a>,
}

impl<'a> FheAesTowerField<'a> {
    pub fn new(fpga_key: &'a BelfortServerKey) -> Self {
        let lookup = FheAesLookup::new(fpga_key);
        Self { fpga_key, lookup }
    }

    /// Applies the algebraic isomorphism transformation required for the tower
    /// field decomposition.
    ///
    /// # Arguments
    /// - `inputs` – Vector of ciphertexts representing input bits.
    /// - `base` – Vector of lookup tables that define the algebraic isomorphism.
    ///
    /// # Note
    /// Modifies the state in place.
    pub fn apply_isomorphism(&self, inputs: &mut Vec<Ciphertext>, base: &[LookupVector]) {
        let lookup = repeat_luts_cycled(base, inputs.len() / base.len());

        self.fpga_key
            .apply_lookup_vector_packed_assign(inputs, &lookup);

        let (even_chunks, odd_chunks) = split_chunks_even_odd(inputs, 4);

        *inputs = self.fpga_key.pack_slices(&even_chunks, &odd_chunks);

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(inputs, self.lookup.lut_bitxor());
    }

    /// Multiplies one operand pair (x, y) in GF(2⁴).
    ///
    /// # Arguments
    /// - `pair`: A two-element `Vec<Vec<Ciphertext>>` containing the operands.
    ///
    /// # Returns
    /// The product `x * y`, as a `Vec<Ciphertext>` of 32 blocks (2 blocks per byte).
    pub fn gf2_4_mul_single(&self, pair: &[Vec<Ciphertext>]) -> Vec<Ciphertext> {
        let (x, y) = (&pair[0], &pair[1]);
        let (x_before_transf, y_before_transf) = (
            self.fpga_key.split_and_pack(x, 1),
            self.fpga_key.split_and_pack(y, 1),
        );

        let x_and_y_blocks_len = x_before_transf.len() + y_before_transf.len();
        // Perform transformations on x and y individually, and also perform transformations
        // on x and y followed by computing their bitwise XOR in GF(2^2) as a batch, using
        // separate lookup tables for each.
        let mut xor_and_fwd_stage = [
            &x_before_transf[..],
            &y_before_transf[..],
            &repeat_each(&x_before_transf, 2)[..],
            &repeat_each(&y_before_transf, 2)[..],
        ]
        .concat();

        let xor_luts = [
            &repeat_lut(self.lookup.bitxor_with_fwd_transf(), x_and_y_blocks_len)[..],
            &repeat_luts_cycled(
                &[self.lookup.fwd_lo(), self.lookup.fwd_hi()],
                x_and_y_blocks_len,
            )[..],
        ]
        .concat();

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut xor_and_fwd_stage, &xor_luts);

        let parts = split_by_chunk_sizes(
            &xor_and_fwd_stage,
            &[
                x_before_transf.len(),
                y_before_transf.len(),
                2 * x_before_transf.len(),
                2 * y_before_transf.len(),
            ],
        );
        let [x0_xor_x1, y0_xor_y1, x0_and_x1, y0_and_y1] = <[_; 4]>::try_from(parts).unwrap();

        // Compute the required GF(2^2) multiplications.
        let (x0, x1) = split_chunks_even_odd(&x0_and_x1, 1);
        let (y0, y1) = split_chunks_even_odd(&y0_and_y1, 1);

        let mut mul_stage = [
            &self.fpga_key.pack_slices(&x0_xor_x1, &y0_xor_y1)[..],
            &self.fpga_key.pack_slices(&x0, &y0)[..],
            &self.fpga_key.pack_slices(&x1, &y1)[..],
        ]
        .concat();

        let mul_luts = [
            &repeat_lut(self.lookup.gf2_mul(), x0_xor_x1.len() + x0.len())[..],
            &repeat_lut(self.lookup.gf2_mul_phi(), x1.len())[..],
        ]
        .concat();

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut mul_stage, &mul_luts);

        let parts = split_by_chunk_sizes(&mul_stage, &[x0_xor_x1.len(), x0.len(), x1.len()]);
        let [xy_xor_mul, x0y0_mul, x1y1_phi_mul] = <[_; 3]>::try_from(parts).unwrap();

        // XOR the required products.
        let hi2 = self.fpga_key.pack_slices(&xy_xor_mul, &x0y0_mul);
        let lo2 = self.fpga_key.pack_slices(&x1y1_phi_mul, &x0y0_mul);
        let mut in_blocks = interleave_low_high(&lo2, &hi2);

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut in_blocks, self.lookup.lut_bitxor());

        // Transform the results back into GF(2^4).
        let (lower, upper) = split_chunks_even_odd(&in_blocks, 1);
        let packed = self.fpga_key.pack_slices(&lower, &upper);
        let mut duplicated = repeat_each(&packed, 2);

        let inv_luts =
            repeat_luts_cycled(&[self.lookup.inv_lo(), self.lookup.inv_hi()], lower.len());

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut duplicated, &inv_luts);

        duplicated
    }

    /// Multiplies two independent operand pairs (s, t) and (p, r) in GF(2^4).
    ///
    /// # Arguments
    /// - `first` – A two-element `Vec<Vec<Ciphertext>>` containing the first operand pair
    /// - `second` – A two-element `Vec<Vec<Ciphertext>>` containing the second operand pair
    ///
    /// # Returns
    /// `(s * t) || (p * r)`, as a `Vec<Ciphertext>` of 64 blocks (2 blocks per byte and
    /// multiplication).
    pub fn gf2_4_mul_double(
        &self,
        first: &[Vec<Ciphertext>],
        second: &[Vec<Ciphertext>],
    ) -> Vec<Ciphertext> {
        let (s, t) = (&first[0], &first[1]);
        let (p, r) = (&second[0], &second[1]);

        let (s_before_transf, t_before_transf, p_before_transf, r_before_transf) = (
            self.fpga_key.split_and_pack(s, 1),
            self.fpga_key.split_and_pack(t, 1),
            self.fpga_key.split_and_pack(p, 1),
            self.fpga_key.split_and_pack(r, 1),
        );

        // Perform transformations on each pair individually, and also perform transformations
        // on the pairs followed by computing their bitwise XOR in GF(2^2) as a batch, using
        // separate lookup tables for each.
        let mut xor_and_fwd_stage = [
            &s_before_transf[..],
            &t_before_transf[..],
            &p_before_transf[..],
            &r_before_transf[..],
            &repeat_each(&s_before_transf, 2)[..],
            &repeat_each(&t_before_transf, 2)[..],
            &repeat_each(&p_before_transf, 2)[..],
            &repeat_each(&r_before_transf, 2)[..],
        ]
        .concat();

        let s_t_p_r_len = s_before_transf.len()
            + t_before_transf.len()
            + p_before_transf.len()
            + r_before_transf.len();
        let xor_luts = [
            &repeat_lut(self.lookup.bitxor_with_fwd_transf(), s_t_p_r_len)[..],
            &repeat_luts_cycled(&[self.lookup.fwd_lo(), self.lookup.fwd_hi()], s_t_p_r_len),
        ]
        .concat();

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut xor_and_fwd_stage, &xor_luts);

        let parts = split_by_chunk_sizes(
            &xor_and_fwd_stage,
            &[
                s_before_transf.len(),
                t_before_transf.len(),
                p_before_transf.len(),
                r_before_transf.len(),
                2 * s_before_transf.len(),
                2 * t_before_transf.len(),
                2 * p_before_transf.len(),
                2 * r_before_transf.len(),
            ],
        );
        let [
            s_xor_s1,
            t_xor_t1,
            p_xor_p1,
            r_xor_r1,
            s_and_s1,
            t_and_t1,
            p_and_p1,
            r_and_r1,
        ] = <[_; 8]>::try_from(parts).unwrap();

        let (s0, s1) = split_chunks_even_odd(&s_and_s1, 1);
        let (t0, t1) = split_chunks_even_odd(&t_and_t1, 1);
        let (p0, p1) = split_chunks_even_odd(&p_and_p1, 1);
        let (r0, r1) = split_chunks_even_odd(&r_and_r1, 1);

        // Compute the required GF(2^2) multiplications for each pair.
        let mut mul_stage = [
            &self.fpga_key.pack_slices(&s_xor_s1, &t_xor_t1)[..],
            &self.fpga_key.pack_slices(&s0, &t0)[..],
            &self.fpga_key.pack_slices(&s1, &t1)[..],
            &self.fpga_key.pack_slices(&p_xor_p1, &r_xor_r1)[..],
            &self.fpga_key.pack_slices(&p0, &r0)[..],
            &self.fpga_key.pack_slices(&p1, &r1)[..],
        ]
        .concat();

        // Duplicate LUT pack for both pairs.
        let mul_luts = repeat_luts_cycled(
            &[
                &repeat_lut(self.lookup.gf2_mul(), s_xor_s1.len() + s0.len())[..],
                &repeat_lut(self.lookup.gf2_mul_phi(), s1.len())[..],
            ]
            .concat(),
            2,
        );

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut mul_stage, &mul_luts);

        let parts_mul = split_by_chunk_sizes(
            &mul_stage,
            &[
                s_xor_s1.len(),
                s0.len(),
                s1.len(),
                p_xor_p1.len(),
                p0.len(),
                r1.len(),
            ],
        );

        let [
            st_xor_mul,
            s0t0_mul,
            s1t1_phi_mul,
            pr_xor_mul,
            p0r0_mul,
            p1r1_phi_mul,
        ] = <[_; 6]>::try_from(parts_mul).unwrap();

        // XOR the required products for both pairs.
        let st_hi2 = self.fpga_key.pack_slices(&st_xor_mul, &s0t0_mul);
        let st_lo2 = self.fpga_key.pack_slices(&s1t1_phi_mul, &s0t0_mul);
        let pr_hi2 = self.fpga_key.pack_slices(&pr_xor_mul, &p0r0_mul);
        let pr_lo2 = self.fpga_key.pack_slices(&p1r1_phi_mul, &p0r0_mul);

        let st_mul = interleave_low_high(&st_lo2, &st_hi2);
        let pr_mul = interleave_low_high(&pr_lo2, &pr_hi2);

        let mut combined = [&st_mul[..], &pr_mul[..]].concat();
        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut combined, self.lookup.lut_bitxor());

        // Transform the results back into GF(2^4).
        let (st_blocks, pr_blocks) = combined.split_at(st_mul.len());
        let (st_lower, st_upper) = split_chunks_even_odd(st_blocks, 1);
        let (pr_lower, pr_upper) = split_chunks_even_odd(pr_blocks, 1);

        let mut inv_stage = [
            &repeat_each(&self.fpga_key.pack_slices(&st_lower, &st_upper), 2)[..],
            &repeat_each(&self.fpga_key.pack_slices(&pr_lower, &pr_upper), 2)[..],
        ]
        .concat();

        let inv_luts = repeat_luts_cycled(
            &[self.lookup.inv_lo(), self.lookup.inv_hi()],
            st_lower.len() + pr_lower.len(),
        );

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut inv_stage, &inv_luts);
        inv_stage
    }

    /// Wrapper which does multiplication in GF(2^4) after choosing single or double path based on
    /// input shape.
    ///
    /// # Arguments
    /// - `input` – An array of vector(s) of operand pairs.
    ///
    /// # Returns
    /// The multiplication result as a `Vec<Ciphertext>` of 32 or 64 blocks based on input shape.
    ///
    /// # Panics
    /// Panics if the number of operand pairs is not 1 or 2.
    pub fn gf2_4_mul(&self, inputs: &mut [Vec<Vec<Ciphertext>>]) -> Vec<Ciphertext> {
        match inputs {
            [pair] => self.gf2_4_mul_single(pair),
            [first, second] => self.gf2_4_mul_double(first, second),
            _ => panic!(
                "gf2_4_mul expects 1 or 2 operand pairs, got {}",
                inputs.len()
            ),
        }
    }

    /// Computes two independent results for each byte in a single batched lookup:
    /// 1. Blockwise bitwise XOR of `a₀` and `a₁`
    /// 2. λ(a₁)²
    ///
    /// # Arguments
    /// - `a0_in_blocks` – Vector of ciphertext blocks encoding a₀.
    /// - `a1_in_blocks` – Vector of ciphertext blocks encoding a₁.
    ///
    /// # Returns
    /// `(a₀ ⊕ a₁, λ(a₁)²)`, for each byte where each as a `Vec<Ciphertext>`
    /// of 32 blocks (2 blocks per byte).
    pub fn compute_xor_and_lambda_sq(
        &self,
        a0_in_blocks: &[Ciphertext],
        a1_in_blocks: &[Ciphertext],
    ) -> (Vec<Ciphertext>, Vec<Ciphertext>) {
        let a0_and_a1_packed = self.fpga_key.pack_slices(a0_in_blocks, a1_in_blocks);

        let a1_packed = self.fpga_key.split_and_pack(a1_in_blocks, 1);

        let mut cts_xor_and_lambda_sq =
            [&a0_and_a1_packed[..], &repeat_each(&a1_packed, 2)[..]].concat();

        let luts_xor_and_lambda_sq = [
            &repeat_lut(self.lookup.lut_bitxor(), a0_and_a1_packed.len())[..],
            &repeat_luts_cycled(
                &[self.lookup.lambda_sq_lo2(), self.lookup.lambda_sq_hi2()],
                a1_packed.len(),
            )[..],
        ]
        .concat();

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut cts_xor_and_lambda_sq, &luts_xor_and_lambda_sq);

        let (a0_xor_a1_in_blocks, lambda_sq_a1_in_blocks) =
            cts_xor_and_lambda_sq.split_at(a0_and_a1_packed.len());

        (
            a0_xor_a1_in_blocks.to_vec(),
            lambda_sq_a1_in_blocks.to_vec(),
        )
    }

    /// Computes Δ⁻¹, the multiplicative inverse of  `Δ = a₀ · (a₀ ⊕ a₁) ⊕ λ(a₁)²` in GF(2⁴)
    /// for each byte within the state.
    ///
    /// # Arguments
    /// - `a0_in_blocks` – Vector of ciphertext blocks encoding a₀ values.
    /// - `a0_xor_a1_in_blocks` — Vector of ciphertext blocks encoding a₀ ⊕ a₁ values.
    /// - `lambda_sq_a1_in_blocks`— Vector of ciphertext blocks encoding λ(a₁)² values.
    ///
    /// # Returns
    /// `Δ⁻¹` for each byte as a `Vec<Ciphertext>` of 32 blocks (2 blocks per byte).
    pub fn compute_delta_inv(
        &self,
        a0_in_blocks: &[Ciphertext],
        a0_xor_a1_in_blocks: &[Ciphertext],
        lambda_sq_a1_in_blocks: &[Ciphertext],
    ) -> Vec<Ciphertext> {
        let mut inputs = [vec![a0_in_blocks.to_vec(), a0_xor_a1_in_blocks.to_vec()]];
        let a0_mul_a0xor_a1 = self.gf2_4_mul(&mut inputs);

        let mut delta_in_blocks = self
            .fpga_key
            .pack_slices(&a0_mul_a0xor_a1, lambda_sq_a1_in_blocks);
        self.fpga_key.apply_same_lookup_vector_packed_assign(
            &mut delta_in_blocks,
            self.lookup.lut_bitxor(),
        );

        let (delta_lo2, delta_hi2) = split_chunks_even_odd(&delta_in_blocks, 1);
        let delta_packed = self.fpga_key.pack_slices(&delta_lo2, &delta_hi2);

        let mut delta_inv_in_blocks: Vec<_> = repeat_each(&delta_packed, 2);

        let luts_delta_inv = repeat_luts_cycled(
            &[self.lookup.delta_inv_lo2(), self.lookup.delta_inv_hi2()],
            delta_lo2.len(),
        );

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut delta_inv_in_blocks, &luts_delta_inv);

        delta_inv_in_blocks
    }

    /// Computes the s-box substitute of each byte in the state using
    /// tower field construction.
    ///
    /// # Arguments
    /// - `state_in_blocks` – Vector of ciphertext blocks containing the flattened state.
    ///
    /// # Returns
    /// `state⁻¹` as a `Vec<Ciphertext>` in the same layout.
    pub fn compute_substitute_in_tower_field(
        &self,
        state_in_blocks: &[Ciphertext],
    ) -> Vec<Ciphertext> {
        let (a0_in_blocks, a1_in_blocks) = split_chunks_even_odd(state_in_blocks, 2);

        let (a0_xor_a1_in_blocks, lambda_sq_a1_in_blocks) =
            self.compute_xor_and_lambda_sq(&a0_in_blocks, &a1_in_blocks);

        let delta_inv_in_blocks =
            self.compute_delta_inv(&a0_in_blocks, &a0_xor_a1_in_blocks, &lambda_sq_a1_in_blocks);

        let mut inputs = [
            vec![a0_xor_a1_in_blocks.clone(), delta_inv_in_blocks.clone()],
            vec![a1_in_blocks, delta_inv_in_blocks],
        ];

        let products = self.gf2_4_mul(&mut inputs);

        let (s_t_mul_in_blocks, p_r_mul_in_blocks) = products.split_at(a0_xor_a1_in_blocks.len());

        let b_in_blocks: Vec<Ciphertext> =
            interleave_chunks_of_two(s_t_mul_in_blocks, p_r_mul_in_blocks);

        self.fpga_key.duplicate_radix4_packed_chunks(&b_in_blocks)
    }
}
