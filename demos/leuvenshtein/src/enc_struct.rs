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
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::shortint::prelude::*;

// Struct to maintain the state of the complete application
pub struct EncStruct<'a> {
    pub input: String,
    pub query: String,
    pub max_factor: usize,
    pub db_size: usize,
    pub th: usize,
    pub time: Instant,
    pub q_enc: Vec<tfhe::shortint::Ciphertext>,
    pub q2_enc: Vec<tfhe::shortint::Ciphertext>,
    pub db_enc_matrix: Vec<Vec<tfhe::shortint::Ciphertext>>,
    pub db1_enc_matrix: Vec<Vec<tfhe::shortint::Ciphertext>>,
    pub db_enc_map: HashMap<usize, HashMap<char, Vec<tfhe::shortint::Ciphertext>>>,
    pub sks: ServerKey,
    pub cks: ClientKey,
    pub fpga_key: &'a mut BelfortServerKey,
    pub one_enc_vec: Vec<tfhe::shortint::Ciphertext>,
    pub v_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>>,
    pub h_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>>,
    pub lut_min_vec_sw: Vec<tfhe::shortint::server_key::LookupTable<Vec<u64>>>,
    pub lut_1eq_vec_sw: Vec<tfhe::shortint::server_key::LookupTable<Vec<u64>>>,
    pub lut_eq_vec_sw: Vec<tfhe::shortint::server_key::LookupTable<Vec<u64>>>,
    pub lut_min_vec_fpga: Vec<LookupVector>,
    pub lut_1eq_vec_fpga: Vec<LookupVector>,
    pub lut_eq_vec_fpga: Vec<LookupVector>,
}
