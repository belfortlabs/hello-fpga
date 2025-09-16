use std::{
    sync::{
        Arc,
        mpsc::{Sender, channel},
    },
    thread,
};

use tfhe::BelfortServerKey;

use crate::algorithm::constants::{AES_BLOCK_SIZE, EXPANDED_KEY_SIZE};

use super::{FheAesEngine, state::FheAesByte};

pub struct AesOperationPackage<F>
where
    F: Fn(
            &BelfortServerKey,
            &[[FheAesByte; AES_BLOCK_SIZE]],
            &[FheAesByte; EXPANDED_KEY_SIZE],
        ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]>
        + Send
        + Sync,
{
    pub idx: usize,
    pub blocks: Vec<[FheAesByte; AES_BLOCK_SIZE]>,
    pub operation: Arc<F>,
}

pub fn split_blocks_evenly<T: Clone>(blocks: &[T], num_fpgas: usize) -> Vec<Vec<T>> {
    let total = blocks.len();
    if num_fpgas == 0 || total == 0 {
        return Vec::new();
    }

    let base = total / num_fpgas;
    let extra = total % num_fpgas;

    let mut result = Vec::with_capacity(num_fpgas);
    let mut start = 0;

    for i in 0..num_fpgas {
        let size = if i < extra { base + 1 } else { base };
        if size > 0 {
            result.push(blocks[start..start + size].to_vec());
            start += size;
        }
    }

    result
}

impl FheAesEngine<'_> {
    pub fn generate_fpga_threads_for_aes<F>(
        &self,
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
        result_sender: &Sender<(usize, Vec<[FheAesByte; AES_BLOCK_SIZE]>)>,
    ) -> Vec<Sender<AesOperationPackage<F>>>
    where
        F: Send
            + Sync
            + Fn(
                &BelfortServerKey,
                &[[FheAesByte; AES_BLOCK_SIZE]],
                &[FheAesByte; EXPANDED_KEY_SIZE],
            ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]>
            + 'static,
    {
        let fpga_indexes = self.fpga_key.fpga_utils.fpga_indexes.clone();
        #[cfg(feature = "emulate_fpga")]
        let fpga_indexes: Vec<usize> =
            crate::core_crypto::fpga::BelfortFpgaUtils::get_fpga_indices_from_env();

        fpga_indexes
            .into_iter()
            .map(|i| {
                let expanded_key = expanded_key.clone();

                let (tx, rx) = channel::<AesOperationPackage<F>>();
                let res_sender = result_sender.clone();
                let mut thread_key = self.fpga_key.clone();

                thread::spawn(move || {
                    thread_key.fpga_utils.fpga_indexes = vec![i];

                    while let Ok(p) = rx.recv() {
                        let result_vec =
                            (p.operation)(&thread_key, p.blocks.as_slice(), &expanded_key);
                        let _ = res_sender.send((p.idx, result_vec));
                    }
                });
                tx
            })
            .collect()
    }

    pub fn par_aes_blocks<F>(
        &self,
        expanded_key: &[FheAesByte; EXPANDED_KEY_SIZE],
        fhe_blocks: &[[FheAesByte; AES_BLOCK_SIZE]],
        op: F,
    ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]>
    where
        F: Send
            + Sync
            + Fn(
                &BelfortServerKey,
                &[[FheAesByte; AES_BLOCK_SIZE]],
                &[FheAesByte; EXPANDED_KEY_SIZE],
            ) -> Vec<[FheAesByte; AES_BLOCK_SIZE]>
            + 'static,
    {
        let num_fpgas = self.fpga_key.fpga_utils.fpga_indexes.len();
        #[cfg(feature = "emulate_fpga")]
        let num_fpgas =
            crate::core_crypto::fpga::BelfortFpgaUtils::get_fpga_indices_from_env().len();

        let (tx_res, rx_res) = channel::<(usize, Vec<[FheAesByte; AES_BLOCK_SIZE]>)>();
        let op = Arc::new(op);

        let threads: Vec<_> = self.generate_fpga_threads_for_aes(expanded_key, &tx_res);
        let chunks: Vec<_> = split_blocks_evenly(fhe_blocks, num_fpgas);

        chunks.iter().enumerate().for_each(|(idx, chunk)| {
            let thread = &threads[idx % num_fpgas];
            let _ = thread.send(AesOperationPackage {
                idx,
                blocks: chunk.clone(),
                operation: Arc::clone(&op),
            });
        });

        let mut results = Vec::new();
        for _ in 0..chunks.len() {
            results.push(rx_res.recv().unwrap());
        }

        drop(tx_res);
        drop(threads);

        results.sort_by_key(|(i, _)| *i);
        results
            .into_iter()
            .flat_map(|(_, v)| v.into_iter())
            .collect()
    }
}
