//! # Simple Module for testing

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, LockableCurrency, ReservableCurrency},
    weights::GetDispatchInfo,
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use pallet_mips_rpc_runtime_api::VoteCount;
use polymesh_primitives::AccountKey;
use polymesh_runtime_common::{
    identity::Trait as IdentityTrait, traits::group::GroupTrait, Context,
};
use polymesh_runtime_identity as identity;
use sp_runtime::{
    traits::{Dispatchable, EnsureOrigin, Hash, Zero},
    DispatchError,
};
use sp_std::{convert::TryFrom, prelude::*, vec};

/// A wrapper for a proposal url.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for Url {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        Url(v)
    }
}

/// A wrapper for a proposal description.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Description(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for Description {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        Description(v)
    }
}

/// Represents a proposal metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct Metadata<AcccountId: Parameter, Hash: Parameter> {
    /// The creator
    proposer: AcccountId,
    /// The proposal being voted on.
    proposal_hash: Hash,
    /// The proposal url for proposal discussion.
    url: Option<Url>,
    /// The proposal description.
    description: Option<Description>,
}

/// The module's configuration trait.
pub trait Trait: frame_system::Trait {
    /// An extrinsic call.
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin> + GetDispatchInfo;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Mips {
        /// The hashes of the active proposals.
        pub ProposalMetadata get(fn proposal_meta): Vec<Metadata<T::AccountId, T::Hash>>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        <T as frame_system::Trait>::Hash,
        <T as frame_system::Trait>::AccountId,
    {
        Proposed(AccountId, Hash),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Incorrect origin
        BadOrigin,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        pub fn propose(
            origin,
            proposal: Box<T::Proposal>,
            url: Option<Url>,
            description: Option<Description>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let proposal_hash = T::Hashing::hash_of(&proposal);

            let proposal_meta = Metadata {
                proposer: proposer.clone(),
                proposal_hash,
                url,
                description,
            };
            <ProposalMetadata<T>>::mutate(|metadata| metadata.push(proposal_meta));

            Self::deposit_event(RawEvent::Proposed(proposer, proposal_hash));
            Ok(())
        }
    }
}
