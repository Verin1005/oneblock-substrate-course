use super::*;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, BoundedVec};

/// 创建存证成功
#[test]
fn create_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		// 创建成功
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));

		// 转换 key(BoundedVec)
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim.clone()).unwrap();
		// 验证数据入库成功
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((1, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

/// 创建存证失败, 存证已存在
#[test]
fn create_claim_failed_when_claim_already_exists() {
	new_test_ext().execute_with(|| {
		// 存证已经存在
		let claim = vec![0, 1];
		let _ = PoeModule::create_claim(Origin::signed(1), claim.clone());
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim),
			Error::<Test>::ClaimAlreadyExists
		);
	});
}

/// 创建存证失败, 存证 hash 超过定义长度
#[test]
fn create_claim_failed_when_claim_too_long() {
	new_test_ext().execute_with(|| {
		// 测试存证长度超出
		let claim = vec![1].repeat(513);
		assert_noop!(
			PoeModule::create_claim(Origin::signed(1), claim.clone()),
			Error::<Test>::ClaimTooLong
		);
	});
}

/// 撤销存证
#[test]
fn revoke_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		// 先创建一个存证
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		// 撤销存证
		assert_ok!(PoeModule::revoke_claim(Origin::signed(1), claim.clone()));
		// 已被撤销
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim).unwrap();
		assert_eq!(Proofs::<Test>::get(&bounded_claim), None);
	});
}

/// 撤销存证失败
#[test]
fn revoke_claim_failed() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		// 先创建一个存证
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		// 撤销一个不存在的存证
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(1), vec![0, 0]),
			Error::<Test>::ClaimNotExists
		);
		// 不是存证的拥有者
		assert_noop!(
			PoeModule::revoke_claim(Origin::signed(2), claim.clone()),
			Error::<Test>::NotClaimOwner
		);
	});
}

/// 转移存证
#[test]
fn transfer_claim_works() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		// 先创建一个存证
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		// 转移存证
		assert_ok!(PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 2));
		// 存证已被转移
		let bounded_claim =
			BoundedVec::<u8, <Test as Config>::MaxClaimLength>::try_from(claim).unwrap();
		assert_eq!(
			Proofs::<Test>::get(&bounded_claim),
			Some((2, frame_system::Pallet::<Test>::block_number()))
		);
	});
}

/// 转移存证失败
#[test]
fn transfer_claim_failed() {
	new_test_ext().execute_with(|| {
		let claim = vec![0, 1];
		// 先创建一个存证
		assert_ok!(PoeModule::create_claim(Origin::signed(1), claim.clone()));
		// 是否转给了自己
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), claim.clone(), 1),
			Error::<Test>::TransferToSelf
		);
		// 不存在的存证
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(1), vec![0, 0], 2),
			Error::<Test>::ClaimNotExists
		);
		// 不是存证的拥有者
		assert_noop!(
			PoeModule::transfer_claim(Origin::signed(2), claim.clone(), 1),
			Error::<Test>::NotClaimOwner
		);
	});
}
