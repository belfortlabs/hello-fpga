use std::iter::repeat_n;

use tfhe::integer::{IntegerCiphertext, RadixCiphertext};
use tfhe::shortint::Ciphertext;

use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::integer::fpga::BelfortServerKey;

use crate::algorithm::engine::state::FheAesState;

pub trait FheAesCiphertextUtils {
    fn pack_radix4(&self, ct_low: &Ciphertext, ct_high: &Ciphertext) -> Ciphertext;
    fn pack_slices(&self, vec_low: &[Ciphertext], vec_high: &[Ciphertext]) -> Vec<Ciphertext>;
    fn split_and_pack(&self, cts: &[Ciphertext], chunk_size: usize) -> Vec<Ciphertext>;
    fn expand_state_blocks(&self, state: &FheAesState) -> Vec<Ciphertext>;
    fn expand_lo_hi_batch(&self, radix_cts: &[RadixCiphertext]) -> Vec<Ciphertext>;
    fn duplicate_radix4_packed_chunks(&self, chunks: &[Ciphertext]) -> Vec<Ciphertext>;
}

impl FheAesCiphertextUtils for BelfortServerKey {
    /// Packs two 2-bit ciphertexts into a single 4-bit ciphertext using radix-4 encoding.
    ///
    /// "ct_low" is treated as the least significant 2 bits, and "ct_high" is multiplied by 4
    /// (<< 2) to become the most significant 2 bits. The result is ct_low + (ct_high << 2).
    ///
    /// # Arguments
    /// - ct_low: "Ciphertext" block containing the least significant 2 bits to pack.
    /// - ct_high: "Ciphertext" block containing the most significant 2 bits to pack.
    ///
    /// # Returns
    /// - Packed "Ciphertext" block ct_low + (ct_high << 2).
    fn pack_radix4(&self, ct_low: &Ciphertext, ct_high: &Ciphertext) -> Ciphertext {
        let shortint_key = &self.pbs_key().key;

        let ct_high_shifted = shortint_key
            .unchecked_scalar_mul(ct_high, shortint_key.message_modulus.0.try_into().unwrap());

        shortint_key.unchecked_add(ct_low, &ct_high_shifted)
    }

    /// Packs the corresponding elements of two equally‑long vectors using "pack_radix4".
    ///
    /// Element i of "vec_low" is combined with element i of "vec_hi" resulting in
    /// "vec_low[i] + (vec_hi[i] << 2)". The two input vectors must be the same length.
    ///
    /// # Arguments
    /// - vec_low: "Vec<Ciphertext>" containing of the least significant 2 bits to pack.
    /// - vec_hi: "Vec<Ciphertext>" containing of the most significant 2 bits to pack.
    ///
    /// # Returns
    /// - "Vec<Ciphertext>" of packed elements.
    ///
    /// # Panics
    /// Panics if the input lengths are not equal.
    fn pack_slices(&self, vec_low: &[Ciphertext], vec_high: &[Ciphertext]) -> Vec<Ciphertext> {
        assert!(
            vec_low.len() == vec_high.len(),
            "Input lengths must be equal."
        );

        let mut packed = Vec::with_capacity(vec_high.len());

        for i in 0..packed.capacity() {
            packed.push(self.pack_radix4(&vec_low[i], &vec_high[i]));
        }

        packed
    }

    /// Wrapper that performs "self.split_chunks_even_odd" and immediately packs corresponding
    /// even/odd chunks via "pack_vecs".
    ///
    /// # Arguments
    /// - cts: "Vec<Ciphertext>" containing the elements to be processed.
    /// - chunk_size: Number of elements to be contained within a chunk.
    ///
    /// # Returns
    /// - "Vec<Ciphertext>" of packed elements.
    fn split_and_pack(&self, cts: &[Ciphertext], chunk_size: usize) -> Vec<Ciphertext> {
        let (even_chunks, odd_chunks) = split_chunks_even_odd(cts, chunk_size);

        self.pack_slices(&even_chunks, &odd_chunks)
    }

