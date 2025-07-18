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

// pub fn process_plain_query_enc_db(
//     query: &str,
//     cks: &ClientKey,
//     sks: &ServerKey,
//     fpga: &mut BelfortFpgaUtils,
//     db_size: usize,
//     print: bool,
// ) -> HashMap<usize, i64> {
//     const DEBUG: bool = false;
//     let index = 0;

//     // Generate keys and encrypt query
//     // let (cks, sks) = gen_keys(PARAM_MESSAGE_4_CARRY_0_KS_PBS);

//     // Get the min and max lenght of the db strings
//     let qlen = query.len() + 1;

//     // DB processing and encrypting
//     // let db_size: usize = data::NAME_LIST.len();
//     let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(db_size);

//     // Size adadaptation
//     // let db_size = 32;

//     for i in 0..db_size {
//         db_len.insert(i, data::NAME_LIST[i].len());
//     }

//     let _db_min_size = *db_len.values().into_iter().min().unwrap();
//     let db_max_size = *db_len.values().into_iter().max().unwrap();

//     // Max factor is defined as the size of the D matrix! (db_max_size + 1)
//     let mut max_factor = std::cmp::max(db_max_size, qlen - 1);
//     max_factor += 1;

//     let th = ((max_factor as f64) / 2.0).ceil() as usize;
//     // let th = 8;

//     if print {
//         println!("Max factor: {max_factor} \t th: {th}");
//     }

//     let query_padded = query.pad_to_width(max_factor - 1);

//     let scale_factor: u8 = 0; // You can put it to 64

//     // let q_enc = query_padded
//     //     .bytes() // convert char to int
//     //     .map(|c| cks.encrypt((c - scale_factor) as u64)) // Encrypts
//     //     .collect::<Vec<tfhe::shortint::Ciphertext>>();

//     // let q2_enc = query_padded
//     //     .bytes() // convert char to int
//     //     .map(|c| cks.encrypt(((c - scale_factor) >> 4) as u64))
//     //     .collect::<Vec<tfhe::shortint::Ciphertext>>();

//     let zero_enc = cks.encrypt(0u64);
//     let one_enc = cks.encrypt(1u64);

//     // ----------------- Preprocess the db -----------------------------
//     let mut db_processed: HashMap<usize, HashMap<char, Vec<Ciphertext>>> = HashMap::new();
//     let mut db_processed_plain: HashMap<usize, HashMap<char, Vec<u8>>> = HashMap::new();

//     // Preprossing all the ascii
//     // let mut ascii_collection = (97..122).collect::<Vec<u8>>(); // Lowercase letters
//     // ascii_collection.extend((48..57).collect::<Vec<u8>>()); // Numbers
//     // ascii_collection.extend((65..90).collect::<Vec<u8>>()); // Uppercase letters

//     let ascii_collection = (20..126).collect::<Vec<u8>>();

//     for k in 0..db_size{
//         let t = levenshtein::data::NAME_LIST[k].pad_to_width(max_factor - 1);
//         let m = t.len();
//         let mut peq = HashMap::new();
//         let mut peq_plain = HashMap::new();

//         for i in &ascii_collection {
//             let s = *i as u8 as char;
//             let mut bitvec = vec![0u8; m];

//             for j in 0..m {
//                 let pj = t.chars().nth(j).unwrap();
//                 if s == pj {
//                     bitvec[j] = 9;
//                 }
//             }

//             let vec_enc = bitvec
//             .iter()
//             .map(|c| cks.encrypt(*c as u64)) // Encrypts
//             .collect::<Vec<tfhe::shortint::Ciphertext>>();

//             peq_plain.insert(s, bitvec);
//             peq.insert(s, vec_enc);
//         }
//         db_processed.insert(k, peq);
//         db_processed_plain.insert(k, peq_plain);
//     }

//     // println!("{:?}", db_processed_plain.get(&0usize).unwrap());

