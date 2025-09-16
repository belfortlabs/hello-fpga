pub mod inv_round_ops;
pub mod key_expansion;
pub mod round_ops;
pub mod state;
pub mod threading;
pub mod tower_field;

use tfhe::integer::fpga::BelfortServerKey;
use tower_field::FheAesTowerField;

use super::lookup::FheAesLookup;

pub struct FheAesEngine<'a> {
    fpga_key: &'a BelfortServerKey,
    lookup: FheAesLookup<'a>,
    tower_field: FheAesTowerField<'a>,
}

impl<'a> FheAesEngine<'a> {
    pub fn new(fpga_key: &'a BelfortServerKey) -> Self {
        let lookup = FheAesLookup::new(fpga_key);
        let tower_field = FheAesTowerField::new(fpga_key);

        Self {
            fpga_key,
            lookup,
            tower_field,
        }
    }
}
