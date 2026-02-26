use tfhe::integer::{IntegerCiphertext, RadixCiphertext};

use crate::algorithm::{
    constants::{BLOCKS_PER_BYTE, BYTES_IN_WORD},
    utils::{FheAesCiphertextUtils, repeat_luts_cycled, split_chunks_even_odd},
};

use super::FheAesEngine;

impl FheAesEngine<'_> {
    /// Performs the AES key-schedule core operation:
    /// `temp ← SubWord(RotWord(temp)) ⊕ Rcon[i / Nk]`
    ///
    /// # Arguments
    /// - `temp`: A mutable vector of RadixCiphertext representing a 4-byte word
    /// - `y`: The round constant
    ///
    /// * Modifies `temp` in place.
    pub fn key_schedule_core(&self, temp: &mut [RadixCiphertext], y: u64) {
        temp.rotate_left(1);

        let mut temp_in_blocks = self.fpga_key.expand_lo_hi_batch(temp);

        self.tower_field
            .apply_isomorphism(&mut temp_in_blocks, &self.lookup.fwd_base());

        let mut inv_in_tower_field = self
            .tower_field
            .compute_substitute_in_tower_field(&temp_in_blocks);

        // Construct lookup table vector with the first byte being XORed with the round constant
        let lut_inv_with_affine_and_xor_base = self.lookup.inv_with_affine_and_xor_base(y);
        let lut_inv_with_affine_and_xor_base =
            lut_inv_with_affine_and_xor_base.iter().collect::<Vec<_>>();
        let lut_inv_with_affine_base = self.lookup.inv_with_affine_base();

        let key_schedule_core_luts = [
            lut_inv_with_affine_and_xor_base,
            repeat_luts_cycled(&lut_inv_with_affine_base, BYTES_IN_WORD - 1),
        ]
        .concat();

        self.fpga_key
            .apply_lookup_vector_packed_assign(&mut inv_in_tower_field, &key_schedule_core_luts);

        let (even_chunks, odd_chunks) = split_chunks_even_odd(&inv_in_tower_field, 4);

        inv_in_tower_field = self.fpga_key.pack_slices(&even_chunks, &odd_chunks);

        self.fpga_key.apply_same_lookup_vector_packed_assign(
            &mut inv_in_tower_field,
            &self.lookup.lut_bitxor(),
        );

        for byte_idx in 0..BYTES_IN_WORD {
            temp[byte_idx] = RadixCiphertext::from(
                inv_in_tower_field[byte_idx * BLOCKS_PER_BYTE..(byte_idx + 1) * BLOCKS_PER_BYTE]
                    .to_vec(),
            );
        }
    }

    /// Computes the next round word in the AES key expansion:
    /// `next_word = temp ⊕ prev_word`
    ///
    /// # Arguments
    /// - `temp`: A vector of RadixCiphertext representing a 4-byte word
    /// - `prev_word`:  A vector of RadixCiphertext representing a 4-byte word
    ///
    /// # Returns
    /// A `Vec<RadixCiphertext>` containing the next word
    pub fn generate_next_round_word(
        &self,
        temp: &[RadixCiphertext],
        prev_word: &[RadixCiphertext],
    ) -> Vec<RadixCiphertext> {
        let mut temp_in_blocks = Vec::with_capacity(BLOCKS_PER_BYTE * BYTES_IN_WORD);
        let mut prev_in_blocks = Vec::with_capacity(BLOCKS_PER_BYTE * BYTES_IN_WORD);

        for byte_idx in 0..BYTES_IN_WORD {
            temp_in_blocks.extend(temp[byte_idx].blocks().to_vec());
            prev_in_blocks.extend(prev_word[byte_idx].blocks().to_vec());
        }

        let mut xor_input = self.fpga_key.pack_slices(&temp_in_blocks, &prev_in_blocks);

        self.fpga_key
            .apply_same_lookup_vector_packed_assign(&mut xor_input, &self.lookup.lut_bitxor());

        let mut next_word = Vec::with_capacity(BYTES_IN_WORD);
        for byte_idx in 0..BYTES_IN_WORD {
            let offset = byte_idx * BLOCKS_PER_BYTE;
            next_word.push(RadixCiphertext::from(
                xor_input[offset..(offset + BLOCKS_PER_BYTE)].to_vec(),
            ));
        }

        next_word.clone()
    }
}
