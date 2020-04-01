#![cfg(feature = "runtime-benchmarks")]

use crate::asset::*;
use frame_system::RawOrigin;
use sp_std::prelude::*;
use frame_benchmarking::{benchmarks, account};

use Module as Benchmark;

const SEED: u32 = 0;

benchmarks! {
    _ {
	let m in 1 .. 1000 => {
	    let origin = RawOrigin::Signed(account("member", m, SEED));
	    Benchmark::<T>::register_ticker(origin.into())?
	};
    }

    register_ticker {
	let m in ...;
    }: _(RawOrigin::Signed(account("member", m + 1, SEED)))
}
