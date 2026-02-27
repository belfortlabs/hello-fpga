/*
* MIT License
*
* Copyright (c) 2025 KU Leuven - COSIC
* Author: Wouter Legiest
*
* Permission is hereby granted, free of charge, to any person obtaining a copy
* of this software and associated documentation files (the "Software"), to deal
* in the Software without restriction, including without limitation the rights
* to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
* copies of the Software, and to permit persons to whom the Software is
* furnished to do so, subject to the following conditions:

* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software.

* THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
* IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
* FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
* AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
* LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
* OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
* SOFTWARE.
*/

use pad::PadStr;
use std::collections::HashMap;
use tfhe::shortint::prelude::*;

pub const LUT_SIZE: usize = 48;

pub const NAME_LIST: [&str; LUT_SIZE] = [
    "Wouter Legiest",
    "Bilbo Baggins",
    "Ingrid Verbauwhede",
    "Nam-Luc Tran",
    "Bojan Spasic",
    "Biff Tannen",
    "Hans Gruber",
    "Freddy Krueger",
    "Hannibal Lecter",
    "Indiana Jones",
    "Vito Corleone",
    "Marty McFly",
    "Peter Venkman",
    "Egon Spengler",
    "Winston Zeddemore",
    "Lorraine Baines",
    "Ray Stantz",
    "Ellen Ripley",
    "John McClane",
    "Tony Stark",
    "Peter Parker",
    "Steve Rogers",
    "Thor Odinson",
    "Natasha Romanoff",
    "Clint Barton",
    "Nick Fury",
    "Phil Coulson",
    "Rocky Balboa",
    "Bilbo Baggins",
    "Tony Montana",
    "Samwise Gamgee",
    "Ethan Hunt",
    "James Bond",
    "Bruce Wayne",
    "Clark Kent",
    "Diana Prince",
    "Arthur Curry",
    "Barry Allen",
    "Hal Jordan",
    "Logan Howlett",
    "Charles Xavier",
    "Jean Grey",
    "Erik Lehnsherr",
    "Stephen Strange",
    "Bruce Banner",
    "Peggy Carter",
    "Norman Bates",
    "Rick Deckard",
];

pub fn process_db(
    cks: &ClientKey,
    max_factor: usize,
) -> HashMap<usize, HashMap<char, Vec<Ciphertext>>> {
    let db_size = NAME_LIST.len();

    let ascii_collection = (20..126u8).collect::<Vec<u8>>();

    let mut db_processed: HashMap<usize, HashMap<char, Vec<Ciphertext>>> = HashMap::new();

    for k in 0..db_size {
        let word_len = NAME_LIST[k].pad_to_width(max_factor - 1);
        let m = word_len.len();
        let mut peq = HashMap::new();

        for i in &ascii_collection {
            let s = *i as char;
            let bitvec: Vec<u8> = (0..m)
                .map(|j| {
                    if word_len.chars().nth(j).unwrap() == s {
                        9
                    } else {
                        0
                    }
                })
                .collect();

            let vec_enc = bitvec
                .iter()
                .map(|c| cks.encrypt(*c as u64))
                .collect::<Vec<Ciphertext>>();

            peq.insert(s, vec_enc);
        }
        db_processed.insert(k, peq);
    }

    db_processed
}
