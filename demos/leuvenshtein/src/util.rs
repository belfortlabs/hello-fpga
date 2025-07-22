use std::collections::HashMap;

use tfhe::shortint::prelude::*;

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
