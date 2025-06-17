mod trivium_bool;
pub use trivium_bool::TriviumStream;

mod trivium_byte;
pub use trivium_byte::TriviumStreamByte;

mod trivium_shortint;
pub use trivium_shortint::TriviumStreamShortint;

mod trivium_shortint_fpga;
pub use trivium_shortint_fpga::TriviumStreamFPGAShortint;

mod trivium_byte_fpga;
pub use trivium_byte_fpga::TriviumStreamFPGAByte;

#[cfg(test)]
mod test;
