//! This module implements the Trivium stream cipher, using booleans or FheBool
//! for the representaion of the inner bits.

use tfhe::core_crypto::fpga::lookup_vector::LookupVector;
use tfhe::integer::fpga::BelfortServerKey;
use tfhe::shortint::prelude::*;

use crate::static_deque::StaticDeque;

/// TriviumStreamFPGAShortint: a struct implementing the Trivium stream cipher, using T
/// for the internal representation of bits (bool or FheBool). To be able to
/// compute FHE operations, it also owns an Option for a ServerKey.
pub struct TriviumStreamFPGAShortint<Ciphertext> {
    a: StaticDeque<93, Ciphertext>,
    b: StaticDeque<84, Ciphertext>,
    c: StaticDeque<111, Ciphertext>,
    sk: ServerKey,
    fk: BelfortServerKey,
}

impl TriviumStreamFPGAShortint<Ciphertext> {
    /// Constructor for TriviumStreamFPGAShortint<Ciphertext>: arguments are the encrypted
    /// secret key and input vector, and the FHE server key.
    /// Outputs a TriviumStreamFPGAShortint object already initialized (1152 steps have
    /// been run before returning)
    pub fn new(
        key: [Ciphertext; 80],
        iv: [u64; 80],
        sk: ServerKey,
        fk: BelfortServerKey,
    ) -> TriviumStreamFPGAShortint<Ciphertext> {
        // Initialization of Trivium registers:
        // a has the secret key, b the input vector, and c a few ones.

        let mut a_register = [0; 93].map(|x| sk.create_trivial(x));
        let mut b_register = [0; 84].map(|x| sk.create_trivial(x));
        let mut c_register = [0; 111].map(|x| sk.create_trivial(x));

        for i in 0..80 {
            a_register[93 - 80 + i] = key[i].clone();
            b_register[84 - 80 + i] = sk.create_trivial(iv[i]);
        }

        c_register[0] = sk.create_trivial(1u64);
        c_register[1] = sk.create_trivial(1u64);
        c_register[2] = sk.create_trivial(1u64);

        let mut ret = Self {
            a: StaticDeque::<93, Ciphertext>::new(a_register),
            b: StaticDeque::<84, Ciphertext>::new(b_register),
            c: StaticDeque::<111, Ciphertext>::new(c_register),
            sk,
            fk,
        };

        ret.fk.connect();
        ret
    }

    pub fn init(&mut self, n: usize) {
        use std::io;
        use std::io::Write;

        let loopcount = 18 * 64 / n;

        print!("\r\x1b[90m      Initialising:\x1b[0m {:2}/{}", 0, loopcount);
        io::stdout().flush().unwrap();

        for i in 0..loopcount {
            let values: Vec<Ciphertext> = self.get_n_output_and_values(n);

            for abc in values[..(n * 3)].chunks(3) {
                self.a.push(abc[0].clone());
                self.b.push(abc[1].clone());
                self.c.push(abc[2].clone());
            }

            print!("\r\x1b[90m      Initialising:\x1b[0m {:3}/{}", i, loopcount);
            io::stdout().flush().unwrap();
        }

        println!("\r      Initialised.         ");
    }

    pub fn next_64(&mut self) -> Vec<Ciphertext> {
        self.next_n(64)
    }

    pub fn next_n(&mut self, n: usize) -> Vec<Ciphertext> {
        assert!(n < 65);

        let values: Vec<Ciphertext> = self.get_n_output_and_values(n);

        for abc in values[..n * 3].chunks(3) {
            self.a.push(abc[0].clone());
            self.b.push(abc[1].clone());
            self.c.push(abc[2].clone());
        }

        let mut ret: Vec<Ciphertext> = Vec::<Ciphertext>::with_capacity(n);
        for o in values[n * 3..].iter() {
            ret.push(o.clone());
        }
        ret
    }

    fn pack_block_assign(&self, low: &Ciphertext, high: &Ciphertext) -> Ciphertext {
        let mut new_high = self
            .sk
            .unchecked_scalar_mul(high, high.message_modulus.0 as u8);
        self.sk.unchecked_add_assign(&mut new_high, &low);
        new_high
    }