//     // Create LUTs
//     let lut_min_vec = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();

//     let lut_min = sks.generate_lookup_table_vector(&lut_min_vec);

//     // Build and fill all the h_matrices
//     let mut h_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> = Vec::with_capacity(db_size);
//     let mut v_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> = Vec::with_capacity(db_size);

//     for _ in 0..db_size {
//         let mut h_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);
//         let mut v_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);

//         for _ in 0..max_factor {
//             let mut vec: Vec<tfhe::shortint::Ciphertext> = Vec::with_capacity(max_factor);
//             for _ in 0..max_factor {
//                 vec.push(zero_enc.clone());
//             }
//             h_matrix.push(vec.clone());
//             v_matrix.push(vec.clone());
//         }

//         for i in 0..max_factor {
//             v_matrix[i][0] = cks.encrypt(1u64);
//         }
//         for i in 0..max_factor {
//             h_matrix[0][i] = cks.encrypt(1u64);
//         }

//         h_matrices.push(h_matrix);
//         v_matrices.push(v_matrix);
//     }

//     let lut_min_vec = vec![lut_min; db_size];

//     for i in 1..max_factor {
//         // let q1_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q_enc[i - 1].clone(); db_size];
//         // let q2_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q2_enc[i - 1].clone(); db_size];

//         // let query1_vec = q1_vec.iter().collect();
//         // let query2_vec = q1_vec.iter().collect();

//         if print {
//             println!("{i}");
//         }

//         for j in 1..max_factor {
//             if usize::abs_diff(i, j) <= th {
//                 let t = Instant::now();

//                 let eq: Vec<Ciphertext> = get_db_enc_vec(query_padded.chars().nth(i - 1).unwrap(), j - 1, &db_processed);

//                 let vin = extract_number_elements(&v_matrices, i, j - 1);
//                 let hin = extract_number_elements(&h_matrices, i - 1, j);

//                 let v1 = sks.unchecked_scalar_add_packed(vin.iter().collect(), 1);
//                 let h1 = sks.unchecked_scalar_add_packed(hin.iter().collect(), 1);

//                 let key1 = sks.unchecked_scalar_mul_packed(h1.iter().collect(), 3);
//                 let key12 =
//                     sks.unchecked_add_packed(key1.iter().collect(), eq.iter().collect());

//                 let key = sks.unchecked_add_packed(key12.iter().collect(), v1.iter().collect());

//                 // sks.unchecked_scalar_add_packed_assign(&mut key, 16);

//                 // let mut ct_res: Vec<Ciphertext> = Vec::new();
//                 #[cfg(not(feature = "fpga"))]
//                 let mut ct_res =
//                     sks.apply_lookup_table_packed_parallellized(key.iter().collect(), &lut_min_vec);

//                 #[cfg(feature = "fpga")]
//                 let mut ct_res =
//                 {
//                     let mut ct_res = Vec::new();
//                     for keys in key.chunks(16){
//                         let ct_result = sks.apply_lookup_table_packed_fpga(
//                             keys.iter().collect(),
//                             vec![8usize; 16],
//                             fpga,
//                         );
//                         ct_res.extend(ct_result);
//                     }
//                     ct_res
//                 };

//                 // println!("LUT MIN {i} {j}");

//                 sks.unchecked_scalar_add_packed_assign(&mut ct_res, 16);

//                 let mut dbg_dec_vec: Vec<u64> = Vec::with_capacity(db_size);
//                 for i in 0..db_size {
//                     let dec_tmp: u64 = cks.decrypt(&eq[i]);
//                     dbg_dec_vec.push(dec_tmp);
//                 }

//                 let v_res = sks.unchecked_sub_packed(ct_res.iter().collect(), hin.iter().collect());
//                 let h_res = sks.unchecked_sub_packed(ct_res.iter().collect(), vin.iter().collect());

//                 write_number_elements(&mut v_matrices, &v_res, i, j);
//                 write_number_elements(&mut h_matrices, &h_res, i, j);

