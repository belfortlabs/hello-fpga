use rand::Rng;
use std::time::Instant;
use tfhe::prelude::*;
use tfhe::set_server_key;
use tfhe::{ClientKey, ConfigBuilder, FheUint64};


fn main() {

    // Test data

    let mut rng = rand::thread_rng();

    let generate_value_weight = (rng.gen_range(1..=10), rng.gen_range(1..=10));

    let (value1, weight1) = generate_value_weight;
    let (value2, weight2) = generate_value_weight;
    let (value3, weight3) = generate_value_weight;

    let weighted_sum= value1 * weight1 + value2 * weight2 + value3 * weight3;

    // Create Keys

    let config = ConfigBuilder::default().build();
    let client_key = ClientKey::generate(config);
    let server_key = client_key.generate_server_key();



    set_server_key(server_key);

    // Encrypt Values

    let encrypt_value_weight = |v, w| (FheUint64::encrypt(v, &client_key), FheUint64::encrypt(w, &client_key));

    let (encrypted_value1, encrypted_weight1) = encrypt_value_weight(value1, weight1);
    let (encrypted_value2, encrypted_weight2) = encrypt_value_weight(value2, weight2);
    let (encrypted_value3, encrypted_weight3) = encrypt_value_weight(value3, weight3);

    // Encrypted Calculations

    let time_start = Instant::now();

    let encypted_weighted_sum = encrypted_value1 * encrypted_weight1
        + encrypted_value2 * encrypted_weight2
        + encrypted_value3 * encrypted_weight3;

    println!("Execution time {:?}", time_start.elapsed());

    let decrypted_weighted_sum: u64 = encypted_weighted_sum.decrypt(&client_key);
    assert_eq!(decrypted_weighted_sum, weighted_sum);



}
