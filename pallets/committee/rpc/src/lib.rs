use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_committee_rpc_runtime_api::{
    self as runtime_api, CommitteeApi as CommitteeRuntimeApi,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_std::{prelude::*, vec::Vec};
use std::sync::Arc;

/// Committee RPC methods.
#[rpc]
pub trait CommitteeApi<BlockHash, IdentityId> {
    /// Retrieves proposals `address` voted on
    #[rpc(name = "committee_votedOn")]
    fn voted_on(&self, address: IdentityId, at: Option<BlockHash>) -> Result<Vec<u32>>;
}

/// An implementation of committee specific RPC methods.
pub struct Committee<T, U> {
    client: Arc<T>,
    _marker: std::marker::PhantomData<U>,
}

impl<T, U> Committee<T, U> {
    /// Create new `Committee` with the given reference to the client.
    pub fn new(client: Arc<T>) -> Self {
        Committee {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl<C, Block, IdentityId> CommitteeApi<<Block as BlockT>::Hash, IdentityId> for Committee<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: CommitteeRuntimeApi<Block, IdentityId>,
    IdentityId: Codec,
{
    fn voted_on(
        &self,
        address: IdentityId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<u32>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let result = api.voted_on(&at, address).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError as i64),
            message: "Unable to query voted_on.".into(),
            data: Some(format!("{:?}", e).into()),
        })?;

        Ok(result)
    }
}
