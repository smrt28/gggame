

pub mod token_generator {
    use std::iter;
    use rand::{Rng, RngCore};

    pub const TOKEN_LENGTH: usize = 20;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub enum TokenType {
        Answer,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
    struct Token {
        token_type: TokenType,
        token: [u8; TOKEN_LENGTH],
    }


    impl Token {
        fn random_bytes() -> [u8; TOKEN_LENGTH] {
            const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz123456789";
            let mut rng = rand::rng();

            let mut buf = [0u8; TOKEN_LENGTH];
            for b in &mut buf {
                *b = CHARSET[rng.random_range(0..CHARSET.len())];
            }
            buf
        }

        pub fn new(token_type: TokenType) -> Self {
            Self {
                token_type: token_type,
                token: Self::random_bytes()
            }
        }
    }


    fn generate_random_string(len: usize) -> String {
        const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz123456789";
        let mut rng = rand::rng();
        let one_char = || CHARSET[rng.random_range(0..CHARSET.len())] as char;
        iter::repeat_with(one_char).take(len).collect()
    }

    pub fn generate_token(t: TokenType) -> String {
        let token = generate_random_string(20);

        let prefix = match t {
            TokenType::Answer => "a",
        };

        format!("{}0{}", prefix, token)
    }
}