use tfhe::core_crypto::fpga::keyswitch_bootstrap::KeyswitchBootstrapPacked;
use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::shortint::prelude::*;

use crate::data;
use crate::enc_struct::EncStruct;
use crate::util::{
    self, apply_lookup_table_packed, unchecked_add_packed, unchecked_add_packed_assign,
    unchecked_scalar_add_packed, unchecked_scalar_add_packed_assign, unchecked_scalar_mul_packed,
    unchecked_scalar_mul_packed_assign, unchecked_sub_packed,
};

use pad::PadStr;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Clone)]
pub enum InputMode {
    Normal,
    Editing,
    Process,
    FEditing,
    FProcess,
}

/// App holds the state of the application
#[derive(Clone)]
pub struct App {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    pub character_index: usize,
    /// Current input mode
    pub input_mode: InputMode,
    /// History of recorded messages
    pub messages: Vec<(String, String, String, String)>,
    pub progress_done: Vec<u8>,
}

impl App {
    pub const fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
            progress_done: Vec::new(),
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    pub fn byte_index(&mut self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let current_index = self.character_index;
        let is_cursor_leftmost = current_index == 0;
        if is_cursor_leftmost {
            return;
        }

        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = self.input.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        self.input = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    pub fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    pub fn process_enc_query_enc_db(&mut self, enc_struct: &mut EncStruct) {
        // Get the min and max lenght of the db strings
        let qlen = enc_struct.query.len() + 1;

        // DB processing and encrypting
        // let enc_struct.db_size: usize = data::NAME_LIST.len();
        let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(enc_struct.db_size);

        for i in 0..enc_struct.db_size {
            db_len.insert(i, data::NAME_LIST[i].len());
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
            .collect::<Vec<tfhe::shortint::Ciphertext>>();

        let q2_enc = query_padded
            .bytes() // convert char to int
            .map(|c| enc_struct.cks.encrypt(((c - scale_factor) >> 4) as u64))
            .collect::<Vec<tfhe::shortint::Ciphertext>>();

        let zero_enc = enc_struct.cks.encrypt(0u64);
        let one_enc = enc_struct.cks.encrypt(1u64);

        let mut db_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(enc_struct.db_size);
        let mut db1_enc_matrix: Vec<Vec<Ciphertext>> = Vec::with_capacity(enc_struct.db_size);

        for i in 0..enc_struct.db_size {
            let padded_name = data::NAME_LIST[i].pad_to_width(max_factor - 1);

            let name_enc = padded_name
                .bytes() // convert char to int
                .map(|c| enc_struct.cks.encrypt((c - scale_factor) as u64)) // Encrypts
                .collect::<Vec<tfhe::shortint::Ciphertext>>();

            let name1_enc = padded_name
                .bytes() // convert char to int
                .map(|c| enc_struct.cks.encrypt(((c - scale_factor) >> 4) as u64))
                .collect::<Vec<tfhe::shortint::Ciphertext>>();

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

        let lut_1eq_fpga = LookupVector::new(&lut_1eq_vec_def);
        let lut_eq_fpga = LookupVector::new(&lut_eq_vec_def);
        let lut_min_fpga = LookupVector::new(&lut_min_vec_def);

        let lut_1eq_vec_fpga = vec![lut_1eq_fpga; enc_struct.db_size];
        let lut_eq_vec_fpga = vec![lut_eq_fpga; enc_struct.db_size];
        let lut_min_vec_fpga = vec![lut_min_fpga; enc_struct.db_size];

        // Build and fill all the h_matrices
        let mut h_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> =
            Vec::with_capacity(enc_struct.db_size);
        let mut v_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> =
            Vec::with_capacity(enc_struct.db_size);

        for _ in 0..enc_struct.db_size {
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

    pub fn process_plain_query_enc_db(&mut self, enc_struct: &mut EncStruct) {
        // Get the min and max lenght of the db strings
        let qlen = enc_struct.query.len() + 1;

        // DB processing and encrypting
        let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(enc_struct.db_size);

        for i in 0..enc_struct.db_size {
            db_len.insert(i, data::NAME_LIST[i].len());
        }

        let db_max_size = db_len.values().max().unwrap();

        // Max factor is defined as the size of the D matrix! (db_max_size + 1)
        let mut max_factor = std::cmp::max(*db_max_size, qlen - 1);
        max_factor += 1;

        let th: usize = ((max_factor as f64) / 2.0).ceil() as usize;

        let zero_enc = enc_struct.cks.encrypt(0u64);

        // Build and fill all the h_matrices
        let mut h_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> =
            Vec::with_capacity(enc_struct.db_size);
        let mut v_matrices: Vec<Vec<Vec<tfhe::shortint::Ciphertext>>> =
            Vec::with_capacity(enc_struct.db_size);

        for _ in 0..enc_struct.db_size {
            let mut h_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);
            let mut v_matrix: Vec<Vec<tfhe::shortint::Ciphertext>> = Vec::with_capacity(max_factor);

            for _ in 0..max_factor {
                let mut vec: Vec<tfhe::shortint::Ciphertext> = Vec::with_capacity(max_factor);
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

        let fpga_lut_single = LookupVector::new(&lut_min_vec_def);
        let lut_min_vec_fpga = vec![fpga_lut_single; enc_struct.db_size];

        enc_struct.max_factor = max_factor;
        enc_struct.th = th;
        enc_struct.v_matrices = v_matrices;
        enc_struct.h_matrices = h_matrices;
        enc_struct.lut_min_vec_sw = lut_min_vec;
        enc_struct.lut_min_vec_fpga = lut_min_vec_fpga;

        enc_struct.time = Instant::now();
    }

    pub fn process_part_i(&mut self, index: usize, enc_struct: &mut EncStruct, fpga_enable: bool) {
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
                    enc_struct
                        .fpga_key
                        .fpga_utils
                        .keyswitch_bootstrap_packed(&mut eq1_lut, &enc_struct.lut_1eq_vec_fpga);
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
                    enc_struct.fpga_key.apply_lookup_vector_packed_assign(
                        &mut eq2_lut,
                        &enc_struct.lut_eq_vec_fpga,
                    );
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

                let key = unchecked_add_packed(
                    &enc_struct.sks,
                    key12.iter().collect(),
                    v1.iter().collect(),
                );

                let mut ct_res = Vec::new();

                if fpga_enable {
                    ct_res = key.clone();
                    enc_struct
                        .fpga_key
                        .fpga_utils
                        .keyswitch_bootstrap_packed(&mut ct_res, &enc_struct.lut_min_vec_fpga);
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
        self.progress_done.push(index as u8);
    }

    pub fn process_plain_part_i(
        &mut self,
        index: usize,
        enc_struct: &mut EncStruct,
        fpga_enable: bool,
    ) {
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

                let key12 = unchecked_add_packed(
                    &enc_struct.sks,
                    key1.iter().collect(),
                    eq.iter().collect(),
                );

                let key = unchecked_add_packed(
                    &enc_struct.sks,
                    key12.iter().collect(),
                    v1.iter().collect(),
                );

                let mut ct_res: Vec<Ciphertext>;
                if fpga_enable {
                    ct_res = key.clone();
                    enc_struct
                        .fpga_key
                        .fpga_utils
                        .keyswitch_bootstrap_packed(&mut ct_res, &enc_struct.lut_min_vec_fpga);
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
        self.progress_done.push(index as u8);
    }

    pub fn post_process(&mut self, enc_struct: &mut EncStruct, fpga_enable: bool) {
        // Computation of the rest of the matrix
        let mut h_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(enc_struct.db_size);
        let mut v_dec_matrices: Vec<Vec<Vec<i64>>> = Vec::with_capacity(enc_struct.db_size);

        for k in 0..enc_struct.db_size {
            let mut h_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(enc_struct.max_factor);
            let mut v_dec_matrix: Vec<Vec<i64>> = Vec::with_capacity(enc_struct.max_factor);

            for i in 0..enc_struct.max_factor {
                let mut h_vec: Vec<i64> = Vec::with_capacity(enc_struct.max_factor);
                let mut v_vec: Vec<i64> = Vec::with_capacity(enc_struct.max_factor);

                for j in 0..enc_struct.max_factor {
                    let h_dec: u64 = enc_struct.cks.decrypt(&enc_struct.h_matrices[k][i][j]);
                    let v_dec: u64 = enc_struct.cks.decrypt(&enc_struct.v_matrices[k][i][j]);

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

        let mut result_map: HashMap<usize, i64> = HashMap::new();

        for k in 0..enc_struct.db_size {
            let mut diag_score = 0;

            let m = enc_struct.max_factor - 1;

            for i in 1..m + 1 {
                diag_score += &h_dec_matrices[k][i][i];
            }

            for i in 0..m {
                diag_score += &v_dec_matrices[k][i + 1][i];
            }

            result_map.insert(k, diag_score);
        }

        let sec = enc_struct.time.elapsed().as_secs_f64();

        let mut max_diff = 0;
        let mut matched_name = "None".to_string();

        let time_string = format!("{:.5}", sec);

        let comment = if fpga_enable & enc_struct.input.starts_with("p:") {
            "plaintext query and FPGA Acceleration".to_owned()
        } else if fpga_enable {
            "FPGA Acceleration".to_owned()
        } else if enc_struct.input.starts_with("p:") {
            "plaintext query".to_owned()
        } else {
            "Normal execution".to_owned()
        };

        for i in 0..enc_struct.db_size {
            let enc_score = result_map.get(&i).unwrap();

            let diff = i64::abs_diff(
                data::NAME_LIST[i].len().try_into().unwrap(),
                enc_struct.query.len().try_into().unwrap(),
            ) as i64;

            if i64::abs_diff(*enc_score, data::NAME_LIST[i].len().try_into().unwrap()) as i64 - diff
                > max_diff
            {
                matched_name = data::NAME_LIST[i].to_string();
                max_diff = i64::abs_diff(*enc_score, data::NAME_LIST[i].len().try_into().unwrap())
                    as i64
                    - diff;
            }
        }

        if max_diff <= 5 {
            self.messages.push((
                enc_struct.query.clone(),
                "No".to_owned(),
                time_string,
                "Normal execution".to_owned(),
            ));
        } else {
            self.messages
                .push((enc_struct.query.clone(), matched_name, time_string, comment));
        }

        self.progress_done.clear();
        self.input.clear();
        self.reset_cursor();
    }
}
