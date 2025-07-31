use std::collections::BTreeMap;
use tfhe::prelude::*;
use tfhe::FheUint16;

type Ciphertext = tfhe::FheUint<tfhe::FheUint8Id>;

// Code produced by the HEIR compiler
pub fn fn_under_test(v1: &[[Ciphertext; 1]; 1]) -> [[tfhe::FheUint<tfhe::FheUint16Id>; 3]; 1] {
    let v2 = 0usize;
    static V3: [u16; 3] = [1, 2, 5438];
    static V4: [u16; 3] = [9, 54, 57];
    let mut v5: BTreeMap<(usize, usize), tfhe::FheUint<tfhe::FheUint16Id>> = BTreeMap::new();
    for v6 in 0..3 {
        let v7 = V3[0 + v2 * 3 + v6 * 1];
        let v8 = &v1[v2][v2];
        let v9 = V4[0 + v2 * 3 + v6 * 1];
        let v10 = FheUint16::cast_from(v8.clone());
        let v11 = &v10 * v9;
        let v12 = &v11 + v7;
        v5.insert((v2 as usize, v6 as usize), v12.clone());
    }
    core::array::from_fn(|i0| core::array::from_fn(|i1| v5.get(&(i0, i1)).unwrap().clone()))
}