//                 if DEBUG {
//                     let out = t.elapsed().as_secs_f64();
//                     println!("Time: {out}");

//                     let eq2_dec: u64 = cks.decrypt(&eq[index]);

//                     let vin_dec: u64 = cks.decrypt(&vin[index]);
//                     let hin_dec: u64 = cks.decrypt(&hin[index]);

//                     let v1_dec: u64 = cks.decrypt(&v1[index]);
//                     let h1_dec: u64 = cks.decrypt(&h1[index]);

//                     let key1_dec: u64 = cks.decrypt(&key1[index]);

//                     let key12_dec: u64 = cks.decrypt(&key12[index]);
//                     let ct_dec: u64 = cks.decrypt(&ct_res[index]);
//                     let vout_dec: u64 = cks.decrypt(&v_res[index]);
//                     let hout_dec: u64 = cks.decrypt(&h_res[index]);

//                     let key_dec: u64 = cks.decrypt(&key[index]);

//                     println!("------------ {i}\t{j} ------------------");
//                     let padded_name = data::NAME_LIST[index].pad_to_width(max_factor - 1);

//                     println!(
//                         "Eq: {} {} -> {}",
//                         query_padded.chars().nth(i - 1).unwrap(),
//                         padded_name.chars().nth(j - 1).unwrap(),
//                         eq2_dec
//                     );
//                     println!("vin: {vin_dec}; hin: {hin_dec}");
//                     println!("key = 3 * HIN + 9 * eq + VIN");
//                     println!("3*{h1_dec} + {eq2_dec} + {v1_dec}");
//                     println!("{key1_dec} + {eq2_dec} + {v1_dec}");
//                     println!("{key12_dec} + {v1_dec} = {key_dec} -> {ct_dec}");
//                     println!("eq1_dec: {:?}", dbg_dec_vec);
//                     println!("vout: {vout_dec}; hout: {hout_dec}");
//                 }
//             }
//         }
//     }

//     // Berekning van alle rest van de matrix

//     let mut h_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(db_size);
//     let mut v_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(db_size);

//     for k in 0..db_size {
//         let mut h_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(max_factor);
//         let mut v_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(max_factor);

//         for i in 0..max_factor {
//             let mut h_vec: Vec<i64> = Vec::with_capacity(max_factor);
//             let mut v_vec: Vec<i64> = Vec::with_capacity(max_factor);

//             for j in 0..max_factor {
//                 let h_dec: u64 = cks.decrypt(&h_matrices[k][i][j]);
//                 let v_dec: u64 = cks.decrypt(&v_matrices[k][i][j]);

//                 if h_dec > 8 {
//                     h_vec.push((h_dec - 16) as i64);
//                 } else {
//                     h_vec.push(h_dec as i64);
//                 }

//                 if v_dec > 8 {
//                     v_vec.push((v_dec - 16) as i64);
//                 } else {
//                     v_vec.push(v_dec as i64);
//                 }
//             }

//             h_dec_matrix.push(h_vec);
//             v_dec_matrix.push(v_vec);
//         }
//         h_dec_matrices.push(h_dec_matrix);
//         v_dec_matrices.push(v_dec_matrix);
//     }

//     if DEBUG {
//         println!("-------------------");
//         println!("{}\t {}", &query, &data::NAME_LIST[index]);
//         print_matrix(&h_dec_matrices[index], "h_dec");
//         print_matrix(&v_dec_matrices[index], "v_dec");
//     }

//     let mut result_map: HashMap<usize, i64> = HashMap::new();

//     // for k in 0..db_size {
//     //     let mut score = 0;
//     //     for i in 0..max_factor {

//     //             score += h_dec_matrices[k][max_factor - 1][i];
//     //     }
//     //     score += (max_factor - 1) as i64;

//     //     result_map.insert(k, score);
//     // }

//     for k in 0..db_size {
//         let mut diag_score = 0;

