use std::io;
use std::io::Write;
use std::ops::Add;
use std::time::{Duration, Instant};
use tfhe::prelude::*;
use tfhe::{generate_keys, ConfigBuilder, FheUint8};

#[cfg(not(feature = "fpga"))]
use tfhe_trivium::TriviumStreamByte;
#[cfg(feature = "fpga")]
use tfhe_trivium::TriviumStreamFPGAByte;

fn main() {
    let config = ConfigBuilder::default().build();
    let (client_key, server_key) = generate_keys(config);

    let key_string = "0053A6F94C9FF24598EB".to_string();
    let mut key = [0u8; 10];

    for i in (0..key_string.len()).step_by(2) {
        key[i >> 1] = u8::from_str_radix(&key_string[i..i + 2], 16).unwrap();
    }

    let iv_string = "0D74DB42A91077DE45AC".to_string();
    let mut iv = [0u8; 10];

    for i in (0..iv_string.len()).step_by(2) {
        iv[i >> 1] = u8::from_str_radix(&iv_string[i..i + 2], 16).unwrap();
    }

    let cipher_key = key.map(|x| FheUint8::encrypt(x, &client_key));

    println!("Initialization step starts (~1 min)");

    let start = Instant::now();
    #[cfg(feature = "fpga")]
    let mut trivium = TriviumStreamFPGAByte::<FheUint8>::new(cipher_key, iv, &server_key);
    #[cfg(not(feature = "fpga"))]
    let mut trivium = TriviumStreamByte::<FheUint8>::new(cipher_key, iv, &server_key);

    println!("Initialization took {:?}\n", start.elapsed());

    for i in 0..128 {
        print!("\x1b[90m{:5}\x1b[0m ", i);

        let mut elapsed = Duration::from_secs(0);

        let start = Instant::now();
        let cipher_outputs = trivium.next_64();
        elapsed = elapsed.add(start.elapsed());

        let decrypted_bits: Vec<u8> = cipher_outputs
            .iter()
            .map(|item| item.decrypt(&client_key))
            .collect();

        let hexstring = get_hexagonal_string_from_bytes(decrypted_bits);
        print!("{}", hexstring);
        io::stdout().flush().unwrap();

        print!(" \x1b[90m{:?}\x1b[0m\n", elapsed);
    }
    println!("The end!");
}

fn get_hexagonal_string_from_bytes(a: Vec<u8>) -> String {
    assert!(a.len() % 8 == 0);
    let mut hexadecimal: String = "".to_string();
    for test in a {
        hexadecimal.push_str(&format!("{:02X?}", test));
    }
    hexadecimal
}
