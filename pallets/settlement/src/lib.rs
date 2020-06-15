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
use polymesh_primitives::{
    traits::IdentityCurrency, AccountId, AccountKey, Beneficiary, IdentityId, Ticker,
};

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

type Identity<T> = identity::Module<T>;

pub trait Trait: frame_system::Trait + CommonTrait + BalancesTrait + IdentityTrait {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InstructionStatus {
    Unknown,
    PendingOrExpired,
    Executed,
    Failed,
    Rejected,
}

impl Default for InstructionStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LegStatus {
    ExecutionPending,
    ExecutionSuccessful,
    ExecutionFailed,
    /// receipt used
    ExecutionSkipped,
}

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorizationStatus {
    Unknown,
    Pending,
    Authorized,
    Rejected,
}

impl Default for AuthorizationStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Instruction<T> {
    instruction_id: u64,
    venue_id: u64,
    status: InstructionStatus,
    expiry: Option<T>,
    created_at: Option<T>,
    valid_from: Option<T>,
    auths_pending: u64,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct LegDetails<T> {
    from: Option<IdentityId>,
    to: Option<IdentityId>,
    asset: Ticker,
    amount: T,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Leg<T> {
    leg_id: u64,
    from: Option<IdentityId>,
    to: Option<IdentityId>,
    asset: Ticker,
    amount: T,
    status: LegStatus,
}

impl<T> Leg<T> {
    pub fn new(leg_id: u64, leg: LegDetails<T>) -> Self {
        Leg {
            leg_id,
            from: leg.from,
            to: leg.to,
            asset: leg.asset,
            amount: leg.amount,
            status: LegStatus::ExecutionPending,
        }
    }
}

#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Venue {
    creator: IdentityId,
    // instruction_id
    instructions: Vec<u64>,
    details: Vec<u8>,
}

impl Venue {
    pub fn new(creator: IdentityId, details: Vec<u8>) -> Self {
        Self {
            creator,
            instructions: Vec::new(),
            details,
        }
    }
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
        /// Venue does not exist
        InvalidVenue,
        /// Sender does not have required permissions
        Unauthorized
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        VenueInfo get(fn venue_info): map hasher(twox_64_concat) u64 => Venue;

        VenueSigners get(fn venue_signers): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) AccountId => bool;

        InstructionDetails get(fn instruction_details): map hasher(twox_64_concat) u64 => Instruction<T::Moment>;

        InstructionLegs get(fn instruction_legs): map hasher(twox_64_concat) u64 => Vec<Leg<T::Balance>>;

        AuthsReceived get(fn auths_received): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) IdentityId => AuthorizationStatus;

        UserAuths get(fn user_auths): double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) u64 => AuthorizationStatus;

        ReceiptsUsed get(fn receipts_used): double_map hasher(twox_64_concat) AccountId, hasher(blake2_128_concat) Receipt<T::Balance> => bool;

        VenueFiltering get(fn venue_filtering): map hasher(blake2_128_concat) Ticker => bool;

        VenueAllowList get(fn venue_allow_list): double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) u64 => bool;

        VenueCounter get(fn venue_counter) build(|_| 1u64): u64;

        InstructionCounter get(fn instruction_counter) build(|_| 1u64): u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        pub fn create_venue(origin, details: Vec<u8>, signers: Vec<AccountId>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let venue = Venue::new(did, details);
            let venue_counter = Self::venue_counter();
            <VenueInfo>::insert(venue_counter, venue);
            for signer in signers {
                <VenueSigners>::insert(venue_counter, signer, true);
            }
            <VenueCounter>::put(venue_counter + 1);
            Ok(())
        }

        pub fn add_instruction(
            origin,
            venue_id: u64,
            valid_from: T::Moment,
            expiry: Option<T::Moment>,
            leg_details: Vec<LegDetails<T::Balance>>
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // check if venue exists and sender has permissions
            ensure!(<VenueInfo>::contains_key(venue_id), Error::<T>::InvalidVenue);
            let mut venue = Self::venue_info(venue_id);
            ensure!(venue.creator == did, Error::<T>::Unauthorized);

            // Prepare data to store in storage
            let instruction_counter = Self::instruction_counter();
            let mut legs = Vec::with_capacity(leg_details.len());
            let mut counter_parties = Vec::with_capacity(leg_details.len() * 2);
            for i in 0..leg_details.len() {
                if let Some(from) = leg_details[i].from {
                    counter_parties.push(from);
                }
                if let Some(to) = leg_details[i].to {
                    counter_parties.push(to);
                }
                legs.push(Leg::new(u64::try_from(i).unwrap_or_default(), leg_details[i].clone()));
            }
            counter_parties.sort();
            counter_parties.dedup();
            venue.instructions.push(instruction_counter);
            let instruction = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_id,
                status: InstructionStatus::PendingOrExpired,
                expiry: expiry,
                created_at: Some(<pallet_timestamp::Module<T>>::get()),
                valid_from: Some(valid_from),
                auths_pending: u64::try_from(counter_parties.len()).unwrap_or_default(),
            };

            // write data to storage
            for counter_party in counter_parties {
                <UserAuths>::insert(counter_party, instruction_counter, AuthorizationStatus::Pending);
            }
            <InstructionLegs<T>>::insert(instruction_counter, legs);
            <InstructionDetails<T>>::insert(instruction_counter, instruction);
            <VenueInfo>::insert(venue_id, venue);
            <InstructionCounter>::put(instruction_counter + 1);

            Ok(())
        }

        pub fn authorize_instruction(origin, instruction_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

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
