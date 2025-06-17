mod static_deque;

mod trivium;
pub use trivium::{
    TriviumStream, TriviumStreamByte, TriviumStreamFPGAByte, TriviumStreamFPGAShortint,
    TriviumStreamShortint,
};

mod trans_ciphering;
pub use trans_ciphering::TransCiphering;