    /// Expands the 4 × 4 AES state into a vector of radix-4 ciphertexts.
    ///
    /// # Arguments
    /// - state: The 4x4 state array.
    ///
    /// # Returns
    /// - "Vec<Ciphertext>" of the flattened state.
    fn expand_state_blocks(&self, state: &FheAesState) -> Vec<Ciphertext> {
        let mut out = Vec::with_capacity(128);

        for ct in state.get_whole().iter().flat_map(|row| row.iter()) {
            let ct_blocks = ct.blocks();
            let bits_lo = self.pack_radix4(&ct_blocks[0], &ct_blocks[1]);
            let bits_hi = self.pack_radix4(&ct_blocks[2], &ct_blocks[3]);
            out.extend(repeat_each(&[bits_lo], 4));
            out.extend(repeat_each(&[bits_hi], 4));
        }

        out
    }

    fn expand_lo_hi_batch(&self, radix_cts: &[RadixCiphertext]) -> Vec<Ciphertext> {
        let mut expanded = Vec::with_capacity(radix_cts.len() * 8);
        for radix_ct in radix_cts {
            let blocks = radix_ct.blocks();

            let lo = self.pack_radix4(&blocks[0], &blocks[1]);
            let hi = self.pack_radix4(&blocks[2], &blocks[3]);

            expanded.extend(repeat_n(lo.clone(), 4));
            expanded.extend(repeat_n(hi.clone(), 4));
        }
        expanded
    }

    /// Expands an array of ciphertexts by grouping them into chunks of 4, packing the
    /// first two and last two ciphertexts in each chunk using radix-4, then replicating
    /// each packed result 4 times.
    ///
    /// # Arguments
    /// - "chunks": "[Ciphertext]" to be grouped and expanded.
    ///
    /// # Returns
    /// "Vec<Ciphertext>" where each group of 4 ciphertexts has been expanded into 8 packed
    /// ciphertexts (4 copies of the low-packed and 4 of the high-packed).
    fn duplicate_radix4_packed_chunks(&self, chunks: &[Ciphertext]) -> Vec<Ciphertext> {
        let mut out = Vec::with_capacity(chunks.len() * 2);

        for chunk in chunks.chunks(4) {
            let bits_lo = self.pack_radix4(&chunk[0], &chunk[1]);
            let bits_hi = self.pack_radix4(&chunk[2], &chunk[3]);
            out.extend(repeat_n(bits_lo, 4));
            out.extend(repeat_n(bits_hi, 4));
        }

        out
    }
}

/// Splits an array of ciphertexts into multiple arrays according
/// to given sizes.
///
/// This function takes an array of ciphertexts and a list of sizes,
/// and splits the input into consecutive non-overlapping arrays of
/// the specified sizes.
///
/// # Arguments
/// - cts: The input array of ciphertexts to be split.
/// - sizes: An array of sizes that indicate how many ciphertexts should go into each output array.
///
/// # Returns
/// "Vec<&'a [Ciphertext]>", where each array within the vector corresponds
///  to a subrange of "cts" defined by "sizes".
///
/// # Panics
/// Panics if the sum of `sizes` exceeds the length of `cts`.
pub fn split_by_chunk_sizes(cts: &[Ciphertext], sizes: &[usize]) -> Vec<Vec<Ciphertext>> {
    let mut result = Vec::with_capacity(sizes.len());
    let mut start = 0;

    for &size in sizes {
        let end = start + size;
        assert!(end <= cts.len(), "Sum of sizes exceeds input length");
        result.push(cts[start..end].to_vec());
        start = end;
    }

    result
}

/// Interleaves two ciphertext slices in 2-element chunks.
///
/// Given two slices "a" and "b", this function takes 2-element chunks from each, and
/// interleaves their elements in order: [a0, a1, b0, b1, a2, a3, b2, b3, ...].
///
/// # Arguments
/// - a: First array of ciphertexts.
/// - b: Second array of ciphertexts.
///
/// # Returns
/// "Vec<Ciphertext>" with interleaved values from "a" and "b".
///
/// # Panics
/// Panics if the input lengths are not equal.
pub fn interleave_chunks_of_two(a: &[Ciphertext], b: &[Ciphertext]) -> Vec<Ciphertext> {
    assert!(a.len() == b.len(), "Input lengths must be equal.");

    a.chunks(2)
        .zip(b.chunks(2))
        .flat_map(|(ct_low, ct_high)| ct_low.iter().chain(ct_high.iter()))
        .cloned()
        .collect()
}