//         let m = max_factor - 1;

//         for i in 1..m + 1 {
//             diag_score += &h_dec_matrices[k][i][i];
//         }

//         for i in 0..m {
//             diag_score += &v_dec_matrices[k][i + 1][i];
//         }

//         // let diff = (qlen as i64 - db_len[&k] as i64).abs();
//         // diag_score += diff as u64;

//         result_map.insert(k, diag_score);
//     }

//     #[cfg(feature = "fpga")]
//     sks.disconnect(fpga);

//     result_map
// }

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
    // let (cks, sks) = gen_keys(PARAM_MESSAGE_4_CARRY_0_KS_PBS);

    // Get the min and max lenght of the db strings
    let qlen = query.len() + 1;

    // DB processing and encrypting
    // let db_size: usize = data::NAME_LIST.len();
    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(db_size);

    // let db_size = 32;

    for i in 0..db_size {
        db_len.insert(i, data::NAME_LIST[i].len());
    }

    let _db_min_size = *db_len.values().into_iter().min().unwrap();
    let db_max_size = *db_len.values().into_iter().max().unwrap();

    // Max factor is defined as the size of the D matrix! (db_max_size + 1)
    let mut max_factor = std::cmp::max(db_max_size, qlen - 1);
    max_factor += 1;

    let th = ((max_factor as f64) / 2.0).ceil() as usize;
    // let th = 8;
    // let th = max_factor;

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
    // let lut_min_vec = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
    // let lut_vec_eq = [9u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
    // let lut_vec_1eq = [1u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

    // let lut_min = sks.generate_lookup_table_vector(&lut_min_vec);
    // let lut_1eq = sks.generate_lookup_table_vector(&lut_vec_1eq);
    // let lut_eq = sks.generate_lookup_table_vector(&lut_vec_eq);

    // let lut = BelfortFpgaLuts::lut_by_index(7, &sks.clone());
    // let lut_eq1_fpga_indexes: Vec<tfhe::core_crypto::fpga::luts::BelfortLookupTable> =
    //     (0..db_size).map(|_| lut).collect();

    // let lut = BelfortFpgaLuts::lut_by_index(6, &sks.clone());
    // let lut_eq2_fpga_indexes: Vec<tfhe::core_crypto::fpga::luts::BelfortLookupTable> =
    //     (0..db_size).map(|_| lut).collect();

    // let lut = BelfortFpgaLuts::lut_by_index(8, &sks.clone());
    // let lut_min_fpga_indexes: Vec<tfhe::core_crypto::fpga::luts::BelfortLookupTable> =
    //     (0..db_size).map(|_| lut).collect();

    let lut_min_vec_def = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_eq_vec_def = [9u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
    let lut_1eq_vec_def = [1u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

    // let lut_min = sks
    //     .generate_lookup_table_from_vector(&lut_min_vec_def);
    // let lut_1eq = sks
    //     .generate_lookup_table_from_vector(&lut_1eq_vec_def);
    // let lut_eq = sks
    //     .generate_lookup_table_from_vector(&lut_eq_vec_def);

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

    // let lut_1eq_vec = vec![lut_1eq; db_size];
    // let lut_eq_vec = vec![lut_eq; db_size];
    // let lut_min_vec = vec![lut_min; db_size];

    // let lut_1eq_index_vec = vec![7usize; db_size];
    // let lut_eq_index_vec = vec![6usize; db_size];
    // let lut_min_index_vec = vec![8usize; db_size];

    // let zero_enc_vec = vec![zero_enc.clone(); max_factor];
    let one_enc_vec = vec![one_enc.clone(); db_size];

    let one_enc_vec_ref: Vec<&Ciphertext> = one_enc_vec.iter().collect();

    for i in 1..max_factor {
        let q1_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q_enc[i - 1].clone(); db_size];
        let q2_vec: Vec<tfhe::shortint::prelude::Ciphertext> = vec![q2_enc[i - 1].clone(); db_size];

        // let query1_vec = q1_vec.iter().collect();
        // let query2_vec = q1_vec.iter().collect();

        if print {
            println!("{i}");
        }

        for j in 1..max_factor {
            if usize::abs_diff(i, j) <= th {
                let t = Instant::now();

                // Check the first part of the character
                let mut eq1 = sks.unchecked_sub_packed_parallellized(
                    q1_vec.iter().collect(),
                    get_column(&db_enc_matrix, j - 1).iter().collect(),
                );

                sks.unchecked_scalar_add_packed_parallellized_assign(&mut eq1, 16);

                let mut eq1_lut = Vec::new();

                // #[cfg(not(feature = "fpga"))]
                // eq1_lut = sks.apply_lookup_table_packed_parallellized(eq1.iter().collect(), &lut_1eq_vec);

                // #[cfg(feature = "fpga")]{

                eq1_lut = eq1.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut eq1_lut, &lut_1eq_vec_fpga);
                // }

                // #[cfg(feature = "fpga")]
                // let eq1_lut = sks.apply_lookup_table_packed_fpga(
                //     eq1.iter().collect(),
                //     lut_1eq_index_vec.clone(),
                //     fpga,
                // );

                // println!("LUT EQ1 {i} {j}");

                let eq1_ref: Vec<&Ciphertext> = eq1_lut.iter().collect(); // ?

                eq1 = sks.unchecked_sub_packed_parallellized(one_enc_vec_ref.clone(), eq1_ref);
                sks.unchecked_scalar_add_packed_parallellized_assign(&mut eq1, 16);

                let mut eq2 = sks.unchecked_sub_packed_parallellized(
                    q2_vec.iter().collect(),
                    get_column(&db1_enc_matrix, j - 1).iter().collect(),
                );
                // sks.unchecked_scalar_add_packed_assign(&mut eq2, 16);

                sks.unchecked_scalar_mul_packed_parallellized_assign(&mut eq2, 2);
                sks.unchecked_add_packed_parallellized_assign(&mut eq2, eq1.iter().collect());

                let mut eq2_lut = Vec::new();

                // #[cfg(not(feature = "fpga"))]
                // eq2_lut = sks.apply_lookup_table_packed_parallellized(eq2.iter().collect(), &lut_eq_vec);

                // #[cfg(feature = "fpga")]{

                eq2_lut = eq2.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut eq2_lut, &lut_eq_vec_fpga);
                // }

                // let eq2_lut = sks.apply_lookup_table_packed_fpga(
                //     eq2.iter().collect(),
                //     lut_eq_index_vec.clone(),
                //     fpga,
                // );

                // println!("LUT EQ2 {i} {j}");

                let vin = extract_number_elements(&v_matrices, i, j - 1);
                let hin = extract_number_elements(&h_matrices, i - 1, j);

                let v1 = sks.unchecked_scalar_add_packed_parallellized(vin.iter().collect(), 1);
                let h1 = sks.unchecked_scalar_add_packed_parallellized(hin.iter().collect(), 1);

                let key1 = sks.unchecked_scalar_mul_packed_parallellized(h1.iter().collect(), 3);
                let key12 =
                    sks.unchecked_add_packed(key1.iter().collect(), eq2_lut.iter().collect());

                let key = sks.unchecked_add_packed(key12.iter().collect(), v1.iter().collect());

                // sks.unchecked_scalar_add_packed_assign(&mut key, 16);

                let mut ct_res = Vec::new();

                // #[cfg(not(feature = "fpga"))] {
                //     ct_res = sks.apply_lookup_table_packed_parallellized(key.iter().collect(), &lut_min_vec);
                // }

                // #[cfg(feature = "fpga")]{

                ct_res = key.clone();
                fpga.fpga_utils
                    .keyswitch_bootstrap_packed(&mut ct_res, &lut_min_vec_fpga);
                // }

                // #[cfg(feature = "fpga")]
                // let mut ct_res = sks.apply_lookup_table_packed_fpga(
                //     key.iter().collect(),
                //     lut_min_index_vec.clone(),
                //     fpga,
                // );

                // println!("LUT MIN {i} {j}");

                sks.unchecked_scalar_add_packed_parallellized_assign(&mut ct_res, 16);

                let mut dbg_dec_vec: Vec<u64> = Vec::with_capacity(db_size);
                for i in 0..db_size {
                    let dec_tmp: u64 = cks.decrypt(&ct_res[i]);
                    dbg_dec_vec.push(dec_tmp);
                }

                let v_res = sks.unchecked_sub_packed_parallellized(
                    ct_res.iter().collect(),
                    hin.iter().collect(),
                );
                let h_res = sks.unchecked_sub_packed_parallellized(
                    ct_res.iter().collect(),
                    vin.iter().collect(),
                );

                write_number_elements(&mut v_matrices, &v_res, i, j);
                write_number_elements(&mut h_matrices, &h_res, i, j);

                if DEBUG {
                    let out = t.elapsed().as_secs_f64();
                    println!("Time: {out}");

                    // let eq1_dec: u64 = cks.decrypt(&eq1[index]);
                    // let eq2_dec: u64 = cks.decrypt(&eq2_lut[index]);

                    // let vin_dec: u64 = cks.decrypt(&vin[index]);
                    // let hin_dec: u64 = cks.decrypt(&hin[index]);

                    // let v1_dec: u64 = cks.decrypt(&v1[index]);
                    // let h1_dec: u64 = cks.decrypt(&h1[index]);

                    // let key1_dec: u64 = cks.decrypt(&key1[index]);

                    // let key12_dec: u64 = cks.decrypt(&key12[index]);
                    // let ct_dec: u64 = cks.decrypt(&ct_res[index]);
                    // let vout_dec: u64 = cks.decrypt(&v_res[index]);
                    // let hout_dec: u64 = cks.decrypt(&h_res[index]);

                    // let key_dec: u64 = cks.decrypt(&key[index]);

                    // println!("------------ {i}\t{j} ------------------");
                    // let padded_name = data::NAME_LIST[index].pad_to_width(max_factor - 1);

                    // println!(
                    //     "Eq: {} {} -> {} -> {}",
                    //     query_padded.chars().nth(i - 1).unwrap(),
                    //     padded_name.chars().nth(j - 1).unwrap(),
                    //     eq1_dec,
                    //     eq2_dec
                    // );
                    // println!("vin: {vin_dec}; hin: {hin_dec}");
                    // println!("key = 3 * HIN + 9 * eq + VIN");
                    // println!("3*{h1_dec} + {eq2_dec} + {v1_dec}");
                    // println!("{key1_dec} + {eq2_dec} + {v1_dec}");
                    // println!("{key12_dec} + {v1_dec} = {key_dec} -> {ct_dec}");
                    // println!("eq1_dec: {:?}", dbg_dec_vec);
                    // println!("vout: {vout_dec}; hout: {hout_dec}");
                }
            }
        }
    }

    // Berekning van alle rest van de matrix

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

    // for k in 0..db_size {
    //     let mut score = 0;
    //     for i in 0..max_factor {

    //             score += h_dec_matrices[k][max_factor - 1][i];
    //     }
    //     score += (max_factor - 1) as i64;

    //     result_map.insert(k, score);
    // }

    for k in 0..db_size {
        let mut diag_score = 0;

        let m = max_factor - 1;

        for i in 1..m + 1 {
            diag_score += &h_dec_matrices[k][i][i];
        }

        for i in 0..m {
            diag_score += &v_dec_matrices[k][i + 1][i];
        }

        // let diff = (qlen as i64 - db_len[&k] as i64).abs();
        // diag_score += diff as u64;

        result_map.insert(k, diag_score);
    }

    result_map
}
