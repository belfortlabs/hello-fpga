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

use std::collections::HashMap;
use std::time::Instant;
use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
#[cfg(feature = "fpga")]
use tfhe::integer::ServerKey as IntegerServerKey;
use tfhe::shortint::prelude::*;
use tfhe::shortint::server_key::LookupTable;

use crate::data::NAME_LIST;

#[cfg(feature = "fpga")]
use tfhe::integer::fpga::BelfortServerKey;

// Struct to maintain the state of the complete application
pub struct EncStruct {
    pub input: String,
    pub query: String,
    pub max_factor: usize,
    pub db_size: usize,
    pub th: usize,
    pub time: Instant,
    pub q_enc: Vec<Ciphertext>,
    pub q2_enc: Vec<Ciphertext>,
    pub db_enc_matrix: Vec<Vec<Ciphertext>>,
    pub db1_enc_matrix: Vec<Vec<Ciphertext>>,
    pub db_enc_map: HashMap<usize, HashMap<char, Vec<Ciphertext>>>,
    pub sks: ServerKey,
    pub cks: ClientKey,
    #[cfg(feature = "fpga")]
    pub fpga_key: BelfortServerKey,
    pub one_enc_vec: Vec<Ciphertext>,
    pub v_matrices: Vec<Vec<Vec<Ciphertext>>>,
    pub h_matrices: Vec<Vec<Vec<Ciphertext>>>,
    pub lut_min_vec_sw: Vec<LookupTable<Vec<u64>>>,
    pub lut_1eq_vec_sw: Vec<LookupTable<Vec<u64>>>,
    pub lut_eq_vec_sw: Vec<LookupTable<Vec<u64>>>,
    pub lut_min_fpga: LookupVector,
    pub lut_1eq_fpga: LookupVector,
    pub lut_eq_fpga: LookupVector,
}

impl EncStruct {
    pub fn new(
        max_factor: usize,
        db_processed: HashMap<usize, HashMap<char, Vec<Ciphertext>>>,
        cks: ClientKey,
    ) -> Self {
        let sks: ServerKey = ServerKey::new(&cks);

        #[cfg(feature = "fpga")]
        let fpga_key = {
            let integer_server_key =
                IntegerServerKey::new_radix_server_key_from_shortint(sks.clone());

            let mut fpga_key = BelfortServerKey::from(&integer_server_key);
            fpga_key.connect();
            fpga_key
        };

        let db_size = NAME_LIST.len();

        let lut_min_vec_def = [0u64, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0].to_vec();
        let lut_eq_vec_def = [9u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
        let lut_1eq_vec_def = [1u64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();

        let lut_min = sks.generate_lookup_table_from_vector(&lut_min_vec_def);
        let lut_1eq = sks.generate_lookup_table_from_vector(&lut_1eq_vec_def);
        let lut_eq = sks.generate_lookup_table_from_vector(&lut_eq_vec_def);

        let lut_min_vec = vec![lut_min; db_size];
        let lut_1eq_vec = vec![lut_1eq; db_size];
        let lut_eq_vec = vec![lut_eq; db_size];

        let lut_min_fpga = sks.generate_lookup_vector(&|x| lut_min_vec_def[x as usize]);
        let lut_eq_fpga = sks.generate_lookup_vector(&|x| lut_eq_vec_def[x as usize]);
        let lut_1eq_fpga = sks.generate_lookup_vector(&|x| lut_1eq_vec_def[x as usize]);

        Self {
            input: String::new(),
            query: String::new(),
            max_factor,
            db_size,
            th: 0,
            time: Instant::now(),
            q_enc: Vec::new(),
            q2_enc: Vec::new(),
            db_enc_matrix: Vec::new(),
            db1_enc_matrix: Vec::new(),
            db_enc_map: db_processed,
            sks,
            cks,
            #[cfg(feature = "fpga")]
            fpga_key,
            one_enc_vec: Vec::new(),
            v_matrices: Vec::new(),
            h_matrices: Vec::new(),
            lut_1eq_vec_sw: lut_1eq_vec,
            lut_eq_vec_sw: lut_eq_vec,
            lut_min_vec_sw: lut_min_vec,
            lut_1eq_fpga,
            lut_eq_fpga,
            lut_min_fpga,
        }
    }
}
