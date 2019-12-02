pub mod identity {
   use srml_support::{ decl_event, Parameter };
   use sr_primitives::{traits::Dispatchable };
   use primitives::{IdentityId, Key, Permission, SigningKey};
   use super::balances;

   #[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
   pub struct Claim<U> {
       issuance_date: U,
       expiry: U,
       claim_value: ClaimValue,
   }

   #[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
   pub struct ClaimMetaData {
       claim_key: Vec<u8>,
       claim_issuer: IdentityId,
   }

   #[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
   pub enum DataTypes {
       U8,
       U16,
       U32,
       U64,
       U128,
       Bool,
       VecU8,
   }

   impl Default for DataTypes {
       fn default() -> Self {
           DataTypes::VecU8
       }
   }

   #[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
   pub struct ClaimValue {
       pub data_type: DataTypes,
       pub value: Vec<u8>,
   }


   decl_event!(
       pub enum Event<T>
       where
           AccountId = <T as system::Trait>::AccountId,
           Moment = <T as timestamp::Trait>::Moment,
           {
               /// DID, master key account ID, signing keys
               NewDid(IdentityId, AccountId, Vec<SigningKey>),

               /// DID, new keys
               SigningKeysAdded(IdentityId, Vec<SigningKey>),

               /// DID, the keys that got removed
               SigningKeysRemoved(IdentityId, Vec<Key>),

               /// DID, updated signing key, previous permissions
               SigningPermissionsUpdated(IdentityId, SigningKey, Vec<Permission>),

               /// DID, old master key account ID, new key
               NewMasterKey(IdentityId, AccountId, Key),

               /// DID, claim issuer DID
               NewClaimIssuer(IdentityId, IdentityId),

               /// DID, removed claim issuer DID
               RemovedClaimIssuer(IdentityId, IdentityId),

               /// DID, claim issuer DID, claims
               NewClaims(IdentityId, ClaimMetaData, Claim<Moment>),

               /// DID, claim issuer DID, claim
               RevokedClaim(IdentityId, ClaimMetaData),

               /// DID
               NewIssuer(IdentityId),

               /// DID queried
               DidQuery(Key, IdentityId),
        }
   );

   /// The module's configuration trait.
   pub trait Trait: system::Trait + balances::Trait + timestamp::Trait {
        /// The overarching event type.
        type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
        /// An extrinsic call.
        type Proposal: Parameter + Dispatchable<Origin = Self::Origin>;
   }

   pub trait IdentityTrait<T> {
        fn get_identity(key: &Key) -> Option<IdentityId>;
        fn is_authorized_key(did: IdentityId, key: &Key) -> bool;
        fn is_authorized_with_permissions(
            did: IdentityId,
            key: &Key,
            permissions: Vec<Permission>,
        ) -> bool;
        fn is_master_key(did: IdentityId, key: &Key) -> bool;
   }
}

pub mod balances {
    use codec::{Codec};
    // use rstd::{convert::TryFrom, mem, prelude::*, result};
    use srml_support::{
        decl_event, Parameter,
        traits::{
            Currency, Get, OnFreeBalanceZero, OnUnbalanced,
        }
    };
    use sr_primitives::{
        weights::Weight,
        traits::{
            Convert, MaybeSerializeDebug, Member, SimpleArithmetic,
        }
    };
    use system::OnNewAccount;
    use super::identity::{ IdentityTrait };

    pub trait Instance {}
    pub struct DefaultInstance;
    impl Instance for DefaultInstance {}

    pub trait Subtrait<I: Instance = DefaultInstance>: system::Trait {
        /// The balance of an account.
        type Balance: Parameter
            + Member
            + SimpleArithmetic
            + Codec
            + Default
            + Copy
            + MaybeSerializeDebug
            + From<u128>
            + From<Self::BlockNumber>;

        /// A function that is invoked when the free-balance has fallen below the existential deposit and
        /// has been reduced to zero.
        ///
        /// Gives a chance to clean up resources associated with the given account.
        type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

        /// Handler for when a new account is created.
        type OnNewAccount: OnNewAccount<Self::AccountId>;

        /// The minimum amount required to keep an account open.
        type ExistentialDeposit: Get<Self::Balance>;

