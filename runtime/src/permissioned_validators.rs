use rstd::{prelude::*, marker::PhantomData};

use crate::balances;
use crate::identity;

use codec::{Codec, Encode, Decode};
use sr_primitives::{
	traits::{CheckedAdd, CheckedSub, SignedExtension, Dispatchable},
    weights::DispatchInfo,
	transaction_validity::{
		ValidTransaction, InvalidTransaction, TransactionValidity, TransactionValidityError,
	},
    DispatchError,
};
use srml_support::{
    decl_event, decl_module, decl_storage, Parameter,
    dispatch::Result,
    ensure,
    traits::{Currency, ExistenceRequirement, WithdrawReason},
};
use system::{self, ensure_signed};


pub trait Trait: system::Trait + identity::Trait + staking::Trait {
}

decl_storage! {
    trait Store for Module<T: Trait> as PermissionedValidators {
		/// The current set of permissioned validators, stored as an ordered Vec.
		Members get(members): Vec<T::AccountId>;
	}
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin
	{

		fn add_member(origin, who: T::AccountId) {
            //Check that origin is allowed to add an account_id - could use the collective of current validators here
            //Get DID associated w/ who and check they have a KYB attestation etc.
			let mut members = <Members<T>>::get();
			let location = members.binary_search(&who).err().ok_or("already a member")?;
			members.insert(location, who.clone());
			<Members<T>>::put(&members);
		}

        //Remove members etc. etc. manage ingress / egress of permissioned validators

	}
}

#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct PermissionedValidator<T: Trait + Send + Sync>(PhantomData<T>);

impl<T: Trait + Send + Sync> Default for PermissionedValidator<T> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

#[cfg(feature = "std")]
impl<T: Trait + Send + Sync> std::fmt::Debug for PermissionedValidator<T> {
	fn fmt(&self, _: &mut std::fmt::Formatter) -> std::fmt::Result {
		Ok(())
	}
}

impl<T: Trait + Send + Sync> SignedExtension for PermissionedValidator<T> {
	type AccountId = T::AccountId;
	type Call = T::Call;
	type AdditionalSigned = ();
	type Pre = ();

	fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> { Ok(()) }

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		_: DispatchInfo,
		_: usize,
	) -> TransactionValidity {

        return Ok(ValidTransaction::default());		
	}
}