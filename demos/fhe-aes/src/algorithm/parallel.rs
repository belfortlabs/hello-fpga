use tfhe::BelfortServerKey;

use crate::algorithm::constants::{AES_BLOCK_SIZE, EXPANDED_KEY_SIZE};

use super::FheAes;
use super::engine::state::FheAesByte;

impl FheAes<'_> {
    pub fn aes_enc_blocks_parallelized(
        &self,
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
        fhe_blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]> {
        self.aes_engine.par_aes_blocks(
            expanded_key,
            fhe_blocks,
            |fpga_key: &BelfortServerKey,
             blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
             exp_key: &[FheAesByte; EXPANDED_KEY_SIZE]| {
                let fhe = FheAes::new(fpga_key);
                fhe.aes_enc_blocks(blocks, exp_key)
            },
        )
    }

    pub fn aes_dec_blocks_parallelized(
        &self,
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
        fhe_blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]> {
        self.aes_engine.par_aes_blocks(
            expanded_key,
            fhe_blocks,
            |fpga_key: &BelfortServerKey,
             blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
             exp_key: &[FheAesByte; EXPANDED_KEY_SIZE]| {
                let fhe = FheAes::new(fpga_key);
                fhe.aes_dec_blocks(blocks, exp_key)
            },
        )
    }
}
