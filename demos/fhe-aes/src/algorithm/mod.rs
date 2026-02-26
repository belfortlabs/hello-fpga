pub mod constants;
pub mod engine;
pub mod lookup;
pub mod parallel;
pub mod utils;

use engine::inv_round_ops::inv_shift_rows;
use engine::round_ops::shift_rows;
use engine::state::FheAesByte;
use tfhe::integer::fpga::BelfortServerKey;

use crate::algorithm::constants::{
    AES_128_KEY_SIZE, AES_BLOCK_SIZE, BLOCKS_PER_BYTE, BYTES_IN_WORD, COLS_IN_STATE,
    EXPANDED_KEY_SIZE, RCON, ROUNDS, WORDS_IN_KEY,
};
use crate::algorithm::engine::FheAesEngine;
use crate::algorithm::engine::state::FheAesState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AesDir {
    Encrypt,
    Decrypt,
}

pub struct FheAes<'a> {
    trivial_zero_ct: FheAesByte,
    aes_engine: FheAesEngine<'a>,
}

impl<'a> FheAes<'a> {
    pub fn new(fpga_key: &'a BelfortServerKey) -> Self {
        let trivial_zero_ct = fpga_key
            .pbs_key()
            .create_trivial_zero_radix(BLOCKS_PER_BYTE);

        let aes_engine = FheAesEngine::new(fpga_key);

        Self {
            trivial_zero_ct,
            aes_engine,
        }
    }

    /// Expand an AES key into a buffer of encrypted round keys.
    ///
    /// This function takes an initial key and expands it into a series of round
    /// keys, which are used in each round of the AES encryption/decryption process.
    /// The expanded keys are stored in a single contiguous FheAesByte buffer.
    ///
    /// # Arguments
    /// - `key`: An array containing the initial AES key.
    ///
    /// # Returns
    /// A `[FheAesByte; EXPANDED_KEY_SIZE]` array containing the expanded keys.
    pub fn expand_key(
        &self,
        key: &[FheAesByte; AES_128_KEY_SIZE],
    ) -> [FheAesByte; EXPANDED_KEY_SIZE] {
        let mut expanded_key: [FheAesByte; EXPANDED_KEY_SIZE] =
            core::array::from_fn(|_| self.trivial_zero_ct.clone()); // Fixed buffer for expanded key

        // Copy the initial key as the first round key
        for word_idx in 0..WORDS_IN_KEY {
            let offset = word_idx * BYTES_IN_WORD;
            let end = offset + BYTES_IN_WORD;
            expanded_key[offset..end].clone_from_slice(&key[offset..end]);
        }

        for word_idx in WORDS_IN_KEY..(COLS_IN_STATE * (ROUNDS + 1)) {
            // Load the last word from the previous round key into `temp`
            let prev_offset = (word_idx - 1) * BYTES_IN_WORD;
            let prev_word = expanded_key[prev_offset..prev_offset + BYTES_IN_WORD].to_vec();

            let mut temp = prev_word.clone();

            if word_idx % WORDS_IN_KEY == 0 {
                // temp ← SubWord(RotWord(temp)) ⊕ Rcon[i / Nk]
                let round_constant = RCON[word_idx / WORDS_IN_KEY].into();
                self.aes_engine.key_schedule_core(&mut temp, round_constant);
            }

            // Generate the next word of the round key
            let base_offset = (word_idx - WORDS_IN_KEY) * BYTES_IN_WORD;
            let base_word = expanded_key[base_offset..base_offset + BYTES_IN_WORD].to_vec();

            let next_word = self.aes_engine.generate_next_round_word(&temp, &base_word);

            let dest_offset = word_idx * BYTES_IN_WORD;
            expanded_key[dest_offset..dest_offset + BYTES_IN_WORD].clone_from_slice(&next_word);
        }
        expanded_key
    }

    /// Encrypts or decrypts a single block using the AES algorithm based on `dir`.
    ///
    /// # Arguments
    /// - `dir`: `AesDir::Encrypt` or `AesDir::Decrypt`
    /// - `block`: A reference to an FheAesByte array representing the block to operate on.
    /// - `expanded_key`: A reference to a FheAesByte array representing the expanded key.
    ///
    /// # Returns
    /// A `[FheAesByte; AES_BLOCK_SIZE]` representing the encrypted/decrypted block.
    pub(crate) fn aes_block(
        &self,
        dir: AesDir,
        input: &[FheAesByte; AES_BLOCK_SIZE],
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
    ) -> [FheAesByte; AES_BLOCK_SIZE] {
        let mut state = FheAesState::state_from_column_major(input);

        match dir {
            AesDir::Encrypt => {
                self.aes_engine.add_round_key(0, &mut state, expanded_key);

                for round in 1..=ROUNDS {
                    self.aes_engine.sub_bytes(&mut state);
                    shift_rows(&mut state);

                    if round != ROUNDS {
                        self.aes_engine.mix_columns(&mut state);
                    }
                    self.aes_engine
                        .add_round_key(round, &mut state, expanded_key);
                }
            }
            AesDir::Decrypt => {
                self.aes_engine
                    .add_round_key(ROUNDS, &mut state, expanded_key);

                for round in (0..ROUNDS).rev() {
                    inv_shift_rows(&mut state);
                    self.aes_engine.inv_sub_bytes(&mut state);
                    self.aes_engine
                        .add_round_key(round, &mut state, expanded_key);

                    if round != 0 {
                        self.aes_engine.inv_mix_columns(&mut state);
                    }
                }
            }
        }

        state.state_into_column_major()
    }

    /// Encrypts or decrypts multiple blocks using the AES algorithm based on `dir`.
    ///
    /// # Arguments
    /// - `dir`: `AesDir::Encrypt` or `AesDir::Decrypt`
    /// - `blocks`: A reference to an array of 16-FheAesByte arrays the blocks to operate on.
    /// - `expanded_key`: A reference to a FheAesByte array representing the expanded key.
    ///
    /// # Returns
    /// A `Vec<[FheAesByte; AES_BLOCK_SIZE]> representing the encrypted/decrypted blocks.
    pub(crate) fn aes_blocks(
        &self,
        dir: AesDir,
        blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]> {
        blocks
            .iter()
            .map(|block| self.aes_block(dir, block, expanded_key))
            .collect()
    }

    /// Encrypts multiple using the AES algorithm.
    ///
    /// # Arguments
    /// - `blocks`: A reference to an array of 16-FheAesByte arrays representing the plaintext
    ///   blocks to be encrypted.
    /// - `expanded_key`: A reference to a FheAesByte array representing the expanded key.
    ///
    /// # Returns
    /// A `Vec<[FheAesByte; AES_BLOCK_SIZE]> representing the encrypted plaintext blocks.
    pub fn aes_enc_blocks(
        &self,
        blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]> {
        self.aes_blocks(AesDir::Encrypt, blocks, expanded_key)
    }

    /// Decrypts multiple using the AES algorithm.
    ///
    /// # Arguments
    /// - `blocks`: A reference to an array of 16-FheAesByte arrays representing the ciphertext
    ///   blocks to be decrypted.
    /// - `expanded_key`: A reference to a FheAesByte array representing the expanded key.
    ///
    /// # Returns
    /// A `Vec<[FheAesByte; AES_BLOCK_SIZE]> representing the decrypted ciphertext blocks.
    pub fn aes_dec_blocks(
        &self,
        blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]> {
        self.aes_blocks(AesDir::Decrypt, blocks, expanded_key)
    }
}
