use std::collections::HashMap;
use std::time::Instant;
use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::shortint::prelude::*;
use tfhe::integer::fpga::BelfortServerKey;

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
  pub lut_eq_vec_fpga: Vec<LookupVector>
}
