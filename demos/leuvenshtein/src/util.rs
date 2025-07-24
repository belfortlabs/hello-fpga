use std::collections::HashMap;

use rayon::iter::*;

use rayon::iter::IntoParallelRefMutIterator;
use tfhe::shortint::prelude::*;
use tfhe::shortint::server_key::LookupTableOwned;

pub fn get_column(data: &Vec<Vec<Ciphertext>>, index: usize) -> Vec<Ciphertext> {
    let mut result = Vec::new();
    for row in data {
        if let Some(element) = row.get(index) {
            result.push(element.clone()); // Dereference to own the element
        }
    }
    result
}

pub fn get_db_enc_vec(
    ch: char,
    index: usize,
    db_processed: &HashMap<usize, HashMap<char, Vec<Ciphertext>>>,
) -> Vec<Ciphertext> {
    let mut vec = Vec::new();

    for i in 0..db_processed.len() {
        let value = db_processed.get(&i).unwrap();
        let char_vec = value.get(&ch).unwrap();

        vec.push(char_vec[index].clone());
    }

    vec.clone()
}

pub fn extract_number_elements(
    data: &Vec<Vec<Vec<Ciphertext>>>,
    x: usize,
    y: usize,
) -> Vec<Ciphertext> {
    let mut zero_zero_elements = Vec::new();
    for matrix in data {
        // Check if the matrix has at least one element to avoid indexing errors
        if matrix.is_empty() || matrix[0].is_empty() {
            continue;
        }
        zero_zero_elements.push(matrix[x][y].clone()); // Clone to avoid ownership issues
    }
    zero_zero_elements
}

pub fn write_number_elements(
    data: &mut Vec<Vec<Vec<Ciphertext>>>,
    input: &Vec<Ciphertext>,
    x: usize,
    y: usize,
) {
    for i in 0..input.len() {
        data[i][x][y] = input[i].clone();
    }
}

pub fn apply_lookup_table_packed(
    sk: &tfhe::shortint::ServerKey,
    cts: Vec<&Ciphertext>,
    accs: &[LookupTableOwned],
) -> Vec<Ciphertext> {
    let mut ct_res: Vec<Ciphertext> = cts.iter().map(|&ct| ct.clone()).collect();

    ct_res
        .par_iter_mut()
        .zip(accs.par_iter())
        .for_each(|(ct, acc)| {
            sk.apply_lookup_table_assign(ct, acc);
        });

    ct_res
}

pub fn unchecked_add_packed_assign(
    sk: &tfhe::shortint::ServerKey,
    cts_left: &mut Vec<Ciphertext>,
    cts_right: Vec<&Ciphertext>,
) {
    cts_left
        .par_iter_mut()
        .zip(cts_right.par_iter())
        .for_each(|(ct_left, ct_right)| {
            sk.unchecked_add_assign(ct_left, ct_right);
        });
}

pub fn unchecked_add_packed(
    sk: &tfhe::shortint::ServerKey,
    cts_left: Vec<&Ciphertext>,
    cts_right: Vec<&Ciphertext>,
) -> Vec<Ciphertext> {
    let mut results: Vec<Ciphertext> = cts_left.into_iter().cloned().collect();
    unchecked_add_packed_assign(sk, &mut results, cts_right);

    results
}

pub fn unchecked_scalar_add_packed_assign(
    sk: &tfhe::shortint::ServerKey,
    cts: &mut Vec<Ciphertext>,
    scalar: u8,
) {
    cts.par_iter_mut()
        .for_each(|ct| sk.unchecked_scalar_add_assign(ct, scalar));
}

pub fn unchecked_scalar_add_packed(
    sk: &tfhe::shortint::ServerKey,
    cts: Vec<&Ciphertext>,
    scalar: u8,
) -> Vec<Ciphertext> {
    let mut cts_result = cts.into_iter().cloned().collect();
    unchecked_scalar_add_packed_assign(sk, &mut cts_result, scalar);
    cts_result
}

pub fn unchecked_scalar_mul_packed_assign(
    sk: &tfhe::shortint::ServerKey,
    cts: &mut Vec<Ciphertext>,
    scalar: u8,
) {
    cts.par_iter_mut()
        .for_each(|ct| sk.unchecked_scalar_mul_assign(ct, scalar));
}

pub fn unchecked_scalar_mul_packed(
    sk: &tfhe::shortint::ServerKey,
    cts: Vec<&Ciphertext>,
    scalar: u8,
) -> Vec<Ciphertext> {
    let mut results: Vec<Ciphertext> = cts.into_iter().cloned().collect();
    unchecked_scalar_mul_packed_assign(sk, &mut results, scalar);

    results
}

pub fn unchecked_sub_packed_assign(
    sk: &tfhe::shortint::ServerKey,
    cts_left: &mut Vec<Ciphertext>,
    cts_right: Vec<&Ciphertext>,
) {
    cts_left
        .par_iter_mut()
        .zip(cts_right.par_iter())
        .for_each(|(ct_left, ct_right)| {
            sk.unchecked_sub_assign_with_correcting_term(ct_left, ct_right);
        });
}

pub fn unchecked_sub_packed(
    sk: &tfhe::shortint::ServerKey,
    cts_left: Vec<&Ciphertext>,
    cts_right: Vec<&Ciphertext>,
) -> Vec<Ciphertext> {
    let mut results: Vec<Ciphertext> = cts_left.into_iter().cloned().collect();
    unchecked_sub_packed_assign(sk, &mut results, cts_right);

    results
}
