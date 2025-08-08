mod hello_world_clean_xsmall_test_rs_lib;

use std::time::Instant;
use tfhe::prelude::*;
use tfhe::{set_server_key, ClientKey, ConfigBuilder, FheUint8};

#[cfg(feature = "fpga")]
use tfhe::integer::fpga::BelfortServerKey;

fn main() {
    // Create Keys
    let config = ConfigBuilder::default().build();

    let client_key = ClientKey::generate(config);
    let server_key = client_key.generate_server_key();

    #[cfg(feature = "fpga")]
    {
        let mut fpga_key = BelfortServerKey::from(&server_key);
        fpga_key.connect();
    }


    let input: u8 = 31;

    let a = FheUint8::encrypt(input, &client_key);
    let input_vec = core::array::from_fn(|_| core::array::from_fn(|_| a.clone()));

    set_server_key(server_key);

    let t = Instant::now();
    let result = hello_world_clean_xsmall_test_rs_lib::fn_under_test(&input_vec);
    let elapsed = t.elapsed();
    println!("Time elapsed: {:?}s", elapsed.as_secs_f32());

    let output: u16 = result[0][0].decrypt(&client_key);
    assert_eq!(output, input as u16 * 9 + 1);

    let output: u16 = result[0][1].decrypt(&client_key);
    assert_eq!(output, input as u16 * 54 + 2);

    let output: u16 = result[0][2].decrypt(&client_key);
    assert_eq!(output, input as u16 * 57 + 5438);

    println!("Successfully executed the Hello World Neural network!");
}
