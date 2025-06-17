//! This module extends the Trivium stream cipher implementation for FPGA's.

use crate::static_deque::StaticByteDeque;
use tfhe::{prelude::*, unset_server_key, BelfortServerKey};
use tfhe::{set_server_key, FheUint8, ServerKey};

use super::trivium_byte::TriviumByteInput;

/// TriviumStreamByte: a struct implementing the Trivium stream cipher, using T for the internal
/// representation of bits (u8 or FheUint8). It owns the BelfortServerKey to compute FHE operations.
/// Since the original Trivium registers' sizes are not a multiple of 8, these registers (which
/// store byte-like objects) have a size that is the eighth of the closest multiple of 8 above the
/// originals' sizes.
pub struct TriviumStreamFPGAByte<T> {
    a_byte: StaticByteDeque<12, T>,
    b_byte: StaticByteDeque<11, T>,
    c_byte: StaticByteDeque<14, T>,
    fpga_key: BelfortServerKey,
}

impl TriviumStreamFPGAByte<FheUint8> {
    /// Constructor for `TriviumStream<FheUint8>`: arguments are the encrypted secret key, the input
    /// vector, and the FHE server key.
    /// Outputs an initialized TriviumStream object (1152 steps have run)
    pub fn new(key: [FheUint8; 10], iv: [u8; 10], server_key: &ServerKey) -> Self {
        // Initialize Belfort server key
        let mut fpga_key = BelfortServerKey::from(server_key);
        fpga_key.connect();
        set_server_key(fpga_key.clone());

        // Initialization of Trivium registers: a has the secret key, b the input vector,
        // and c a few ones.
        let mut a_byte_reg = [0u8; 12].map(FheUint8::encrypt_trivial);
        let mut b_byte_reg = [0u8; 11].map(FheUint8::encrypt_trivial);
        let mut c_byte_reg = [0u8; 14].map(FheUint8::encrypt_trivial);

        for i in 0..10 {
            a_byte_reg[12 - 10 + i] = key[i].clone();
            b_byte_reg[11 - 10 + i] = FheUint8::encrypt_trivial(iv[i]);
        }

        // Magic number 14, aka 00001110: this represents the 3 ones at the beginning of the c
        // registers, with additional zeros to make the register's size a multiple of 8.
        c_byte_reg[0] = FheUint8::encrypt_trivial(14u8);

        let mut ret = TriviumStreamFPGAByte::<FheUint8>::new_from_registers(
            a_byte_reg, b_byte_reg, c_byte_reg, fpga_key,
        );
        ret.init();
        ret
    }
}

impl<T> TriviumStreamFPGAByte<T>
where
    T: TriviumByteInput<T> + Send,
    for<'a> &'a T: TriviumByteInput<T>,
{
    /// Internal generic constructor: arguments are already prepared registers, and an optional FHE
    /// server key
    fn new_from_registers(
        a_register: [T; 12],
        b_register: [T; 11],
        c_register: [T; 14],
        fpga_key: BelfortServerKey,
    ) -> Self {
        Self {
            a_byte: StaticByteDeque::<12, T>::new(a_register),
            b_byte: StaticByteDeque::<11, T>::new(b_register),
            c_byte: StaticByteDeque::<14, T>::new(c_register),
            fpga_key,
        }
    }

    /// The specification of Trivium includes running 1152 (= 18*64) unused steps to mix up the
    /// registers, before starting the proper stream
    fn init(&mut self) {
        for _ in 0..18 {
            self.next_64();
        }
    }

    /// Computes 8 potential future step of Trivium, b*8 terms in the future. This does not update
    /// registers, but returns the three values used to update the registers, when the time is right.
    fn get_output_and_values(&self, b: usize) -> [T; 4] {
        let n = b * 8 + 7;
        assert!(n < 65);

        let (a1, a2, a3, a4, a5) =
            Self::get_bytes(&self.a_byte, [91 - n, 90 - n, 68 - n, 65 - n, 92 - n]);
        let (b1, b2, b3, b4, b5) =
            Self::get_bytes(&self.b_byte, [82 - n, 81 - n, 77 - n, 68 - n, 83 - n]);
        let (c1, c2, c3, c4, c5) =
            Self::get_bytes(&self.c_byte, [109 - n, 108 - n, 86 - n, 65 - n, 110 - n]);

        let (temp_c, a_and) = (c4 ^ c5, a1 & a2);
        let (temp_a, temp_b) = (a4 ^ a5, b4 ^ b5);
        let (b_and, c_and) = (b1 & b2, c1 & c2);

        let (temp_a_2, temp_b_2, temp_c_2) = (temp_a.clone(), temp_b.clone(), temp_c.clone());

        let (o, a) = ((temp_a_2 ^ temp_b_2) ^ temp_c_2, temp_c ^ ((c_and) ^ a3));
        let (b, c) = (temp_a ^ (a_and ^ b3), temp_b ^ (b_and ^ c3));

        [o, a, b, c]
    }

    /// This calls `get_output_and_values` 8 times, and stores all results in a Vec.
    fn get_64_output_and_values(&self) -> Vec<[T; 4]> {
        (0..8)
            .into_iter()
            .map(|i| self.get_output_and_values(i))
            .collect()
    }

    /// Computes 64 turns of the stream, outputting the 64 bits (in 8 bytes) all at once in a
    /// Vec (first value is oldest, last is newest)
    pub fn next_64(&mut self) -> Vec<T> {
        let values = self.get_64_output_and_values();
        let mut bytes = Vec::<T>::with_capacity(8);
        for [o, a, b, c] in values {
            self.a_byte.push(a);
            self.b_byte.push(b);
            self.c_byte.push(c);
            bytes.push(o);
        }

        bytes
    }

    /// Reconstructs a bunch of 5 bytes.
    fn get_bytes<const N: usize>(
        reg: &StaticByteDeque<N, T>,
        offsets: [usize; 5],
    ) -> (T, T, T, T, T) {
        let mut ret = offsets
            .iter()
            .rev()
            .map(|&i| reg.byte(i))
            .collect::<Vec<_>>();
        (
            ret.pop().unwrap(),
            ret.pop().unwrap(),
            ret.pop().unwrap(),
            ret.pop().unwrap(),
            ret.pop().unwrap(),
        )
    }
}

impl<T> Drop for TriviumStreamFPGAByte<T> {
    fn drop(&mut self) {
        self.fpga_key.disconnect();
        unset_server_key();
    }
}
