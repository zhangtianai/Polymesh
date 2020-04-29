//! Runtime API definition for committee module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{SaturatedConversion, UniqueSaturatedInto};
use sp_std::{prelude::*, vec::Vec};

sp_api::decl_runtime_apis! {
    /// The API to interact with committee.
    pub trait CommitteeApi<IdentityId>
    where
        IdentityId: Codec,
    {
        /// Retrieve referendums `address` voted on.
        fn voted_on(address: IdentityId) -> Vec<u32>;
    }
}
