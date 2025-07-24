use std::mem;

use crate::data;

pub fn levenshtein_plain(x: &str, y: &str) -> Vec<u32> {
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

pub fn levenshtein_plain_matrix(x: &str, y: &str) -> u32 {
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

use std::iter::zip;
use std::sync::mpsc;
use std::time::Instant;
use std::{collections::HashMap, f64::consts::E};

use tfhe::core_crypto::fpga::keyswitch_bootstrap::KeyswitchBootstrapPacked;
use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::core_crypto::fpga::BelfortFpgaLuts;
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::shortint::prelude::*;

use pad::PadStr;
use tfhe::shortint::server_key::LookupTable;

pub fn get_column(data: &Vec<Vec<Ciphertext>>, index: usize) -> Vec<Ciphertext> {
    let mut result = Vec::new();
    for row in data {
        if let Some(element) = row.get(index) {
            result.push(element.clone()); // Dereference to own the element
        }
    }
    result
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
        // Check if the matrix has at least one element (to avoid indexing errors)
        if !matrix.is_empty() && !matrix[0].is_empty() {
            zero_zero_elements.push(matrix[x][y].clone()); // Clone to avoid ownership issues
        }
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

pub fn process_enc_query_enc_db(
    query: &str,
    cks: &ClientKey,
    sks: &ServerKey,
    fpga: &mut BelfortServerKey,
    db_size: usize,
    print: bool,
) -> HashMap<usize, i64> {
    const DEBUG: bool = false;
    let index = 0;

    // Generate keys and encrypt query

    // Get the min and max lenght of the db strings
    let qlen = query.len() + 1;

    // DB processing and encrypting
    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(db_size);

    for i in 0..db_size {
        db_len.insert(i, data::NAME_LIST[i].len());
    }

    let _db_min_size = *db_len.values().into_iter().min().unwrap();
    let db_max_size = *db_len.values().into_iter().max().unwrap();

    // Max factor is defined as the size of the D matrix! (db_max_size + 1)
    let mut max_factor = std::cmp::max(db_max_size, qlen - 1);
    max_factor += 1;

    let th = ((max_factor as f64) / 2.0).ceil() as usize;

    if print {
        println!("Max factor: {max_factor} \t th: {th}");
    }

    let query_padded = query.pad_to_width(max_factor - 1);

    let scale_factor: u8 = 0; // You can put it to 64

    let q_enc = query_padded
        .bytes() // convert char to int
        .map(|c| cks.encrypt((c - scale_factor) as u64)) // Encrypts
        .collect::<Vec<tfhe::shortint::Ciphertext>>();

    let q2_enc = query_padded
        .bytes() // convert char to int
        .map(|c| cks.encrypt(((c - scale_factor) >> 4) as u64))
        .collect::<Vec<tfhe::shortint::Ciphertext>>();

    let zero_enc = cks.encrypt(0u64);
    let one_enc = cks.encrypt(1u64);

    let mut db_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(db_size);
    let mut db1_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(db_size);

    for i in 0..db_size {
        let padded_name = data::NAME_LIST[i].pad_to_width(max_factor - 1);

        let name_enc = padded_name
            .bytes() // convert char to int
            .map(|c| cks.encrypt((c - scale_factor) as u64)) // Encrypts
            .collect::<Vec<tfhe::shortint::Ciphertext>>();

        let name1_enc = padded_name
            .bytes() // convert char to int
            .map(|c| cks.encrypt(((c - scale_factor) >> 4) as u64))
            .collect::<Vec<tfhe::shortint::Ciphertext>>();

        db_enc_matrix.push(name_enc);
        db1_enc_matrix.push(name1_enc);
    }

    // Create LUTs
    let lut_min_vec_def = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_eq_vec_def = [9u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_1eq_vec_def = [1u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

    let lut_1eq_fpga = LookupVector::new(&lut_1eq_vec_def);
    let lut_eq_fpga = LookupVector::new(&lut_eq_vec_def);
    let lut_min_fpga = LookupVector::new(&lut_min_vec_def);

    let lut_1eq_vec_fpga = vec![lut_1eq_fpga; db_size];
    let lut_eq_vec_fpga = vec![lut_eq_fpga; db_size];
    let lut_min_vec_fpga = vec![lut_min_fpga; db_size];

    // Build and fill all the h_matrices
    let mut h_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> = Vec::with_capacity(db_size);
    let mut v_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> = Vec::with_capacity(db_size);

    for _ in 0..db_size {
        let mut h_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);
        let mut v_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);

        for _ in 0..max_factor {
            let mut vec: Vec<tfhe::shortint::Ciphertext> = Vec::with_capacity(max_factor);
            for _ in 0..max_factor {
                vec.push(zero_enc.clone());
            }
            h_matrix.push(vec.clone());
            v_matrix.push(vec.clone());
        }

        for i in 0..max_factor {
            v_matrix[i][0] = cks.encrypt(1u64);
        }
        for i in 0..max_factor {
            h_matrix[0][i] = cks.encrypt(1u64);
        }

        h_matrices.push(h_matrix);
        v_matrices.push(v_matrix);
    }

    let one_enc_vec = vec![one_enc.clone(); db_size];

    let one_enc_vec_ref: Vec<&Ciphertext> = one_enc_vec.iter().collect();

    for i in 1..max_factor {
        let q1_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q_enc[i - 1].clone(); db_size];
        let q2_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q2_enc[i - 1].clone(); db_size];

        if print {
            println!("{i}");
        }

        for j in 1..max_factor {
            if usize::abs_diff(i, j) <= th {
                let t = Instant::now();

                // Check the first part of the character
                let mut eq1 = unchecked_sub_packed(
                    &sks,
                    q1_vec.iter().collect(),
                    get_column(&db_enc_matrix, j - 1).iter().collect(),
                );

                sks.unchecked_scalar_add_packed_assign(&mut eq1, 16);

                let mut eq1_lut = Vec::new();

                eq1_lut = eq1.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut eq1_lut, &lut_1eq_vec_fpga);

                let eq1_ref: Vec<&Ciphertext> = eq1_lut.iter().collect(); // ?

                eq1 = unchecked_sub_packed(&sks, one_enc_vec_ref.clone(), eq1_ref);
                sks.unchecked_scalar_add_packed_assign(&mut eq1, 16);

                let mut eq2 = unchecked_sub_packed(
                    &sks,
                    q2_vec.iter().collect(),
                    get_column(&db1_enc_matrix, j - 1).iter().collect(),
                );

                sks.unchecked_scalar_mul_packed_assign(&mut eq2, 2);
                sks.unchecked_add_packed_assign(&mut eq2, eq1.iter().collect());

                let mut eq2_lut = Vec::new();

                eq2_lut = eq2.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut eq2_lut, &lut_eq_vec_fpga);

                let vin = extract_number_elements(&v_matrices, i, j - 1);
                let hin = extract_number_elements(&h_matrices, i - 1, j);

                let v1 = unchecked_scalar_add_packed(&sks, vin.iter().collect(), 1);
                let h1 = unchecked_scalar_add_packed(&sks, hin.iter().collect(), 1);

                let key1 = sks.unchecked_scalar_mul_packed(h1.iter().collect(), 3);
                let key12 =
                    unchecked_add_packed(&sks, key1.iter().collect(), eq2_lut.iter().collect());

                let key =
                    unchecked_add_packed(&sks, key12.iter().collect(), v1.iter().collect());

                let mut ct_res = Vec::new();

                ct_res = key.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut ct_res, &lut_min_vec_fpga);

                sks.unchecked_scalar_add_packed_assign(&mut ct_res, 16);

                let mut dbg_dec_vec: Vec<u64> = Vec::with_capacity(db_size);
                for i in 0..db_size {
                    let dec_tmp: u64 = cks.decrypt(&ct_res[i]);
                    dbg_dec_vec.push(dec_tmp);
                }

                let v_res =
                    unchecked_sub_packed(&sks, ct_res.iter().collect(), hin.iter().collect());
                let h_res =
                    unchecked_sub_packed(&sks, ct_res.iter().collect(), vin.iter().collect());

                write_number_elements(&mut v_matrices, &v_res, i, j);
                write_number_elements(&mut h_matrices, &h_res, i, j);

                if DEBUG {
                    let out = t.elapsed().as_secs_f64();
                    println!("Time: {out}");
                }
            }
        }
    }

    let mut h_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(db_size);
    let mut v_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(db_size);

    for k in 0..db_size {
        let mut h_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(max_factor);
        let mut v_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(max_factor);

        for i in 0..max_factor {
            let mut h_vec: Vec<i64> = Vec::with_capacity(max_factor);
            let mut v_vec: Vec<i64> = Vec::with_capacity(max_factor);

            for j in 0..max_factor {
                let h_dec: u64 = cks.decrypt(&h_matrices[k][i][j]);
                let v_dec: u64 = cks.decrypt(&v_matrices[k][i][j]);

                if h_dec > 8 {
                    h_vec.push((h_dec - 16) as i64);
                } else {
                    h_vec.push(h_dec as i64);
                }

                if v_dec > 8 {
                    v_vec.push((v_dec - 16) as i64);
                } else {
                    v_vec.push(v_dec as i64);
                }
            }

            h_dec_matrix.push(h_vec);
            v_dec_matrix.push(v_vec);
        }
        h_dec_matrices.push(h_dec_matrix);
        v_dec_matrices.push(v_dec_matrix);
    }

    if DEBUG {
        println!("-------------------");
        println!("{}\t {}", &query, &data::NAME_LIST[index]);
        print_matrix(&h_dec_matrices[index], "h_dec");
        print_matrix(&v_dec_matrices[index], "v_dec");
    }

    let mut result_map: HashMap<usize, i64> = HashMap::new();

    for k in 0..db_size {
        let mut diag_score = 0;

        let m = max_factor - 1;

        for i in 1..m + 1 {
            diag_score += &h_dec_matrices[k][i][i];
        }

        for i in 0..m {
            diag_score += &v_dec_matrices[k][i + 1][i];
        }

        result_map.insert(k, diag_score);
    }

    result_map
}
