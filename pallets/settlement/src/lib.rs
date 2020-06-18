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
    traits::{asset::Trait as AssetTrait, identity::Trait as IdentityTrait, CommonTrait},
    Context,
    SystematicIssuers::Settlement as SettlementDID,
};
use polymesh_primitives::{AccountId, AccountKey, IdentityId, Ticker};

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::AccountIdConversion;
use sp_std::{convert::TryFrom, prelude::*};

type Identity<T> = identity::Module<T>;

pub trait Trait:
    frame_system::Trait + CommonTrait + IdentityTrait + pallet_timestamp::Trait
{
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// Asset module
    type Asset: AssetTrait<Self::Balance, Self::AccountId>;
}

// TODO: add comments and tests
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
    /// receipt used but not executed yet, (receipt signer, receipt uid)
    ExecutionSkipped(AccountId, u64),
    /// receipt used, (receipt signer, receipt uid)
    ExecutionToBeSkipped(AccountId, u64),
}

impl Default for LegStatus {
    fn default() -> Self {
        Self::ExecutionPending
    }
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

#[derive(Encode, Decode, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum SettlementType<T> {
    SettleOnAuthorization,
    SettleOnDate(T),
}

impl<T> Default for SettlementType<T> {
    fn default() -> Self {
        Self::SettleOnAuthorization
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Instruction<T> {
    instruction_id: u64,
    venue_id: u64,
    status: InstructionStatus,
    settlement_type: SettlementType<T>,
    created_at: Option<T>,
    valid_from: Option<T>,
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct LegDetails<T> {
    from: IdentityId,
    to: IdentityId,
    asset: Ticker,
    amount: T,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct Leg<T> {
    leg_number: u64,
    from: IdentityId,
    to: IdentityId,
    asset: Ticker,
    amount: T,
}

impl<T> Leg<T> {
    pub fn new(leg_number: u64, leg: LegDetails<T>) -> Self {
        Leg {
            leg_number,
            from: leg.from,
            to: leg.to,
            asset: leg.asset,
            amount: leg.amount,
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
        Moment = <T as pallet_timestamp::Trait>::Moment,
    {
        /// A new venue has been created (did, venue_id)
        VenueCreated(IdentityId, u64),
        /// A new instruction has been created
        /// (did, venue_id, instruction_id, settlement_type, valid_from, legs)
        InstructionCreated(
            IdentityId,
            u64,
            u64,
            SettlementType<Moment>,
            Option<Moment>,
            Vec<Leg<Balance>>,
        ),
        /// An instruction has been authorized (did, instruction_id)
        InstructionAuthorized(IdentityId, u64),
        /// An instruction has been unauthorized (did, instruction_id)
        InstructionUnauthorized(IdentityId, u64),
        /// A receipt has been claimed (did, instruction_id, leg_number, receipt_uid, signer)
        ReceiptClaimed(IdentityId, u64, u64, u64, AccountId),
        /// A receipt has been unclaimed (did, instruction_id, leg_number, receipt_uid, signer)
        ReceiptUnclaimed(IdentityId, u64, u64, u64, AccountId),
        /// Venue filtering has been enabled or disabled for a ticker (did, ticker, filtering_enabled)
        VenueFiltering(IdentityId, Ticker, bool),
        /// Venues added to allow list (did, ticker, vec<venue_id>)
        VenuesAllowed(IdentityId, Ticker, Vec<u64>),
        /// Venues added to block list (did, ticker, vec<venue_id>)
        VenuesBlocked(IdentityId, Ticker, Vec<u64>),
    }
);

decl_error! {
    /// Errors for the Settlement module.
    pub enum Error for Module<T: Trait> {
        /// Venue does not exist
        InvalidVenue,
        /// Sender does not have required permissions
        Unauthorized,
        /// No pending authorization for the provided instruction
        NoPendingAuth,
        /// Instruction has not been authorized
        InstructionNotAuthorized,
        /// Provided leg is not pending execution
        LegNotPending,
        /// Signer is not authorized by the venue
        UnauthorizedSigner,
        /// Receipt already used
        ReceiptAlreadyClaimed,
        /// Receipt not used yet
        ReceiptNotClaimed,
        /// Venue does not have required permissions
        UnauthorizedVenue
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        VenueInfo get(fn venue_info): map hasher(twox_64_concat) u64 => Venue;

        VenueSigners get(fn venue_signers): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) AccountId => bool;

        InstructionDetails get(fn instruction_details): map hasher(twox_64_concat) u64 => Instruction<T::Moment>;

        InstructionLegs get(fn instruction_legs): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) u64 => Leg<T::Balance>;

        InstructionLegStatus get(fn instruction_leg_status): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) u64 => LegStatus;

        InstructionAuthsPending get(fn instruction_auths_pending): map hasher(twox_64_concat) u64 => u64;

        AuthsReceived get(fn auths_received): double_map hasher(twox_64_concat) u64, hasher(twox_64_concat) IdentityId => AuthorizationStatus;

        UserAuths get(fn user_auths): double_map hasher(twox_64_concat) IdentityId, hasher(twox_64_concat) u64 => AuthorizationStatus;

        ReceiptsUsed get(fn receipts_used): double_map hasher(twox_64_concat) AccountId, hasher(blake2_128_concat) u64 => bool;

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
            Self::deposit_event(RawEvent::VenueCreated(did, venue_counter));
            Ok(())
        }

        pub fn add_instruction(
            origin,
            venue_id: u64,
            settlement_type: SettlementType<T::Moment>,
            valid_from: Option<T::Moment>,
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
            let mut tickers = Vec::with_capacity(leg_details.len());
            for i in 0..leg_details.len() {
                counter_parties.push(leg_details[i].from);
                counter_parties.push(leg_details[i].to);
                tickers.push(leg_details[i].asset);
                legs.push(Leg::new(u64::try_from(i).unwrap_or_default(), leg_details[i].clone()));
            }

            // Check if venue has required permissions from token owners
            tickers.sort();
            tickers.dedup();
            for ticker in &tickers {
                if Self::venue_filtering(ticker) {
                    ensure!(Self::venue_allow_list(ticker, venue_id), Error::<T>::UnauthorizedVenue);
                }
            }

            counter_parties.sort();
            counter_parties.dedup();
            venue.instructions.push(instruction_counter);
            let instruction = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_id,
                status: InstructionStatus::PendingOrExpired,
                settlement_type: settlement_type,
                created_at: Some(<pallet_timestamp::Module<T>>::get()),
                valid_from: valid_from
            };

            // write data to storage
            for counter_party in &counter_parties {
                <UserAuths>::insert(counter_party, instruction_counter, AuthorizationStatus::Pending);
            }

            for i in 0..legs.len() {
                <InstructionLegs<T>>::insert(instruction_counter, legs[i].leg_number, legs[i].clone());
            }

            <InstructionDetails<T>>::insert(instruction_counter, instruction);
            <InstructionAuthsPending>::insert(instruction_counter, u64::try_from(counter_parties.len()).unwrap_or_default());
            <VenueInfo>::insert(venue_id, venue);
            <InstructionCounter>::put(instruction_counter + 1);
            Self::deposit_event(RawEvent::InstructionCreated(did, venue_id, instruction_counter, settlement_type, valid_from, legs));
            Ok(())
        }

        pub fn authorize_instruction(origin, instruction_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // checks if the sender is a counter party with a pending authorization
            ensure!(Self::user_auths(did, instruction_id) == AuthorizationStatus::Pending, Error::<T>::NoPendingAuth);

            // lock tokens
            let legs = <InstructionLegs<T>>::iter_prefix(instruction_id).collect::<Vec<_>>();
            for i in 0..legs.len() {
                if legs[i].from == did {
                    // TODO: Implement a way to do the checks before committing changes to storage.
                    if T::Asset::unsafe_increase_custody_allowance(
                        did,
                        legs[i].asset,
                        did,
                        SettlementDID.as_id(),
                        legs[i].amount
                    ).is_err() {
                        // Undo custody locks
                        for j in 0..i {
                            T::Asset::unsafe_decrease_custody_allowance(did,
                                legs[j].asset,
                                did,
                                SettlementDID.as_id(),
                                legs[j].amount
                            );
                        }
                        break;
                    }
                }
            }

            let auths_pending = Self::instruction_auths_pending(instruction_id);
            if auths_pending <= 1 {
                // TODO: execute instruction
            }
            // Updates storage
            <UserAuths>::insert(did, instruction_id, AuthorizationStatus::Authorized);
            <AuthsReceived>::insert(instruction_id, did, AuthorizationStatus::Authorized);
            <InstructionAuthsPending>::insert(instruction_id, auths_pending - 1);
            Self::deposit_event(RawEvent::InstructionAuthorized(did, instruction_id));
            Ok(())
        }

        pub fn unauthorize_instruction(origin, instruction_id: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // checks if instruction exists and sender is a counter party with a pending authorization
            ensure!(Self::user_auths(did, instruction_id) == AuthorizationStatus::Authorized, Error::<T>::InstructionNotAuthorized);

            // unlock tokens
            let legs = <InstructionLegs<T>>::iter_prefix(instruction_id).collect::<Vec<_>>();
            for i in 0..legs.len() {
                match Self::instruction_leg_status(instruction_id, legs[i].leg_number) {
                    LegStatus::ExecutionToBeSkipped(signer, receipt_uid) => {
                        <ReceiptsUsed>::insert(&signer, receipt_uid, false);
                        <InstructionLegStatus>::insert(instruction_id, legs[i].leg_number, LegStatus::ExecutionPending);
                        Self::deposit_event(RawEvent::ReceiptUnclaimed(did, instruction_id, legs[i].leg_number, receipt_uid, signer));
                    },
                    LegStatus::ExecutionPending | LegStatus::ExecutionFailed => T::Asset::unsafe_decrease_custody_allowance(
                        did,
                        legs[i].asset,
                        did,
                        SettlementDID.as_id(),
                        legs[i].amount
                    ),
                    LegStatus::ExecutionSuccessful | LegStatus::ExecutionSkipped(..) => return Err(Error::<T>::LegNotPending.into())
                };
            }

            // Updates storage
            <UserAuths>::insert(did, instruction_id, AuthorizationStatus::Pending);
            <AuthsReceived>::remove(instruction_id, did);
            <InstructionAuthsPending>::mutate(instruction_id, |auths_pending| *auths_pending - 1);
            Self::deposit_event(RawEvent::InstructionUnauthorized(did, instruction_id));
            Ok(())
        }

        pub fn claim_receipt(origin, instruction_id: u64, leg_number: u64, receipt_uid: u64, signer: AccountId /*signed_data*/) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // checks if instruction exists and sender is a counter party
            let user_auth = Self::user_auths(did, instruction_id);
            ensure!(
                user_auth == AuthorizationStatus::Authorized,
                Error::<T>::InstructionNotAuthorized
            );
            ensure!(
                Self::instruction_leg_status(instruction_id, leg_number) == LegStatus::ExecutionPending,
                Error::<T>::LegNotPending
            );
            let venue_id = Self::instruction_details(instruction_id).venue_id;
            ensure!(
                Self::venue_signers(venue_id, &signer), Error::<T>::UnauthorizedSigner
            );
            ensure!(
                !Self::receipts_used(&signer, receipt_uid), Error::<T>::ReceiptAlreadyClaimed
            );

            //TODO verify signed data

            let leg = Self::instruction_legs(instruction_id, leg_number);
            T::Asset::unsafe_decrease_custody_allowance(
                did,
                leg.asset,
                did,
                SettlementDID.as_id(),
                leg.amount
            );

            <ReceiptsUsed>::insert(&signer, receipt_uid, true);

            <InstructionLegStatus>::insert(instruction_id, leg_number, LegStatus::ExecutionToBeSkipped(signer.clone(), receipt_uid));
            Self::deposit_event(RawEvent::ReceiptClaimed(did, instruction_id, leg_number, receipt_uid, signer));
            Ok(())
        }

        pub fn unclaim_receipt(origin, instruction_id: u64, leg_number: u64) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // checks if instruction exists and sender is a counter party
            let user_auth = Self::user_auths(did, instruction_id);
            ensure!(
                user_auth == AuthorizationStatus::Authorized,
                Error::<T>::InstructionNotAuthorized
            );

            if let LegStatus::ExecutionToBeSkipped(signer, receipt_uid) = Self::instruction_leg_status(instruction_id, leg_number) {
                let leg = Self::instruction_legs(instruction_id, leg_number);
                T::Asset::unsafe_decrease_custody_allowance(
                    did,
                    leg.asset,
                    did,
                    SettlementDID.as_id(),
                    leg.amount
                );
                <ReceiptsUsed>::insert(&signer, receipt_uid, false);
                <InstructionLegStatus>::insert(instruction_id, leg_number, LegStatus::ExecutionPending);
                Self::deposit_event(RawEvent::ReceiptUnclaimed(did, instruction_id, leg_number, receipt_uid, signer));
                Ok(())
            } else {
                Err(Error::<T>::ReceiptNotClaimed.into())
            }
        }

        pub fn set_venue_filtering(origin, ticker: Ticker, enabled: bool) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            <VenueFiltering>::insert(ticker, enabled);
            Self::deposit_event(RawEvent::VenueFiltering(did, ticker, enabled));
            Ok(())
        }

        pub fn allow_venues(origin, ticker: Ticker, venues: Vec<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            for venue in &venues {
                <VenueAllowList>::insert(&ticker, venue, true);
            }
            Self::deposit_event(RawEvent::VenuesAllowed(did, ticker, venues));
            Ok(())
        }

        pub fn disallow_venues(origin, ticker: Ticker, venues: Vec<u64>) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);
            for venue in &venues {
                <VenueAllowList>::insert(&ticker, venue, false);
            }
            Self::deposit_event(RawEvent::VenuesBlocked(did, ticker, venues));
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

    /// Returns true if `sender_did` is the owner of `ticker` asset.
    fn is_owner(ticker: &Ticker, sender_did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, sender_did)
    }
}
