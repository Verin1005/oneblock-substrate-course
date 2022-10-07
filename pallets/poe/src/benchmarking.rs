//! Benchmarking setup for pallet-poe

use super::*;

#[allow(unused)]
use crate::Pallet as PoeModule;
use frame_benchmarking::{benchmarks, whitelisted_caller};
use frame_support::traits::Get;
use frame_system::RawOrigin;
use sp_std::vec;

benchmarks! {
	create_claim {
		/* 1.初始化需用到的数据 */
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
	}: {
		/* 2.调用调度函数 */
		PoeModule::<T>::create_claim(RawOrigin::Signed(caller).into(),claim.clone())?;
	}

	revoke_claim {
		/* 1.初始化需用到的数据 */
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		// 先创建一个
		PoeModule::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone())?;
	}: {
		/* 2.调用调度函数 */
		PoeModule::<T>::revoke_claim(RawOrigin::Signed(caller).into(),claim.clone())?;
	}

	transfer_claim {
		/* 1.初始化需用到的数据 */
		let d in 0 .. T::MaxClaimLength::get();
		let claim = vec![0; d as usize];
		let caller: T::AccountId = whitelisted_caller();
		let dest_account_id: T::AccountId = frame_benchmarking::account("dest_account_id",0,0);
		// 先创建一个
		PoeModule::<T>::create_claim(RawOrigin::Signed(caller.clone()).into(), claim.clone())?;
	}: {
		/* 2.调用调度函数 */
		PoeModule::<T>::transfer_claim(RawOrigin::Signed(caller).into(),claim.clone(),dest_account_id)?;
	}

	impl_benchmark_test_suite!(PoeModule, crate::mock::new_test_ext(), crate::mock::Test);
}