/// Takes an input slice and returns a Vec in which each chunk of length "chunks_size"
/// is repeated "times" times in a row.
///
/// # Type Parameters
/// * "T": The element type which must implement "Clone".
///
/// # Arguments
/// * data: The input slice to be chunked.
/// * chunk_size: The size of each chunk to split data into.
/// * times: How many times to repeat each chunk.
///
/// # Returns
///  "Vec<T>" where eeach chunk of size "chunk_size" is repeated "times" times.
///
/// # Panics
/// Panics if `data.len()` is not a multiple of `chunks_size`.
pub fn repeat_chunks_n_times<T: Clone>(data: &[T], chunks_size: usize, times: usize) -> Vec<T> {
    assert!(
        data.len().is_multiple_of(chunks_size),
        "input length ({}) must be a multiple of {}",
        data.len(),
        chunks_size
    );

    data.chunks(chunks_size)
        .flat_map(|chunk| repeat_n(chunk, times).flatten().cloned())
        .collect()
}

/// Repeats each element in the input array "times" times.
///
/// # Type Parameters
/// * "T": The element type which must implement "Clone".
///
/// # Arguments
/// * vals: The array of values to be duplicated.
/// * times: How many times to repeat each element.
///
/// # Returns
///  "Vec<T>" where each element of "vals" is repeated "times" times.
pub fn repeat_each<T: Clone>(vals: &[T], times: usize) -> Vec<T> {
    vals.iter()
        .flat_map(|x| repeat_n(x, times))
        .cloned()
        .collect()
}

pub fn extend_in_order(
    dst: &mut Vec<Ciphertext>,
    src: &[Vec<Vec<Vec<Ciphertext>>>],
    row: usize,
    col: usize,
    order: &[usize],
) {
    for &i in order {
        dst.extend(src[row][col][i].clone());
    }
}

/// Splits a ciphertext array into two vectors by alternating chunks.
///
/// The input array is split into chunks of size "chunk_size".
/// - Even-indexed chunks (0, 2, 4, ...) are appended to "even_chunks".
/// - Odd-indexed chunks (1, 3, 5, ...) are appended to "odd_chunks".
///
/// # Arguments
/// - cts: "[Ciphertext]" to be split.
/// - chunk_size: Size of each group/chunk.
///
/// # Returns
/// "(Vec<Ciphertext>, Vec<Ciphertext>)", a tuple "(even_chunks, odd_chunks)"
/// containing the split ciphertexts.
pub fn split_chunks_even_odd(
    cts: &[Ciphertext],
    chunk_size: usize,
) -> (Vec<Ciphertext>, Vec<Ciphertext>) {
    let num_chunks = cts.len().div_ceil(chunk_size);
    let estimated_half = num_chunks.div_ceil(2) * chunk_size;

    let mut even_chunks = Vec::with_capacity(estimated_half);
    let mut odd_chunks = Vec::with_capacity(estimated_half);

    for (idx, chunk) in cts.chunks(chunk_size).enumerate() {
        if idx % 2 == 0 {
            even_chunks.extend_from_slice(chunk);
        } else {
            odd_chunks.extend_from_slice(chunk);
        }
    }

    (even_chunks, odd_chunks)
}

/// Repeats a vector of lookup vectors in a cyclic pattern to reach a desired total length.
///
/// # Arguments
/// - base: The list of "LookupVector"s to repeat cyclically.
/// - repeat_count" How many full cycles of "base" to produce.
///
/// # Returns
/// A "Vec<LookupVector>" of length "base.len() * repeat_count".
pub fn repeat_luts_cycled(base: &[LookupVector], repeat_count: usize) -> Vec<&LookupVector> {
    let base_len = base.len();

    base.iter()
        .cycle()
        .take(base_len * repeat_count)
        .collect::<Vec<_>>()
}

/// Zips two ciphertext slices element-wise and interleaves them.
///
/// For each pair "(low[i], high[i])", the result will contain "low[i]" followed by "high[i]".
/// That is, given:
/// - low = [l0, l1]
/// - high = [h0, h1] The output will be: [l0, h0, l1, h1]
///
/// # Arguments
/// - low: "[Ciphertext]" of "low" ciphertexts.
/// - high: "[Ciphertext]" of "high" ciphertexts. Must be the same length as "low".
///
/// # Returns
/// "Vec<Ciphertext>" interleaving elements from "low" and "high".
///
/// # Panics
/// Panics if the input lengths are not equal.
pub fn interleave_low_high(low: &[Ciphertext], high: &[Ciphertext]) -> Vec<Ciphertext> {
    assert!(low.len() == high.len(), "Input lengths must be equal.");

    low.iter()
        .zip(high.iter())
        .flat_map(|(low, high)| vec![low.clone(), high.clone()])
        .collect()
}
