/*
* MIT License
*
* Copyright (c) 2025 KU Leuven - COSIC
* Author: Wouter Legiest
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:

* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software.

* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/

use std::mem;

use crate::data::*;
use crate::enc_struct::EncStruct;
use crate::util::{
    self, apply_lookup_table_packed, unchecked_add_packed, unchecked_add_packed_assign,
    unchecked_scalar_add_packed, unchecked_scalar_add_packed_assign, unchecked_scalar_mul_packed,
    unchecked_scalar_mul_packed_assign, unchecked_sub_packed,
};
use std::collections::HashMap;
use std::time::Instant;

use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::shortint::prelude::*;

use pad::PadStr;

fn levenshtein_plain(x: &str, y: &str) -> Vec<u32> {
    let xlen = x.len();
    let ylen = y.len();

    let str1 = x.bytes().collect::<Vec<u8>>();
    let str2 = y.bytes().collect::<Vec<u8>>();

    let vec_size = std::cmp::max(xlen + 1, ylen + 1);
    let mut current: Vec<u32> = Vec::with_capacity(vec_size);
    let mut prev: Vec<u32> = Vec::with_capacity(vec_size);

    for i in 0..vec_size {
        current.push(0u32);
        prev.push(i as u32);
    }

    for j in 0..ylen {
        current[0] = (j + 1) as u32;

        for i in 0..xlen {
            let ins = current[i] + 1;
            let dlt = prev[i + 1] + 1;
            let mut sub = prev[i];
            if str1[i] != str2[j] {
                sub += 1;
            }

            current[i + 1] = std::cmp::min(std::cmp::min(dlt, ins), sub);
        }

        mem::swap(&mut current, &mut prev);
    }
    prev
    // anwser sits in previous[vec_size]
}

fn levenshtein_plain_matrix(x: &str, y: &str) -> u32 {
    assert_eq!(x.len(), y.len());

    let xlen = x.len();

    let str1 = x.bytes().collect::<Vec<u8>>();
    let str2 = y.bytes().collect::<Vec<u8>>();

    let ins_cost: u32 = 1;
    let del_cost: u32 = 1;
    let sub_cost: u32 = 1;

    // Initialise the D matrix and fill the first row and column
    let vec_size = xlen + 1;

    let mut d_matrix: Vec<Vec<u32>> = Vec::with_capacity(vec_size);

    for _ in 0..vec_size {
        let mut vec: Vec<u32> = Vec::with_capacity(vec_size);
        for _ in 0..vec_size {
            vec.push(0u32);
        }
        d_matrix.push(vec);
    }

    d_matrix[0][0] = 0u32;

    for i in 1..vec_size {
        d_matrix[0][i] = i as u32 * del_cost;
        d_matrix[i][0] = i as u32 * ins_cost;
    }

    for i in 1..vec_size {
        for j in 1..vec_size {
            let dlt = d_matrix[i - 1][j] + del_cost;
            let ins = d_matrix[i][j - 1] + ins_cost;
            let mut sub: u32 = d_matrix[i - 1][j - 1];

            if str1[i - 1] != str2[j - 1] {
                sub += sub_cost;
            }

            d_matrix[i][j] = std::cmp::min(std::cmp::min(dlt, ins), sub);
        }
    }
    d_matrix[xlen][xlen]
}

pub fn print_matrix<T: std::fmt::Display>(matrix: &[Vec<T>], name: &str) {
    println!("Matrix: {name}");
    // Find the maximum width of each column
    let mut col_widths = vec![0; matrix[0].len()];
    for row in matrix {
        for (col, item) in row.iter().enumerate() {
            col_widths[col] = col_widths[col].max(format!("{}", item).len());
        }
    }

    // Print the matrix with separators
    for row in matrix {
        for (col, item) in row.iter().enumerate() {
            let padding = col_widths[col] - format!("{}", item).len();
            print!("{:>width$} |", item, width = padding);
        }
        println!();
    }
}

/// Plain Processing
pub fn process_plain_query_enc_db(enc_struct: &mut EncStruct) {
    // Get the min and max lenght of the db strings
    let qlen = enc_struct.query.len() + 1;

    // DB processing and encrypting
    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(enc_struct.db_size);

    for i in 0..enc_struct.db_size {
        db_len.insert(i, NAME_LIST[i].len());
    }

    let db_max_size = db_len.values().max().unwrap();

    // Max factor is defined as the size of the D matrix! (db_max_size + 1)
    let mut max_factor = std::cmp::max(*db_max_size, qlen - 1);
    max_factor += 1;

    let th: usize = ((max_factor as f64) / 2.0).ceil() as usize;

    let zero_enc = enc_struct.cks.encrypt(0u64);

    // Build and fill all the h_matrices
    let mut h_matrices: Vec<Vec<Vec<Ciphertext>>> = Vec::with_capacity(enc_struct.db_size);
    let mut v_matrices: Vec<Vec<Vec<Ciphertext>>> = Vec::with_capacity(enc_struct.db_size);

    for _ in 0..enc_struct.db_size {
        let mut h_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(max_factor);
        let mut v_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(max_factor);

        for _ in 0..max_factor {
            let mut vec: Vec<Ciphertext> = Vec::with_capacity(max_factor);
            for _ in 0..max_factor {
                vec.push(zero_enc.clone());
            }
            h_matrix.push(vec.clone());
            v_matrix.push(vec);
        }

        for i in 0..max_factor {
            v_matrix[i][0] = enc_struct.cks.encrypt(1u64);
        }
        for i in 0..max_factor {
            h_matrix[0][i] = enc_struct.cks.encrypt(1u64);
        }

        h_matrices.push(h_matrix);
        v_matrices.push(v_matrix);
    }

    let lut_min_vec_def = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_min = enc_struct
        .sks
        .generate_lookup_table_from_vector(&lut_min_vec_def);
    let lut_min_vec = vec![lut_min; enc_struct.db_size];

    let fpga_lut_single = LookupVector::new(lut_min_vec_def);
    let lut_min_vec_fpga = vec![fpga_lut_single; enc_struct.db_size];

    enc_struct.max_factor = max_factor;
    enc_struct.th = th;
    enc_struct.v_matrices = v_matrices;
    enc_struct.h_matrices = h_matrices;
    enc_struct.lut_min_vec_sw = lut_min_vec;
    enc_struct.lut_min_vec_fpga = lut_min_vec_fpga;

    enc_struct.time = Instant::now();
}

pub fn process_plain_part_i(index: usize, enc_struct: &mut EncStruct, fpga_enable: bool) {
    let query_padded = enc_struct.query.pad_to_width(enc_struct.max_factor - 1);

    for j in 1..enc_struct.max_factor {
        if usize::abs_diff(index, j) <= enc_struct.th {
            let eq: Vec<Ciphertext> = util::get_db_enc_vec(
                query_padded.chars().nth(index - 1).unwrap(),
                j - 1,
                &enc_struct.db_enc_map,
            );

            let vin = util::extract_number_elements(&enc_struct.v_matrices, index, j - 1);
            let hin = util::extract_number_elements(&enc_struct.h_matrices, index - 1, j);

            let v1 = unchecked_scalar_add_packed(&enc_struct.sks, vin.iter().collect(), 1);
            let h1 = unchecked_scalar_add_packed(&enc_struct.sks, hin.iter().collect(), 1);

            let key1 = unchecked_scalar_mul_packed(&enc_struct.sks, h1.iter().collect(), 3);

            let key12 =
                unchecked_add_packed(&enc_struct.sks, key1.iter().collect(), eq.iter().collect());

            let key =
                unchecked_add_packed(&enc_struct.sks, key12.iter().collect(), v1.iter().collect());

            let mut ct_res: Vec<Ciphertext>;
            if fpga_enable {
                ct_res = key.clone();

                #[cfg(feature = "fpga")]
                {
                    let ref_lookupvector: [&LookupVector; LUT_SIZE] =
                        std::array::from_fn(|i| &enc_struct.lut_min_vec_fpga[i]);
                    enc_struct
                        .fpga_key
                        .apply_lookup_vector_packed_assign(&mut ct_res, &ref_lookupvector);
                }
            } else {
                ct_res = apply_lookup_table_packed(
                    &enc_struct.sks,
                    key.iter().collect(),
                    &enc_struct.lut_min_vec_sw,
                );
            }

            unchecked_scalar_add_packed_assign(&enc_struct.sks, &mut ct_res, 16);

            let v_res = unchecked_sub_packed(
                &enc_struct.sks,
                ct_res.iter().collect(),
                hin.iter().collect(),
            );
            let h_res = unchecked_sub_packed(
                &enc_struct.sks,
                ct_res.iter().collect(),
                vin.iter().collect(),
            );

            util::write_number_elements(&mut enc_struct.v_matrices, &v_res, index, j);
            util::write_number_elements(&mut enc_struct.h_matrices, &h_res, index, j);
        }
    }
}

/// Encrypted Processing
pub fn process_enc_query_enc_db(enc_struct: &mut EncStruct) {
    // Get the min and max lenght of the db strings
    let qlen = enc_struct.query.len() + 1;

    // DB processing and encrypting
    // let enc_struct.db_size: usize = data::NAME_LIST.len();
    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(enc_struct.db_size);

    for i in 0..enc_struct.db_size {
        db_len.insert(i, NAME_LIST[i].len());
    }

    let db_max_size = *db_len.values().max().unwrap();

    // Max factor is defined as the size of the D matrix! (db_max_size + 1)
    let mut max_factor = std::cmp::max(db_max_size, qlen - 1);
    max_factor += 1;

    let th = ((max_factor as f64) / 2.0).ceil() as usize;

    let query_padded = enc_struct.query.pad_to_width(max_factor - 1);

    let scale_factor: u8 = 0; // You can put it to 64

    let q_enc = query_padded
        .bytes() // convert char to int
        .map(|c| enc_struct.cks.encrypt((c - scale_factor) as u64))
        .collect::<Vec<Ciphertext>>();

    let q2_enc = query_padded
        .bytes() // convert char to int
        .map(|c| enc_struct.cks.encrypt(((c - scale_factor) >> 4) as u64))
        .collect::<Vec<Ciphertext>>();

    let zero_enc = enc_struct.cks.encrypt(0u64);
    let one_enc = enc_struct.cks.encrypt(1u64);

    let mut db_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(enc_struct.db_size);
    let mut db1_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(enc_struct.db_size);

    for i in 0..enc_struct.db_size {
        let padded_name = NAME_LIST[i].pad_to_width(max_factor - 1);

        let name_enc = padded_name
            .bytes() // convert char to int
            .map(|c| enc_struct.cks.encrypt((c - scale_factor) as u64)) // Encrypts
            .collect::<Vec<Ciphertext>>();

        let name1_enc = padded_name
            .bytes() // convert char to int
            .map(|c| enc_struct.cks.encrypt(((c - scale_factor) >> 4) as u64))
            .collect::<Vec<Ciphertext>>();

        db_enc_matrix.push(name_enc);
        db1_enc_matrix.push(name1_enc);
    }

    let lut_min_vec_def = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_eq_vec_def = [9u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_1eq_vec_def = [1u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

    let lut_min = enc_struct
        .sks
        .generate_lookup_table_from_vector(&lut_min_vec_def);
    let lut_1eq = enc_struct
        .sks
        .generate_lookup_table_from_vector(&lut_1eq_vec_def);
    let lut_eq = enc_struct
        .sks
        .generate_lookup_table_from_vector(&lut_eq_vec_def);

    let lut_1eq_vec = vec![lut_1eq; enc_struct.db_size];
    let lut_eq_vec = vec![lut_eq; enc_struct.db_size];
    let lut_min_vec = vec![lut_min; enc_struct.db_size];

    let lut_1eq_fpga = LookupVector::new(lut_1eq_vec_def);
    let lut_eq_fpga = LookupVector::new(lut_eq_vec_def);
    let lut_min_fpga = LookupVector::new(lut_min_vec_def);

    let lut_1eq_vec_fpga = vec![lut_1eq_fpga; enc_struct.db_size];
    let lut_eq_vec_fpga = vec![lut_eq_fpga; enc_struct.db_size];
    let lut_min_vec_fpga = vec![lut_min_fpga; enc_struct.db_size];

    // Build and fill all the h_matrices
    let mut h_matrices: Vec<Vec<Vec<Ciphertext>>> = Vec::with_capacity(enc_struct.db_size);
    let mut v_matrices: Vec<Vec<Vec<Ciphertext>>> = Vec::with_capacity(enc_struct.db_size);

    for _ in 0..enc_struct.db_size {
        let mut h_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(max_factor);
        let mut v_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(max_factor);

        for _ in 0..max_factor {
            let mut vec: Vec<Ciphertext> = Vec::with_capacity(max_factor);
            for _ in 0..max_factor {
                vec.push(zero_enc.clone());
            }
            h_matrix.push(vec.clone());
            v_matrix.push(vec.clone());
        }

        for i in 0..max_factor {
            v_matrix[i][0] = enc_struct.cks.encrypt(1u64);
        }
        for i in 0..max_factor {
            h_matrix[0][i] = enc_struct.cks.encrypt(1u64);
        }

        h_matrices.push(h_matrix);
        v_matrices.push(v_matrix);
    }

    let one_enc_vec = vec![one_enc.clone(); enc_struct.db_size];

    enc_struct.max_factor = max_factor;
    enc_struct.th = th;
    enc_struct.q_enc = q_enc;
    enc_struct.q2_enc = q2_enc;
    enc_struct.db_enc_matrix = db_enc_matrix;
    enc_struct.db1_enc_matrix = db1_enc_matrix;
    enc_struct.one_enc_vec = one_enc_vec;
    enc_struct.v_matrices = v_matrices;
    enc_struct.h_matrices = h_matrices;
    enc_struct.lut_1eq_vec_sw = lut_1eq_vec;
    enc_struct.lut_eq_vec_sw = lut_eq_vec;
    enc_struct.lut_min_vec_sw = lut_min_vec;
    enc_struct.lut_1eq_vec_fpga = lut_1eq_vec_fpga;
    enc_struct.lut_eq_vec_fpga = lut_eq_vec_fpga;
    enc_struct.lut_min_vec_fpga = lut_min_vec_fpga;

    enc_struct.time = Instant::now();
}

pub fn process_part_i(index: usize, enc_struct: &mut EncStruct, fpga_enable: bool) {
    let q1_vec: Vec<tfhe::shortint::prelude::Ciphertext> =
        vec![enc_struct.q_enc[index - 1].clone(); enc_struct.db_size];
    let q2_vec: Vec<tfhe::shortint::prelude::Ciphertext> =
        vec![enc_struct.q2_enc[index - 1].clone(); enc_struct.db_size];

    let one_enc_vec_ref: Vec<&Ciphertext> = enc_struct.one_enc_vec.iter().collect();

    for j in 1..enc_struct.max_factor {
        if usize::abs_diff(index, j) <= enc_struct.th {
            // Check the first part of the character
            let mut eq1 = unchecked_sub_packed(
                &enc_struct.sks,
                q1_vec.iter().collect(),
                util::get_column(&enc_struct.db_enc_matrix, j - 1)
                    .iter()
                    .collect(),
            );

            unchecked_scalar_add_packed_assign(&enc_struct.sks, &mut eq1, 16);

            let mut eq1_lut = Vec::new();

            if fpga_enable {
                eq1_lut = eq1.clone();
                #[cfg(feature = "fpga")]
                {
                    let ref_lookupvector: [&LookupVector; LUT_SIZE] =
                        std::array::from_fn(|i| &enc_struct.lut_1eq_vec_fpga[i]);
                    enc_struct
                        .fpga_key
                        .apply_lookup_vector_packed_assign(&mut eq1_lut, &ref_lookupvector);
                }
            } else {
                let ct = apply_lookup_table_packed(
                    &enc_struct.sks,
                    eq1.iter().collect(),
                    &enc_struct.lut_1eq_vec_sw,
                );
                eq1_lut.extend(ct);
            }

            let eq1_ref: Vec<&Ciphertext> = eq1_lut.iter().collect();

            eq1 = unchecked_sub_packed(&enc_struct.sks, one_enc_vec_ref.clone(), eq1_ref);

            unchecked_scalar_add_packed_assign(&enc_struct.sks, &mut eq1, 16);

            let mut eq2 = unchecked_sub_packed(
                &enc_struct.sks,
                q2_vec.iter().collect(),
                util::get_column(&enc_struct.db1_enc_matrix, j - 1)
                    .iter()
                    .collect(),
            );

            unchecked_scalar_mul_packed_assign(&enc_struct.sks, &mut eq2, 2);

            unchecked_add_packed_assign(&enc_struct.sks, &mut eq2, eq1.iter().collect());

            let mut eq2_lut = Vec::new();

            if fpga_enable {
                eq2_lut = eq2.clone();
                #[cfg(feature = "fpga")]
                {
                    let ref_lookupvector: [&LookupVector; LUT_SIZE] =
                        std::array::from_fn(|i| &enc_struct.lut_eq_vec_fpga[i]);
                    enc_struct
                        .fpga_key
                        .apply_lookup_vector_packed_assign(&mut eq2_lut, &ref_lookupvector);
                }
            } else {
                let ct = apply_lookup_table_packed(
                    &enc_struct.sks,
                    eq2.iter().collect(),
                    &enc_struct.lut_eq_vec_sw,
                );
                eq2_lut.extend(ct);
            }

            let vin = util::extract_number_elements(&enc_struct.v_matrices, index, j - 1);
            let hin = util::extract_number_elements(&enc_struct.h_matrices, index - 1, j);

            let v1 = unchecked_scalar_add_packed(&enc_struct.sks, vin.iter().collect(), 1);
            let h1 = unchecked_scalar_add_packed(&enc_struct.sks, hin.iter().collect(), 1);

            let key1 = unchecked_scalar_mul_packed(&enc_struct.sks, h1.iter().collect(), 3);

            let key12 = unchecked_add_packed(
                &enc_struct.sks,
                key1.iter().collect(),
                eq2_lut.iter().collect(),
            );

            let key =
                unchecked_add_packed(&enc_struct.sks, key12.iter().collect(), v1.iter().collect());

            let mut ct_res = Vec::new();

            if fpga_enable {
                ct_res = key.clone();

                #[cfg(feature = "fpga")]
                {
                    let ref_lookupvector: [&LookupVector; LUT_SIZE] =
                        std::array::from_fn(|i| &enc_struct.lut_min_vec_fpga[i]);
                    enc_struct
                        .fpga_key
                        .apply_lookup_vector_packed_assign(&mut ct_res, &ref_lookupvector);
                }
            } else {
                let ct = apply_lookup_table_packed(
                    &enc_struct.sks,
                    key.iter().collect(),
                    &enc_struct.lut_min_vec_sw,
                );
                ct_res.extend(ct);
            }

            unchecked_scalar_add_packed_assign(&enc_struct.sks, &mut ct_res, 16);

            let v_res = unchecked_sub_packed(
                &enc_struct.sks,
                ct_res.iter().collect(),
                hin.iter().collect(),
            );
            let h_res = unchecked_sub_packed(
                &enc_struct.sks,
                ct_res.iter().collect(),
                vin.iter().collect(),
            );

            util::write_number_elements(&mut enc_struct.v_matrices, &v_res, index, j);
            util::write_number_elements(&mut enc_struct.h_matrices, &h_res, index, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::time::Instant;

    /// Builds an `EncStruct` with the same parameter set used by the application.
    fn setup_enc_struct() -> EncStruct {
        let mut params = tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS.clone();
        params.message_modulus = MessageModulus(16);
        params.carry_modulus = CarryModulus(1);
        let cks: ClientKey = ClientKey::new(params);
        let db_max_size = NAME_LIST.iter().map(|s| s.len()).max().unwrap_or(0);
        let max_factor = std::cmp::max(db_max_size, 25) + 1;
        println!("Start DB Processing");
        let db_processed = process_db(&cks, max_factor);
        println!("DB Processed!");
        EncStruct::new(max_factor, db_processed, cks)
    }

    /// 1. Plaintext – no FHE, pure Levenshtein on unencrypted strings.
    #[test]
    fn test_unencrypted_levenshtein() {
        let x = "Bilba Baggins";

        // Direct distance to the expected match must be exactly 1 ('a' → 'o').
        let dist = levenshtein_plain(x, "Bilbo Baggins");
        assert_eq!(dist[x.len()], 1);

        // Best match across the full NAME_LIST must be "Bilbo Baggins".
        let best = NAME_LIST
            .iter()
            .min_by_key(|&&name| levenshtein_plain(x, name)[x.len()])
            .copied()
            .unwrap();
        assert_eq!(best, "Bilbo Baggins");
    }

    /// 2. CPU – plain query, encrypted database, FHE evaluated on the CPU.
    // #[test]
    fn test_cpu_levenshtein() {
        let query = "Bilba Baggins";
        let mut enc_struct = setup_enc_struct();
        enc_struct.input = format!("p:{query}");
        enc_struct.query = query.to_string();

        let start = Instant::now();
        process_plain_query_enc_db(&mut enc_struct);
        println!("Start Plain Processing took: {:?}", start.elapsed());
        let max_factor = enc_struct.max_factor;
        for i in 1..max_factor {
            let start = Instant::now();
            process_plain_part_i(i, &mut enc_struct, false);
            println!("PProcess : {i}/{max_factor} took: {:?}", start.elapsed());
        }

        let mut app = App::new();
        let start = Instant::now();
        app.post_process(&mut enc_struct, false);
        println!("STart Post Processing took: {:?}", start.elapsed());
        assert_eq!(app.messages[0].1, "Bilbo Baggins");
    }

    /// 3. FPGA – plain query, encrypted database, FHE evaluated on the FPGA.
    #[test]
    #[cfg(feature = "fpga")]
    fn test_fpga_levenshtein() {
        let query = "Bilba Baggins";
        let mut enc_struct = setup_enc_struct();
        enc_struct.input = format!("p:{query}");
        enc_struct.query = query.to_string();

        let start = Instant::now();
        process_plain_query_enc_db(&mut enc_struct);
        println!("Plain processing took: {:?}", start.elapsed());
        let max_factor = enc_struct.max_factor;
        for i in 1..max_factor {
            let start = Instant::now();
            process_plain_part_i(i, &mut enc_struct, true);
            println!(
                "Plain part {i}/{max_factor} processing took: {:?}",
                start.elapsed()
            );
        }

        let mut app = App::new();
        let start = Instant::now();
        app.post_process(&mut enc_struct, true);
        println!("Post processing took: {:?}", start.elapsed());
        assert_eq!(app.messages[0].1, "Bilbo Baggins");
    }
}
