// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Settlement Module
//!
//! Settlement module manages all kinds of transfers and settlements of assets
//!
//! ## Overview
//!
//! TODO
//!
//! ## Dispatchable Functions
//!
//! TODO
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

use pallet_identity as identity;
use polymesh_common_utilities::{
    constants::SETTLEMENT_MODULE_ID,
    traits::{balances::Trait as BalancesTrait, identity::Trait as IdentityTrait, CommonTrait},
    Context, SystematicIssuers,
};
use polymesh_primitives::{traits::IdentityCurrency, AccountKey, Beneficiary, IdentityId, Ticker};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, WithdrawReason},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_runtime::traits::{AccountIdConversion, Saturating};
use sp_std::{convert::TryFrom, prelude::*};

pub trait Trait: frame_system::Trait + CommonTrait + BalancesTrait + IdentityTrait {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionStatus<T> {
    PendingOrExpired,
    Executed(T),
    Rejected(IdentityId),
    // leg id
    Failed(u64),
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorizationStatus {
    Pending,
    Authorized,
    Rejected,
}

impl Default for AuthorizationStatus {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Instruction<T> {
    instruction_id: u64,
    venue_id: u64,
    status: InstructionStatus<T>,
    expiry: Option<T>,
    created_at: T,
    valid_from: T,
    auths_pending: u64,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Leg<T> {
    leg_id: u64,
    from: Option<IdentityId>,
    to: Option<IdentityId>,
    asset: Ticker,
    amount: T,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Venue {
    creator: IdentityId,
    // instruction_id
    instructions: Vec<u64>,
    details: Vec<u8>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Receipt<T> {
    receipt_uid: u64, //anything unique per signer
    from: IdentityId,
    to: IdentityId,
    amount: T,
    asset: Ticker,
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        /// Disbursement to a target Identity.
        /// (target identity, amount)
        TreasuryDisbursement(IdentityId, IdentityId, Balance),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Proposer's balance is too low.
        InsufficientBalance,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        VenueInfo get(fn venue_info): map hasher(twox_64_concat) u64 => Venue;

        VenueSigners get(fn venue_signers): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) IdentityId => bool;

        InstructionLegs get(fn instruction_legs): map hasher(twox_64_concat) u64 => Vec<Leg<T::Balance>>;

        AuthsReceived get(fn auths_received): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) IdentityId => AuthorizationStatus;

        UserAuths get(fn user_auths): double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) u64 => AuthorizationStatus;

        ReceiptsUsed get(fn receipts_used): double_map hasher(twox_64_concat) IdentityId, hasher(blake2_128_concat) Receipt<T::Balance> => bool;

        VenueFiltering get(fn venue_filtering): map hasher(blake2_128_concat) Ticker => bool;

        VenueAllowList get(fn venue_allow_list): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => bool;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        pub fn disbursement(origin) -> DispatchResult {
            // let sender = ensure_signed(origin)?;
            // let sender_key = AccountKey::try_from(sender.encode())?;
            // let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            Ok(())
        }

    }
}

impl<T: Trait> Module<T> {
    /// The account ID of the settlement module.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    pub fn account_id() -> T::AccountId {
        SETTLEMENT_MODULE_ID.into_account()
    }
}
