use parity_codec::Codec;
use rstd::prelude::*;
use runtime_primitives::traits::{As, Member, SimpleArithmetic};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, Parameter};
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
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    fn as_u128(v: Self::TokenBalance) -> u128;
    fn as_tb(v: u128) -> Self::TokenBalance;
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event<T>() = default;

        pub fn verify_signed_checksum(checksum: Vec<u8>, signature: Vec<u8>, generator_point: Vec<u8>) -> Result {

            Ok(())
        }

    }
}

decl_event!(
    pub enum Event<T>
    where
        // Macro will fail without at least one type definition
        AccountId = <T as system::Trait>::AccountId,
    {
        // signature, checksum, verification key
        SignatureValid(Vec<u8>, Vec<u8>, Vec<u8>),
        // signature, checksum, verification key
        SignatureInvalid(Vec<u8>, Vec<u8>, Vec<u8>),

        // There's a weird error about the type definition being unused
        Dummy(AccountId),
    }
);

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
