use aes::{
    Aes128,
    cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray},
};

use crossterm::style::Color;
use rand_chacha::rand_core::{OsRng, RngCore};
use std::{io::Result, time::Instant};
use tfhe::{
    BelfortServerKey, FheUint128,
    integer::{IntegerCiphertext, RadixCiphertext, gen_keys_radix},
    set_server_key,
    shortint::prelude::PARAM_MESSAGE_2_CARRY_2_KS_PBS,
};

use crate::{
    algorithm::{
        FheAes,
        constants::{AES_128_KEY_SIZE, AES_BLOCK_SIZE},
    },
    demo::{
        constants::{INDENTED_STEPS_COLOR, NEWS_COLOR, STEPS_COLOR, TIME_COLOR},
        ui::worker::LogKind,
    },
};

pub(crate) fn homomorphic_addition(
    x: u128,
    y: u128,
    client_log: &mut dyn FnMut(&str, Color),
    server_log: &mut dyn FnMut(&str, Color),
    start_step: &mut dyn FnMut(LogKind, &str, Color) -> u32,
    end_step: &mut dyn FnMut(u32, bool),
) -> Result<u128> {
    let step_id = start_step(LogKind::Client, "Generating a random AES key:", STEPS_COLOR);
    let key = gen_key_aes128();
    end_step(step_id, true);

    let x_bytes = x.to_be_bytes();
    let y_bytes = y.to_be_bytes();
    let key_full = u128::from_be_bytes(key);
    client_log(&format!("   0x{key_full:032X}"), INDENTED_STEPS_COLOR);

    let (client_key, server_key) = gen_keys_radix(PARAM_MESSAGE_2_CARRY_2_KS_PBS, 4);
    let mut fpga_key = BelfortServerKey::from(&server_key);

    #[cfg(not(feature = "emulate_fpga"))]
    let step_id = start_step(LogKind::Client, "Connecting to the FPGA(s)", STEPS_COLOR);

    #[cfg(feature = "emulate_fpga")]
    let step_id = start_step(LogKind::Client, "Emulating FPGA", STEPS_COLOR);

    fpga_key.connect();
    set_server_key(fpga_key.clone());
    end_step(step_id, true);

    let step_id = start_step(
        LogKind::Client,
        "Encrypting the AES key with FHE and expanding",
        STEPS_COLOR,
    );
    let fhe = FheAes::new(&fpga_key);
    let fhe_key = core::array::from_fn(|i| client_key.encrypt(key[i] as u64));

    let exp_key = fhe.expand_key(&fhe_key);
    end_step(step_id, true);

    let step_id = start_step(
        LogKind::Client,
        "Performing AES encryption of the operands:",
        STEPS_COLOR,
    );

    let [enc_x, enc_y] = encrypt_reference_aes128(&[x_bytes, y_bytes], key)
        .try_into()
        .unwrap();

    end_step(step_id, true);

    let enc_x_full = u128::from_be_bytes(enc_x);
    let enc_y_full = u128::from_be_bytes(enc_y);

    client_log(
        &format!("   Operand 1 -> 0x{enc_x_full:032X}"),
        INDENTED_STEPS_COLOR,
    );
    client_log(
        &format!("   Operand 2 -> 0x{enc_y_full:032X}"),
        INDENTED_STEPS_COLOR,
    );

    client_log(
        "!!! With transciphering, 32 B sent instead of 32 MiB",
        NEWS_COLOR,
    );
    client_log("  → ~99.9999% memory bandwidth saved", NEWS_COLOR);

    for _ in 0..9 {
        server_log(
            "\u{200B}",
            Color::Rgb {
                r: 240,
                g: 43,
                b: 43,
            },
        );
    }

    let step_id = start_step(
        LogKind::Server,
        "Migrating AES ciphertexts into FHE domain",
        STEPS_COLOR,
    );
    let start = Instant::now();
    let [enc_x, enc_y] = encrypt_reference_aes128(&[x_bytes, y_bytes], key)
        .try_into()
        .unwrap();
    let fhe_aes_x = core::array::from_fn(|i| server_key.create_trivial_radix(enc_x[i] as u64, 4));
    let fhe_aes_y = core::array::from_fn(|i| server_key.create_trivial_radix(enc_y[i] as u64, 4));
    let duration = start.elapsed();
    server_log(
        &format!(
            "  → ⏱ Time elapsed: {:.2}ms",
            duration.as_secs_f64() * 1000.0
        ),
        TIME_COLOR,
    );
    end_step(step_id, true);

    let step_id = start_step(
        LogKind::Server,
        "Decrypting AES within FHE domain",
        STEPS_COLOR,
    );
    let start = Instant::now();
    let [dec_x, dec_y] = fhe
        .aes_dec_blocks_parallelized(&exp_key, &[fhe_aes_x, fhe_aes_y])
        .try_into()
        .unwrap();
    let duration = start.elapsed();
    server_log(
        &format!("  → ⏱ Time elapsed: {:.2}s", duration.as_secs_f64()),
        TIME_COLOR,
    );
    end_step(step_id, true);

    let step_id = start_step(
        LogKind::Server,
        "Computing the sum in FHE domain",
        STEPS_COLOR,
    );
    let start = Instant::now();
    let fhe_x = from_bytes_to_fhe_uint128(&dec_x);
    let fhe_y = from_bytes_to_fhe_uint128(&dec_y);
    let sum = &fhe_x + &fhe_y;
    let sum_bytes = from_fhe_uint128_to_bytes(&sum);
    let duration = start.elapsed();
    server_log(
        &format!("  → ⏱ Time elapsed: {:.2}s", duration.as_secs_f64()),
        TIME_COLOR,
    );
    end_step(step_id, true);

    server_log(&format!("!!! Sent for client-side decryption:"), NEWS_COLOR);
    server_log(&format!("  → Only client will know the result"), NEWS_COLOR);

    for _ in 0..8 {
        client_log(
            "\u{200B}",
            Color::Rgb {
                r: 240,
                g: 43,
                b: 43,
            },
        );
    }

    let step_id = start_step(LogKind::Client, "Decrypting result", STEPS_COLOR);
    let dec_sum: [u8; AES_BLOCK_SIZE] = core::array::from_fn(|i| client_key.decrypt(&sum_bytes[i]));
    let res = u128::from_be_bytes(dec_sum);
    assert_eq!(x + y, res);
    end_step(step_id, true);

    client_log("!!! Result displayed above", NEWS_COLOR);
    fpga_key.disconnect();
    Ok(res)
}