    fn get_n_output_and_values(&self, n: usize) -> Vec<Ciphertext> {
        // We have three packs.
        // The outputs of a pack is required for the latter one.
        // Hence, the outputs are fetched before moving to the second pack.
        //
        // Depending on n and batch_size, FPGA can calculate a pack in multiple
        // pipelined batches.

        let func_and = |x, y| (x & y);
        let lut_and = self.sk.generate_lookup_vector_bivariate(&func_and);

        let func_xor = |x, y| (x ^ y);
        let lut_xor = self.sk.generate_lookup_vector_bivariate(&func_xor);

        ////////////////////////////////////////////////////////////////////////
        // Pack 1

        let mut pack1: Vec<Ciphertext> = Vec::with_capacity(n * 3 * 2);
        let mut pack1_g: Vec<LookupVector> = Vec::with_capacity(n * 3 * 2);

        for i in 0..n {
            pack1.push(self.pack_block_assign(&self.a[91 - i], &self.a[90 - i]));
            pack1.push(self.pack_block_assign(&self.b[82 - i], &self.b[81 - i]));
            pack1.push(self.pack_block_assign(&self.c[109 - i], &self.c[108 - i]));
        }

        for _ in 0..(n * 3) {
            pack1_g.push(lut_and);
        }

        for i in 0..n {
            pack1.push(self.pack_block_assign(&self.a[65 - i], &self.a[92 - i]));
            pack1.push(self.pack_block_assign(&self.b[68 - i], &self.b[83 - i]));
            pack1.push(self.pack_block_assign(&self.c[65 - i], &self.c[110 - i]));
        }

        for _ in 0..(n * 3) {
            pack1_g.push(lut_xor);
        }

        self.fk
            .apply_lookup_vector_packed_assign(&mut pack1, &pack1_g);

        ////////////////////////////////////////////////////////////////////////
        // Pack 2

        let mut pack2: Vec<Ciphertext> = Vec::with_capacity(n * 3 + n);
        let mut pack2_g: Vec<LookupVector> = Vec::with_capacity(n * 3 + n);

        for i in 0..n {
            pack2.push(self.pack_block_assign(&pack1[i * 3 + 2], &self.a[68 - i]));
            pack2.push(self.pack_block_assign(&pack1[i * 3 + 0], &self.b[77 - i]));
            pack2.push(self.pack_block_assign(&pack1[i * 3 + 1], &self.c[86 - i]));
        }

        for i in 0..n {
            pack2.push(self.pack_block_assign(&pack1[n * 3 + i * 3], &pack1[n * 3 + i * 3 + 1]));
        }

        for _ in 0..(n * 3 + n) {
            pack2_g.push(lut_xor);
        }

        self.fk
            .apply_lookup_vector_packed_assign(&mut pack2, &pack2_g);

        ////////////////////////////////////////////////////////////////////////
        // Pack 3

        let mut pack3: Vec<Ciphertext> = Vec::with_capacity(n * 3 + n);
        let mut pack3_g: Vec<LookupVector> = Vec::with_capacity(n * 3 + n);

        for i in 0..n {
            pack3.push(self.pack_block_assign(&pack1[n * 3 + i * 3 + 2], &pack2[i * 3 + 0]));
            pack3.push(self.pack_block_assign(&pack1[n * 3 + i * 3 + 0], &pack2[i * 3 + 1]));
            pack3.push(self.pack_block_assign(&pack1[n * 3 + i * 3 + 1], &pack2[i * 3 + 2]));
        }

        for i in 0..n {
            pack3.push(self.pack_block_assign(&pack1[n * 3 + i * 3 + 2], &pack2[n * 3 + i]));
        }

        for _ in 0..(n * 3 + n) {
            pack3_g.push(lut_xor);
        }

        self.fk
            .apply_lookup_vector_packed_assign(&mut pack3, &pack3_g);

        pack3
    }
}

impl<T> Drop for TriviumStreamFPGAShortint<T> {
    fn drop(&mut self) {
        self.fk.disconnect();
    }
}
