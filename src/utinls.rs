

pub mod token_generator {
    use std::iter;
    use rand::{Rng, RngCore};
    use crate::token::*;




    fn generate_random_string(len: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz123456789";
        let mut rng = rand::rng();
        let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(len).collect()
    }

    pub fn generate_token() -> String {
        generate_random_string(20)
    }
}