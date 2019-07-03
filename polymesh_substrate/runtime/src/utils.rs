use parity_codec::Codec;
use rstd::prelude::*;
use runtime_io::{secp256k1_ecdsa_recover, EcdsaVerifyError};
use runtime_primitives::traits::{As, Member, SimpleArithmetic};
use support::{decl_module, decl_storage, dispatch::Result, ensure, Parameter};
use system;

use core::result::Result as StdResult;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
    type TokenBalance: Parameter
        + Member
        + SimpleArithmetic
        + Codec
        + Default
        + Copy
        + As<usize>
        + As<u64>
        + As<<Self as balances::Trait>::Balance>;
    fn as_u128(v: Self::TokenBalance) -> u128;
    fn as_tb(v: u128) -> Self::TokenBalance;
}

decl_storage! {
    trait Store for Module<T: Trait> as Utils {

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

    }
}

impl<T: Trait> Module<T> {
    // The token is expected to be an s-normalized RS signature (V is a constant 0x28), msg a 32-byte hash of the message
    // and pubkey an uncompressed form public key.
}

// Other utility functions
#[inline]
/// Convert all letter characters of a slice to their upper case counterparts.
pub fn bytes_to_upper(v: &[u8]) -> Vec<u8> {
    v.iter()
        .map(|chr| match chr {
            97..=122 => chr - 32,
            other => *other,
        })
        .collect()
}

/// Decodes a hex byte slice into the direct byte representation. Ignores case. Hex byte count
/// needs to be even. Does not take a "0x" prefix.
pub fn decode_hex_bytes(v: &[u8]) -> StdResult<Vec<u8>, &'static str> {
    if v.len() % 2 != 0 {
        return Err("Odd input length");
    }

    let decoded_nibbles: Vec<u8> = v
        .iter()
        .enumerate()
        .map(|(idx, hex_byte)| {
            let digit = match *hex_byte as char {
                '0'..='9' => hex_byte - '0' as u8,
                'a'..='f' => hex_byte - 'a' as u8 + 10,
                'A'..='F' => hex_byte - 'A' as u8 + 10,
                other => return Err("Non-hex byte encountered"),
            };

            Ok(if idx % 2 == 0 { digit * 16 } else { digit })
        })
        .collect::<StdResult<Vec<u8>, &'static str>>()?;

    Ok(decoded_nibbles
        .as_slice()
        .chunks(2)
        .map(|pair| pair[0] + pair[1])
        .collect())
}

pub fn verify_compliance_token(
    mut comp_token: Vec<u8>,
    msg_hash: Vec<u8>,
    pubkey: Vec<u8>,
) -> Result {
    // Add v for lower-half-normalized signature
    comp_token.push(28);

    let mut comp_token_array: [u8; 65] = [0u8; 65];
    let mut msg_hash_array: [u8; 32] = [0u8; 32];

    // Convert the comp token into an array
    comp_token_array.copy_from_slice(comp_token.as_slice());

    // Do the same for the message hash
    msg_hash_array.copy_from_slice(&msg_hash[..]);

    let comp_token_pubkey =
        secp256k1_ecdsa_recover(&comp_token_array, &msg_hash_array).map_err(|e| match e {
            EcdsaVerifyError::BadRS => "Invalid compliance token: bad r or s",
            EcdsaVerifyError::BadV => "Invalid compliance token: bad v",
            EcdsaVerifyError::BadSignature => "Invalid compliance token: bad signature",
        })?;

    ensure!(
        &comp_token_pubkey[..] == &pubkey[1..],
        "Invalid comp token public key"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_bytes() {
        let hex = "deadbeef1090".as_bytes();
        let expected = vec![0xde, 0xad, 0xbe, 0xef, 0x10, 0x90];

        assert_eq!(decode_hex_bytes(hex), Ok(expected));
    }
}
