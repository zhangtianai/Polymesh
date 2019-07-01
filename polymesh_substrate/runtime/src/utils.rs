use parity_codec::Codec;
use rstd::prelude::*;
use runtime_io::{secp256k1_ecdsa_recover, EcdsaVerifyError};
use runtime_primitives::traits::{As, Member, SimpleArithmetic};
use support::{decl_module, decl_storage, dispatch::Result, Parameter};
use system;

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

        let comp_token_pubkey = secp256k1_ecdsa_recover(&comp_token_array, &msg_hash_array)
            .map_err(|e| match e {
                EcdsaVerifyError::BadRS => "Invalid compliance token: bad r or s",
                EcdsaVerifyError::BadV => "Invalid compliance token: bad v",
                EcdsaVerifyError::BadSignature => "Invalid compliance token: bad signature",
            })?;

        assert_eq!(
            &comp_token_pubkey[..],
            &pubkey[1..],
            "Invalid comp token public key"
        );
        Ok(())
    }
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
