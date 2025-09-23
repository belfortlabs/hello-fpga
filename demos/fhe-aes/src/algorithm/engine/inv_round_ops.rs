use tfhe::shortint::Ciphertext;

use crate::algorithm::{
    constants::{
        BLOCKS_IN_STATE, BLOCKS_PER_BYTE, BYTES_IN_STATE, COLS_IN_STATE, INV_MIX_COL_ORDERS,
        ROWS_IN_STATE, SHIFT_ROW_PAIRS,
    },
    utils::{FheAesCiphertextUtils, extend_in_order, repeat_chunks_n_times, repeat_luts_cycled},
};

use super::{FheAesEngine, state::FheAesState};

impl FheAesEngine<'_> {
    /// Performs the InvMixColumns transformation for the AES decryption.
    ///
    /// # Arguments
    /// - `state`: The current state of the cipher.
    ///
    /// # Note
    /// Modifies the state in place.
    pub fn inv_mix_columns(&self, state: &mut FheAesState) {
        // Expand and repeat the state to multiply each byte with constants 9, 11, 13 and 14.
        let expanded_state = self.fpga_key.expand_state_blocks(state);
        let mut repeated_state = repeat_chunks_n_times(&expanded_state, 8, 4);

        let mul_lut_bases = [
            &self.lookup.mul9_base()[..],
            &self.lookup.mul11_base()[..],
            &self.lookup.mul13_base()[..],
            &self.lookup.mul14_base()[..],
        ]
        .concat();

        let repeated_luts = repeat_luts_cycled(&mul_lut_bases, BYTES_IN_STATE);

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut repeated_state, &repeated_luts);

        // XOR each group of consecutive 4 products which are the partial products to construct each
        // full product.
        let mut xor_inputs = self
            .fpga_key
            .split_and_pack(&repeated_state, BLOCKS_PER_BYTE);

        let lut_bitxor = self.lookup.lut_bitxor();

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_inputs, &lut_bitxor);

        // Restructure the products into a 3D vector indexed by (row, col, constant index) where
        // - 0 -> 9
        // - 1 -> 11
        // - 2 -> 13
        // - 3 -> 14
        let mut products: Vec<Vec<Vec<Vec<Ciphertext>>>> = vec![vec![vec![vec![]; 4]; 4]; 4];
        for (row, product_row) in products.iter_mut().enumerate().take(ROWS_IN_STATE) {
            for (col, product_col) in product_row.iter_mut().enumerate().take(COLS_IN_STATE) {
                let base_idx = (row * 4 + col) * 16;
                for (constant_idx, product_cell) in product_col.iter_mut().enumerate().take(4) {
                    *product_cell = xor_inputs
                        [base_idx + constant_idx * 4..base_idx + constant_idx * 4 + 4]
                        .to_vec();
                }
            }
        }

        // Group the multiplication results according to the columns of the AES state.
        // Within each column group, the corresponding elements will be XORed together
        // to reconstruct the final COLS_IN_STATE-major order.
        let mut grouped_products: Vec<Vec<Ciphertext>> = (0..COLS_IN_STATE)
            .map(|_| Vec::with_capacity(BLOCKS_IN_STATE))
            .collect::<Vec<_>>();

        for col in 0..COLS_IN_STATE {
            for row in 0..ROWS_IN_STATE {
                extend_in_order(
                    &mut grouped_products[row],
                    &products,
                    row,
                    col,
                    &INV_MIX_COL_ORDERS[row],
                );
            }
        }

        let packed_rows_0_1 = self
            .fpga_key
            .pack_slices(&grouped_products[0], &grouped_products[1]);
        let packed_rows_2_3 = self
            .fpga_key
            .pack_slices(&grouped_products[2], &grouped_products[3]);

        let mut xor_step1 = [packed_rows_0_1, packed_rows_2_3].concat();

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_step1, &lut_bitxor);

        let (xor_step1_left, xor_step1_right) = xor_step1.split_at(64);

        let mut xor_step2 = self.fpga_key.pack_slices(xor_step1_left, xor_step1_right);

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_step2, &lut_bitxor);

        state.state_from_column_major_in_blocks_assign(&xor_step2);
    }

    /// Performs the InvSubBytes operation using tower field construction for AES decryption.
    ///
    /// # Arguments
    /// - `state`: The current state of the cipher.
    ///
    /// # Note
    /// Modifies the state in place.
    pub fn inv_sub_bytes(&self, state: &mut FheAesState) {
        let mut state_in_blocks = self.fpga_key.expand_state_blocks(state);

        self.tower_field.apply_isomorphism(
            &mut state_in_blocks,
            &self.lookup.fwd_with_inv_affine_base(),
        );

        let mut inv_in_tower_field = self
            .tower_field
            .compute_substitute_in_tower_field(&state_in_blocks);

        self.tower_field
            .apply_isomorphism(&mut inv_in_tower_field, &self.lookup.inv_base());

        state.state_from_row_major_in_blocks_assign(&inv_in_tower_field);
    }
}

/// Performs the InvShiftRows transformation for AES decryption.
///
/// This function cyclically shifts the rows of the state matrix to the right.
/// Each row is shifted by a different offset depending on its index.
///
/// # Arguments
/// - `state`: The current state of the cipher.
///
/// # Note
/// Modifies the state in place.
pub fn inv_shift_rows(state: &mut FheAesState) {
    for (row, offset) in SHIFT_ROW_PAIRS {
        state.rot_row_right(*row, *offset);
    }
}