        /// The fee required to make a transfer.
        type TransferFee: Get<Self::Balance>;

        /// The fee required to create an account.
        type CreationFee: Get<Self::Balance>;

        /// The fee to be paid for making a transaction; the base.
        type TransactionBaseFee: Get<Self::Balance>;

        /// The fee to be paid for making a transaction; the per-byte portion.
        type TransactionByteFee: Get<Self::Balance>;

        /// Convert a weight value into a deductible fee based on the currency type.
        type WeightToFee: Convert<Weight, Self::Balance>;

        /// Used to charge fee to identity rather than user directly
        type Identity: IdentityTrait<Self::Balance>;
    }


    impl<T: Trait<I>, I: Instance> Subtrait<I> for T {
        type Balance = T::Balance;
        type OnFreeBalanceZero = T::OnFreeBalanceZero;
        type OnNewAccount = T::OnNewAccount;
        type ExistentialDeposit = T::ExistentialDeposit;
        type TransferFee = T::TransferFee;
        type CreationFee = T::CreationFee;
        type TransactionBaseFee = T::TransactionBaseFee;
        type TransactionByteFee = T::TransactionByteFee;
        type WeightToFee = T::WeightToFee;
        type Identity = T::Identity;
    }

    /// Opaque, move-only struct with private fields that serves as a token denoting that
    /// funds have been destroyed without any equal and opposite accounting.
    #[must_use]
    pub struct NegativeImbalance<T: Subtrait<I>, I: Instance = DefaultInstance>(T::Balance);

    impl<T: Subtrait<I>, I: Instance> NegativeImbalance<T, I> {
        /// Create a new negative imbalance from a balance.
        pub fn new(amount: T::Balance) -> Self {
            NegativeImbalance(amount)
        }
    }

    decl_event!(
        pub enum Event<T, I: Instance = DefaultInstance>
        where
            <T as system::Trait>::AccountId,
            <T as Trait<I>>::Balance
        {
            /// A new account was created.
            NewAccount(AccountId, Balance),
            /// An account was reaped.
            ReapedAccount(AccountId),
            /// Transfer succeeded (from, to, value, fees).
            Transfer(AccountId, AccountId, Balance, Balance),
        }
    );

    pub trait Trait<I: Instance = DefaultInstance>: system::Trait {
        /// The balance of an account.
        type Balance: Parameter
            + Member
            + SimpleArithmetic
            + Codec
            + Default
            + Copy
            + MaybeSerializeDebug
            + From<u128>
            + From<Self::BlockNumber>;

        /// A function that is invoked when the free-balance has fallen below the existential deposit and
        /// has been reduced to zero.
        ///
        /// Gives a chance to clean up resources associated with the given account.
        type OnFreeBalanceZero: OnFreeBalanceZero<Self::AccountId>;

        /// Handler for when a new account is created.
        type OnNewAccount: OnNewAccount<Self::AccountId>;

        /// Handler for the unbalanced reduction when taking transaction fees.
        type TransactionPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

        /// Handler for the unbalanced reduction when taking fees associated with balance
        /// transfer (which may also include account creation).
        type TransferPayment: OnUnbalanced<NegativeImbalance<Self, I>>;

        /// Handler for the unbalanced reduction when removing a dust account.
        type DustRemoval: OnUnbalanced<NegativeImbalance<Self, I>>;

        /// The overarching event type.
        type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;

        /// The minimum amount required to keep an account open.
        type ExistentialDeposit: Get<Self::Balance>;

        /// The fee required to make a transfer.
        type TransferFee: Get<Self::Balance>;

        /// The fee required to create an account.
        type CreationFee: Get<Self::Balance>;

        /// The fee to be paid for making a transaction; the base.
        type TransactionBaseFee: Get<Self::Balance>;

        /// The fee to be paid for making a transaction; the per-byte portion.
        type TransactionByteFee: Get<Self::Balance>;

        /// Convert a weight value into a deductible fee based on the currency type.
        type WeightToFee: Convert<Weight, Self::Balance>;

        /// Used to charge fee to identity rather than user directly
        type Identity: IdentityTrait<Self::Balance>;
    }

    pub trait BalancesTrait<T, A> : Currency<A> {

    }
}
