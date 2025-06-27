use std::io::Write;
use std::ops::Add;
use std::time::{Duration, Instant};
use std::{cmp, io};

use tfhe::shortint::prelude::*;

#[cfg(not(feature = "fpga"))]
use tfhe_trivium::TriviumStreamShortint;

const BATCHED_BITS: usize = 64;

fn main() {
    let output_length_bit: usize = 64;
    let output_at_each_bit: usize = cmp::max(BATCHED_BITS, 8);

    //////////////////////////////////////////////////////////////////////////////

    let key_string = "0053A6F94C9FF24598EB".to_string();
    let mut key = [0; 80];

    for i in (0..key_string.len()).step_by(2) {
        let mut val: u8 = u8::from_str_radix(&key_string[i..i + 2], 16).unwrap();
        for j in 0..8 {
            key[8 * (i >> 1) + j] = if val % 2 == 1 { 1 } else { 0 };
            val >>= 1;
        }
    }

    let iv_string = "0D74DB42A91077DE45AC".to_string();
    let mut iv = [0; 80];

    for i in (0..iv_string.len()).step_by(2) {
        let mut val: u8 = u8::from_str_radix(&iv_string[i..i + 2], 16).unwrap();
        for j in 0..8 {
            iv[8 * (i >> 1) + j] = if val % 2 == 1 { 1 } else { 0 };
            val >>= 1;
        }
    }

    // Params and Keys
    let params: ClassicPBSParameters = tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    let client_key: ClientKey = ClientKey::new(params);
    let server_key: ServerKey = ServerKey::new(&client_key);

    let cipher_key = key.map(|x| client_key.encrypt(x));

    #[cfg(feature = "fpga")]
    let mut trivium = {
        use tfhe::integer::fpga::BelfortServerKey;
        use tfhe::integer::ServerKey as IntegerServerKey;
        use tfhe_trivium::TriviumStreamFPGAShortint;

        let integer_server_key =
            IntegerServerKey::new_radix_server_key_from_shortint(server_key.clone());
        let fpga_key = BelfortServerKey::from(&integer_server_key);
        let mut trivium = TriviumStreamFPGAShortint::new(cipher_key, iv, server_key, fpga_key);
        trivium.init(BATCHED_BITS);
        trivium
    };
    #[cfg(not(feature = "fpga"))]
    let mut trivium = {
        use tfhe::{generate_keys, ConfigBuilder};

        let ksk = KeySwitchingKey::new(
            (&client_key, Some(&server_key)),
            (&client_key, &server_key),
            V0_11_PARAM_KEYSWITCH_1_1_KS_PBS_TO_2_2_KS_PBS,
        );
        let config = ConfigBuilder::default().build();
        let (_, hl_server_key) = generate_keys(config);

        TriviumStreamShortint::new(cipher_key, iv, server_key, ksk, hl_server_key)
    };
    println!("\n");

    for i in 0..128 {
        print!("\x1b[90m{:5}\x1b[0m ", i);

        let mut elapsed = Duration::from_secs(0);

        for _ in 0..(output_length_bit / output_at_each_bit) {
            let mut cipher_outputs = Vec::<Ciphertext>::with_capacity(output_at_each_bit);

            let start = Instant::now();
            while cipher_outputs.len() < output_at_each_bit {
                let prallel_computed_cipher_outputs = trivium.next_64();

                for element in prallel_computed_cipher_outputs {
                    cipher_outputs.push(element);
                }
            }
            elapsed = elapsed.add(start.elapsed());

            let decrypted_bits: Vec<_> = cipher_outputs
                .iter()
                .map(|item| client_key.decrypt(&item))
                .collect();

            let hexstring = hex_string_from_vec(decrypted_bits);
            print!("{}", hexstring);
            io::stdout().flush().unwrap();
        }

        print!(" \x1b[90m{:?}\x1b[0m\n", elapsed);
    }
}

fn hex_string_from_vec(a: Vec<u64>) -> String {
    assert!(a.len() % 8 == 0);
    let mut hexadecimal: String = String::new();

    for test in a.chunks(8) {
        // Only use the LSB of each u64 in the match
        let high_bits = [test[4] & 1, test[5] & 1, test[6] & 1, test[7] & 1];
        let low_bits = [test[0] & 1, test[1] & 1, test[2] & 1, test[3] & 1];

        let high_val =
            (high_bits[0] << 0) | (high_bits[1] << 1) | (high_bits[2] << 2) | (high_bits[3] << 3);
        let low_val =
            (low_bits[0] << 0) | (low_bits[1] << 1) | (low_bits[2] << 2) | (low_bits[3] << 3);

        hexadecimal.push(
            std::char::from_digit(high_val as u32, 16)
                .unwrap()
                .to_ascii_uppercase(),
        );
        hexadecimal.push(
            std::char::from_digit(low_val as u32, 16)
                .unwrap()
                .to_ascii_uppercase(),
        );
    }

    hexadecimal
}
