use tfhe::{
    integer::{IntegerCiphertext, RadixCiphertext},
    shortint::Ciphertext,
};

use crate::algorithm::constants::{AES_BLOCK_SIZE, BLOCKS_PER_BYTE};

pub type FheAesByte = RadixCiphertext;

#[derive(Clone)]
pub struct FheAesState([[FheAesByte; 4]; 4]);

impl FheAesState {
    pub fn state_from_column_major(block: &[FheAesByte; AES_BLOCK_SIZE]) -> Self {
        let init = core::array::from_fn(|_| block[0].clone());
        let mut state: [[FheAesByte; 4]; 4] = core::array::from_fn(|_| init.clone());

        for col in 0..4 {
            for row in 0..4 {
                state[row][col] = block[col * 4 + row].clone();
            }
        }

        Self(state)
    }

    pub fn state_from_column_major_in_blocks_assign(&mut self, flat: &[Ciphertext]) {
        for i in 0..AES_BLOCK_SIZE {
            let ct_blocks = flat[i * BLOCKS_PER_BYTE..(i + 1) * BLOCKS_PER_BYTE].to_vec();
            let col = i / 4;
            let row = i % 4;
            self.0[row][col] = FheAesByte::from_blocks(ct_blocks);
        }
    }

    pub fn state_into_column_major(self) -> [FheAesByte; AES_BLOCK_SIZE] {
        let mut block = core::array::from_fn(|_| self.0[0][0].clone());

        for col in 0..4 {
            for row in 0..4 {
                block[col * 4 + row] = self.0[row][col].clone();
            }
        }

        block
    }

    pub fn state_from_row_major_in_blocks_assign(&mut self, flat: &[Ciphertext]) {
        for i in 0..AES_BLOCK_SIZE {
            let ct_blocks = flat[i * BLOCKS_PER_BYTE..(i + 1) * BLOCKS_PER_BYTE].to_vec();
            let row = i / 4;
            let col = i % 4;
            self.0[row][col] = FheAesByte::from_blocks(ct_blocks); // You must implement this
        }
    }

    pub fn rot_row_left(&mut self, row: usize, offset: usize) {
        let mut tmp = [
            self.0[row][0].clone(),
            self.0[row][1].clone(),
            self.0[row][2].clone(),
            self.0[row][3].clone(),
        ];
        tmp.rotate_left(offset);
        self.0[row].clone_from_slice(&tmp);
    }

    pub fn rot_row_right(&mut self, row: usize, offset: usize) {
        self.rot_row_left(row, 4 - (offset % 4));
    }

    pub fn rows(&self) -> impl Iterator<Item = [&FheAesByte; 4]> + '_ {
        (0..4).map(move |r| [&self.0[r][0], &self.0[r][1], &self.0[r][2], &self.0[r][3]])
    }

    pub fn get_in_blocks(&self, row: usize, col: usize) -> Vec<Ciphertext> {
        self.0[row][col].blocks().to_vec()
    }

    pub fn get_whole(&self) -> [[FheAesByte; 4]; 4] {
        self.0.clone()
    }
}
