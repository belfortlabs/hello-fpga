use tfhe::{
    integer::{IntegerCiphertext, RadixCiphertext},
    shortint::Ciphertext,
};

use crate::algorithm::{
    constants::{
        BLOCKS_IN_STATE, BLOCKS_PER_BYTE, BYTES_IN_STATE, COLS_IN_STATE, EXPANDED_KEY_SIZE,
        MIXCOL_PAIRS, ROWS_IN_STATE, SHIFT_ROW_PAIRS,
    },
    engine::state::FheAesState,
    utils::{FheAesCiphertextUtils, repeat_luts_cycled, split_chunks_even_odd},
};

use super::FheAesEngine;
use super::state::FheAesByte;

impl FheAesEngine<'_> {
    /// Performs AES MixColumns operation.
    ///
    /// # Arguments
    /// - `state`: The current state of the cipher, represented as a mutable 2D array.
    ///
    /// * Modifies the state in place.
    pub fn mix_columns(&self, state: &mut FheAesState) {
        // XOR row pairs (0 ^ 1, 1 ^ 2, 2 ^ 3, 0 ^ 3) column-wise.
        let mut xor_inputs_per_column = Vec::with_capacity(BLOCKS_IN_STATE);

        for col in 0..COLS_IN_STATE {
            for (i, j) in MIXCOL_PAIRS {
                let packed_pair = self
                    .fpga_key
                    .pack_slices(&state.get_in_blocks(*i, col), &state.get_in_blocks(*j, col));
                xor_inputs_per_column.extend(packed_pair);
            }
        }

        self.fpga_key.apply_same_lookup_vector_packed_assign(
            &mut xor_inputs_per_column,
            self.lookup.lut_bitxor(),
        );

        let xor_pairs: Vec<Vec<_>> = xor_inputs_per_column
            .chunks(BLOCKS_PER_BYTE)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Restructure the XOR results into a 3D vector indexed by (xored rows, col) where
        // - 0 -> 0 ^ 1
        // - 1 -> 1 ^ 2
        // - 2 -> 2 ^ 3
        // - 3 -> 0 ^ 3
        let mut xored_rows: Vec<Vec<Vec<Ciphertext>>> = vec![
            (0..ROWS_IN_STATE)
                .map(|_| Vec::with_capacity(BLOCKS_PER_BYTE))
                .collect::<Vec<_>>();
            COLS_IN_STATE
        ];

        for (col, chunk) in xor_pairs.chunks_exact(BLOCKS_PER_BYTE).enumerate() {
            for row in 0..ROWS_IN_STATE {
                xored_rows[row][col].clone_from(&chunk[row]);
            }
        }

        // XOR (0 ^ 1) with (2 ^ 3) to get full column XOR.
        let mut full_column_xor_in_blocks = (0..COLS_IN_STATE)
            .flat_map(|col| {
                self.fpga_key
                    .pack_slices(&xored_rows[0][col], &xored_rows[2][col])
            })
            .collect::<Vec<_>>();

        self.fpga_key.apply_same_lookup_vector_packed_assign(
            &mut full_column_xor_in_blocks,
            self.lookup.lut_bitxor(),
        );

        let full_column_xor: Vec<_> = full_column_xor_in_blocks
            .chunks(BLOCKS_PER_BYTE)
            .map(|chunk| chunk.to_vec())
            .collect();

        // Multiply each XOR result with 2.
        let mut transformed_rows = Vec::with_capacity(16);
        for col in 0..COLS_IN_STATE {
            for row in xored_rows.iter().take(ROWS_IN_STATE) {
                transformed_rows.push(RadixCiphertext::from(row[col].clone()));
            }
        }

        let mut expanded_transforms = self.fpga_key.expand_lo_hi_batch(&transformed_rows);

        let mul2_luts = repeat_luts_cycled(&self.lookup.mul2_base(), BYTES_IN_STATE);

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut expanded_transforms, &mul2_luts);

        let (mul2_with_lower_bits, mul2_with_higher_bits) =
            split_chunks_even_odd(&expanded_transforms, BLOCKS_PER_BYTE);

        let mut xor_shifted_mul = self
            .fpga_key
            .pack_slices(&mul2_with_lower_bits, &mul2_with_higher_bits);
        self.fpga_key.apply_same_lookup_vector_packed_assign(
            &mut xor_shifted_mul,
            self.lookup.lut_bitxor(),
        );

        // XOR the multiplied XORs for each column with its corresponding full column XOR.
        let mut xor_step1 = Vec::with_capacity(BLOCKS_IN_STATE);

        for row in 0..ROWS_IN_STATE {
            for (col, col_val) in full_column_xor.iter().take(COLS_IN_STATE).enumerate() {
                let idx_start = col * 16 + row * 4;

                xor_step1.extend(
                    self.fpga_key.pack_slices(
                        &xor_shifted_mul[idx_start..idx_start + BLOCKS_PER_BYTE]
                            .iter()
                            .rev()
                            .cloned()
                            .collect::<Vec<_>>(),
                        &col_val.clone(),
                    ),
                );
            }
        }

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_step1, self.lookup.lut_bitxor());

        // XOR the results from the first XOR step with their corresponding state elements.
        let mut xor_step2 = Vec::with_capacity(BLOCKS_IN_STATE);
        for row in 0..ROWS_IN_STATE {
            for col in 0..COLS_IN_STATE {
                let idx_start = (row * COLS_IN_STATE + col) * BLOCKS_PER_BYTE;
                xor_step2.extend(self.fpga_key.pack_slices(
                    &state.get_in_blocks(row, col),
                    &xor_step1[idx_start..idx_start + BLOCKS_PER_BYTE],
                ));
            }
        }

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_step2, self.lookup.lut_bitxor());

        state.state_from_row_major_in_blocks_assign(&xor_step2);
    }

    /// Performs AES SubBytes operation using tower field construction.
    ///
    /// # Arguments
    /// - `state`: The current state of the cipher, represented as a mut 2D array.
    ///
    /// * Modifies the state in place.
    pub fn sub_bytes(&self, state: &mut FheAesState) {
        let mut state_in_blocks = self.fpga_key.expand_state_blocks(state);

        self.tower_field
            .apply_isomorphism(&mut state_in_blocks, &self.lookup.fwd_base());

        let mut inv_in_tower_field = self
            .tower_field
            .compute_substitute_in_tower_field(&state_in_blocks);

        self.tower_field
            .apply_isomorphism(&mut inv_in_tower_field, &self.lookup.inv_with_affine_base());

        state.state_from_row_major_in_blocks_assign(&inv_in_tower_field);
    }

    /// Performs the addition of a round key to the state using an XOR operation.
    ///
    /// # Arguments
    /// - `round`: The current round number.
    /// - `state`: The current state of the cipher, represented as a mut 2D array.
    /// - `expanded_key`: The expanded key buffer containing all round keys.
    // # Note
    ///   Modifies the state in place.
    pub fn add_round_key(
        &self,
        round: usize,
        state: &mut FheAesState,
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
    ) {
        let mut round_key: Vec<Ciphertext> = Vec::with_capacity(BLOCKS_IN_STATE);

        for i in 0..4 {
            for j in 0..4 {
                round_key.extend(
                    expanded_key[round * COLS_IN_STATE * 4 + i * COLS_IN_STATE + j]
                        .blocks()
                        .iter()
                        .cloned(),
                )
            }
        }

        let mut col_major_state: Vec<Ciphertext> = Vec::with_capacity(BLOCKS_IN_STATE);

        for col in 0..COLS_IN_STATE {
            for row in state.rows() {
                if let Some(ct) = row.get(col) {
                    col_major_state.extend(ct.blocks().iter().cloned());
                }
            }
        }

        let mut packed = self.fpga_key.pack_slices(&round_key, &col_major_state);

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut packed, self.lookup.lut_bitxor());

        state.state_from_column_major_in_blocks_assign(&packed);
    }
}

/// Perform the ShiftRows transformation for AES encryption.
///
/// This function cyclically shifts the rows of the state matrix to the left.
/// Each row is shifted by a different offset depending on its index.
///
/// # Arguments
/// - `state`: The current state of the cipher, represented as a mut 2D array.
///
/// # Note
///   Modifies the state in place.
pub fn shift_rows(state: &mut FheAesState) {
    for (row, offset) in SHIFT_ROW_PAIRS {
        state.rot_row_left(*row, *offset);
    }
}