fn gen_key_aes128() -> [u8; AES_128_KEY_SIZE] {
    let mut k = [0u8; AES_128_KEY_SIZE];
    OsRng.fill_bytes(&mut k);
    k
}


fn encrypt_reference_aes128(
    blocks: &[[u8; AES_BLOCK_SIZE]],
    key: [u8; AES_128_KEY_SIZE],
) -> Vec<[u8; AES_BLOCK_SIZE]> {
    let aes = Aes128::new(&GenericArray::from(key));
    blocks
        .iter()
        .map(|b| {
            let mut tmp = GenericArray::clone_from_slice(b);
            aes.encrypt_block(&mut tmp);
            tmp.into()
        })
        .collect()
}

fn from_bytes_to_fhe_uint128(bytes: &[RadixCiphertext; 16]) -> FheUint128 {
    let mut blocks = Vec::with_capacity(64);
    for ct in bytes.iter().rev() {
        blocks.extend_from_slice(ct.blocks());
    }
    FheUint128::try_from(RadixCiphertext::from(blocks)).unwrap()
}

fn from_fhe_uint128_to_bytes(ct: &FheUint128) -> [RadixCiphertext; 16] {
    ct.clone()
        .into_raw_parts()
        .0
        .blocks()
        .chunks(4)
        .enumerate()
        .map(|(_i, c)| RadixCiphertext::from(c.to_vec()))
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
