use rand::Rng;
use std::time::Instant;
// Enable FPGA: Import the BelfortServerKey
#[cfg(feature = "fpga")]
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::prelude::*;
use tfhe::set_server_key;
use tfhe::{ClientKey, ConfigBuilder, FheUint64};

fn main() {
    // Initialize a logger for interfacing with runtime warnings
    env_logger::init();

    // Test data
    let mut rng = rand::rng();

    let generate_value_weight = (rng.random_range(1..=10), rng.random_range(1..=10));

    let (value1, weight1) = generate_value_weight;
    let (value2, weight2) = generate_value_weight;
    let (value3, weight3) = generate_value_weight;

    let weighted_sum = value1 * weight1 + value2 * weight2 + value3 * weight3;

    // Create Keys
    let config = ConfigBuilder::default().build();
    let client_key = ClientKey::generate(config);
    let server_key = client_key.generate_server_key();

    // Enable FPGA: Create FPGA key from your server and connect to it
    #[cfg(feature = "fpga")]
    let mut fpga_key = {
        let mut fpga_key = BelfortServerKey::from(&server_key);
        fpga_key.connect();
        set_server_key(fpga_key.clone());
        fpga_key
    };
    #[cfg(not(feature = "fpga"))]
    set_server_key(server_key);

    // Encrypt Values
    let encrypt_value_weight = |v, w| {
        (
            FheUint64::encrypt(v, &client_key),
            FheUint64::encrypt(w, &client_key),
        )
    };

    let (encrypted_value1, encrypted_weight1) = encrypt_value_weight(value1, weight1);
    let (encrypted_value2, encrypted_weight2) = encrypt_value_weight(value2, weight2);
    let (encrypted_value3, encrypted_weight3) = encrypt_value_weight(value3, weight3);

    // Encrypted Calculations

    let time_start = Instant::now();

    let encypted_weighted_sum = encrypted_value1 * encrypted_weight1
        + encrypted_value2 * encrypted_weight2
        + encrypted_value3 * encrypted_weight3;

    #[cfg(feature = "fpga")]
    println!("Execution time on FPGA: {:?}", time_start.elapsed());
    #[cfg(not(feature = "fpga"))]
    println!("Execution time on CPU: {:?}", time_start.elapsed());

    let decrypted_weighted_sum: u64 = encypted_weighted_sum.decrypt(&client_key);
    assert_eq!(decrypted_weighted_sum, weighted_sum);

    // Enable FPGA: Disconnect the BelfortServerKey
    #[cfg(feature = "fpga")]
    fpga_key.disconnect();
}
