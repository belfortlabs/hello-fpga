#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
// Supress not FFI-safe warning
#![allow(improper_ctypes)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use std::collections::HashMap;
use std::time::Instant;
use tfhe::shortint::server_key::LookupTable;

use tfhe::shortint::prelude::*;

use tfhe::core_crypto::commons::generators::DeterministicSeeder;
use tfhe::core_crypto::commons::math::random::Seed;
use tfhe::core_crypto::fpga::utils::Connect;
use tfhe::core_crypto::prelude::*;
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::shortint::engine::ShortintEngine;
use tfhe::{generate_keys, set_server_key, ConfigBuilder, FheUint4};

use tfhe::core_crypto::prelude::*;
use tfhe::integer::ServerKey as IntegerServerKey;
use tfhe::shortint::prelude::*;
use tfhe::shortint::server_key::ShortintBootstrappingKey;

use pad::PadStr;

mod myers;

fn get_decrypt_packed(data: &Vec<Ciphertext>, cks: &tfhe::shortint::ClientKey) -> Vec<u64> {
    let mut result = Vec::new();
    for element in data {
        let tmp: u64 = cks.decrypt(element);
        result.push(tmp);
    }
    result
}

fn main() {
    // // Build up the input strings
    let x = "JanPeter Danivers";
    let qlen = x.len();

    // DB processing and encrypting
    let db_size: usize = super_mod::data::NAME_LIST.len();
    // let db_size = 768;

    let mut db_len: HashMap<usize, usize> = HashMap::with_capacity(db_size);

    for i in 0..db_size {
        db_len.insert(i, super_mod::data::NAME_LIST[i].len());
    }

    let db_max_size = *db_len.values().into_iter().max().unwrap();

    let max_factor = std::cmp::max(db_max_size, qlen);

    let mut plain_score = HashMap::new();

    for i in 0..db_size {
        let lev = myers::levenshtein_plain_matrix(
            &x.pad_to_width(max_factor),
            &super_mod::data::NAME_LIST[i].pad_to_width(max_factor),
        );

        plain_score.insert(i, lev as i64);
        println!(
            "{} [{}]: {}",
            super_mod::data::NAME_LIST[i],
            super_mod::data::NAME_LIST[i].len(),
            lev
        );
    }

    // Params and Keys
    let params: ClassicPBSParameters = tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    let cks: ClientKey = ClientKey::new(params);
    let sks: ServerKey = ServerKey::new(&cks);
    let integer_server_key: IntegerServerKey =
        tfhe::integer::ServerKey::new_radix_server_key_from_shortint(sks.clone());
    let mut fpga_key = BelfortServerKey::from(&integer_server_key);

    #[cfg(feature = "fpga")]
    {
        fpga_key.connect();
    }

    let t = Instant::now();
    let lev_enc = myers::process_enc_query_enc_db(&x, &cks, &sks, &mut fpga_key, db_size, true);
    let sec = t.elapsed().as_secs_f64();
    println!("Enc Lev[{db_size}]: {sec} s");

    let mut matched_name = String::new();
    let mut max_diff = 2;

    for i in 0..db_size {
        let enc_score = lev_enc.get(&i).unwrap();

        if i64::abs_diff(
            *enc_score,
            super_mod::data::NAME_LIST[i].len().try_into().unwrap(),
        ) > max_diff
        {
            matched_name = super_mod::data::NAME_LIST[i].to_string();
            max_diff = i64::abs_diff(
                *enc_score,
                super_mod::data::NAME_LIST[i].len().try_into().unwrap(),
            );
        }
    }

    if max_diff <= 5 {
        println!("No match found; try larger query");
    } else {
        println!("Matched name: {matched_name} - {max_diff}");
    }

    for i in 0..db_size {
        let enc_score = lev_enc.get(&i).unwrap();
        let plain_score = plain_score.get(&i).unwrap();

        if enc_score != plain_score {
            println!(
                "{} [{}]: \t Plain: {} - Enc: {}",
                super_mod::data::NAME_LIST[i],
                super_mod::data::NAME_LIST[i].len(),
                plain_score,
                enc_score
            );
        } else {
            println!(
                "{} [{}]: \t {}",
                super_mod::data::NAME_LIST[i],
                super_mod::data::NAME_LIST[i].len(),
                enc_score
            );
        }
    }

    #[cfg(feature = "fpga")]
    {
        fpga_key.disconnect();
    }
}
