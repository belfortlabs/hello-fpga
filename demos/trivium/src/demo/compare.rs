use tfhe::prelude::*;
use tfhe::{generate_keys, ConfigBuilder, FheBool};
use tfhe_trivium::TriviumStream;

use tfhe::boolean::engine::BooleanEngine;
use tfhe::boolean::prelude::*;
use tfhe_trivium::TriviumStreamFPGA;

use chrono::prelude::*;
use std::fs::OpenOptions;
use std::io::prelude::*;

const PARALLEL_BITS: usize = 16;

fn main() -> std::io::Result<()> {
  let mut boolean_engine = BooleanEngine::new();

  let hw_params = tfhe::core_crypto::fpga::boolean::parameters::DEFAULT_PARAMETERS_KS_PBS;
  let hw_client_key = boolean_engine.create_client_key(*hw_params);
  let hw_server_key = boolean_engine.create_server_key(&hw_client_key);

  let sw_config = ConfigBuilder::default().build();
  let (sw_client_key, sw_server_key) = generate_keys(sw_config);

  let key_string = "0053A6F94C9FF24598EB".to_string();
  let mut key = [false; 80];

  for i in (0..key_string.len()).step_by(2) {
    let mut val: u8 = u8::from_str_radix(&key_string[i..i + 2], 16).unwrap();
    for j in 0..8 {
      key[8 * (i >> 1) + j] = val % 2 == 1;
      val >>= 1;
    }
  }

  let iv_string = "0D74DB42A91077DE45AC".to_string();
  let mut iv = [false; 80];

  for i in (0..iv_string.len()).step_by(2) {
    let mut val: u8 = u8::from_str_radix(&iv_string[i..i + 2], 16).unwrap();
    for j in 0..8 {
      iv[8 * (i >> 1) + j] = val % 2 == 1;
      val >>= 1;
    }
  }

  let hw_cipher_key = key.map(|x| hw_client_key.encrypt(x));
  let sw_cipher_key = key.map(|x| FheBool::encrypt(x, &sw_client_key));

  let mut hw_trivium = TriviumStreamFPGA::<Ciphertext>::new(hw_cipher_key, iv, hw_server_key, hw_params);
  let mut sw_trivium = TriviumStream::<FheBool>::new(sw_cipher_key, iv, &sw_server_key);

  hw_trivium.init(PARALLEL_BITS);
  sw_trivium.init(PARALLEL_BITS);

  let mut counter_fail: u32 = 0;
  let mut counter_index: u64 = 0;

  while counter_fail < 10 {
    let hw_cipher_outputs = hw_trivium.next_n(PARALLEL_BITS);
    let sw_cipher_outputs = sw_trivium.next_n(PARALLEL_BITS);

    let hw_decrypted_bits: Vec<_> = hw_cipher_outputs
      .iter()
      .map(|item| hw_client_key.decrypt(&item))
      .collect();

    let sw_decrypted_bits: Vec<_> = sw_cipher_outputs
      .iter()
      .map(|item| item.decrypt(&sw_client_key))
      .collect();

    let hw_hexstring = hex_string_from_vecbool(hw_decrypted_bits);
    let sw_hexstring = hex_string_from_vecbool(sw_decrypted_bits);

    if hw_hexstring != sw_hexstring {
      counter_fail += 1;

      let timestamp = Local::now();
      let formatted_timestamp = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
      let msg = format!(
        "{} Fail @ {} | {}=/={}\n",
        formatted_timestamp, counter_index, hw_hexstring, sw_hexstring
      );

      log_to_file(msg.clone())?;
      print!("{}", msg);
    }

    counter_index += 1;

    if counter_index % 1000 == 0 {
      let timestamp = Local::now();
      let formatted_timestamp = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
      let msg = format!("{} Counter {}\n", formatted_timestamp, counter_index);

      log_to_file(msg.clone())?;
      print!("{}", msg);
    }
  }

  Ok(())
}

fn log_to_file(msg: String) -> std::io::Result<()> {
  let mut file = OpenOptions::new().create(true).append(true).open("log.txt")?;

  file.write_all(msg.as_bytes())?;

  file.flush()?;
  file.sync_all()?;
  drop(file);

  Ok(())
}

fn hex_string_from_vecbool(a: Vec<bool>) -> String {
  assert!(a.len() % 8 == 0);
  let mut hexadecimal: String = "".to_string();
  for test in a.chunks(8) {
    // Encoding is bytes in LSB order
    match test[4..8] {
      [false, false, false, false] => hexadecimal.push('0'),
      [true, false, false, false] => hexadecimal.push('1'),
      [false, true, false, false] => hexadecimal.push('2'),
      [true, true, false, false] => hexadecimal.push('3'),

      [false, false, true, false] => hexadecimal.push('4'),
      [true, false, true, false] => hexadecimal.push('5'),
      [false, true, true, false] => hexadecimal.push('6'),
      [true, true, true, false] => hexadecimal.push('7'),

      [false, false, false, true] => hexadecimal.push('8'),
      [true, false, false, true] => hexadecimal.push('9'),
      [false, true, false, true] => hexadecimal.push('A'),
      [true, true, false, true] => hexadecimal.push('B'),

      [false, false, true, true] => hexadecimal.push('C'),
      [true, false, true, true] => hexadecimal.push('D'),
      [false, true, true, true] => hexadecimal.push('E'),
      [true, true, true, true] => hexadecimal.push('F'),
      _ => (),
    };
    match test[0..4] {
      [false, false, false, false] => hexadecimal.push('0'),
      [true, false, false, false] => hexadecimal.push('1'),
      [false, true, false, false] => hexadecimal.push('2'),
      [true, true, false, false] => hexadecimal.push('3'),

      [false, false, true, false] => hexadecimal.push('4'),
      [true, false, true, false] => hexadecimal.push('5'),
      [false, true, true, false] => hexadecimal.push('6'),
      [true, true, true, false] => hexadecimal.push('7'),

      [false, false, false, true] => hexadecimal.push('8'),
      [true, false, false, true] => hexadecimal.push('9'),
      [false, true, false, true] => hexadecimal.push('A'),
      [true, true, false, true] => hexadecimal.push('B'),

      [false, false, true, true] => hexadecimal.push('C'),
      [true, false, true, true] => hexadecimal.push('D'),
      [false, true, true, true] => hexadecimal.push('E'),
      [true, true, true, true] => hexadecimal.push('F'),
      _ => (),
    };
  }
  return hexadecimal;
}
