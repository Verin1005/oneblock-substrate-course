use super::*;
use crate::mock::*;
use frame_support::{assert_noop, assert_ok};
use sp_runtime::traits::Get;

#[test]
fn create_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(2)));
		// kitty 创建成功
		assert_ne!(Kitties::<Test>::get(0), None);
		assert_ne!(Kitties::<Test>::get(1), None);
		// 小猫主人是否正确
		assert_eq!(KittyOwner::<Test>::get(0), Some(1));
		assert_eq!(KittyOwner::<Test>::get(1), Some(2));
		// kitty_id 自增
		assert_eq!(NextKittyId::<Test>::get(), 2);
		// 用户的小猫们
		assert_eq!(UserKitties::<Test>::get(1), vec![0]);
		assert_eq!(UserKitties::<Test>::get(2), vec![1]);
		// 质押金额
		let account = System::account(&1);
		assert_eq!(Balances::reserved_balance(1), <Test as Config>::StakeAmount::get());
		assert_eq!(account.data.reserved, <Test as Config>::StakeAmount::get());
		assert_eq!(Balances::free_balance(1), 500);
		// 事件
		System::assert_has_event(crate::mock::Event::KittiesModule(crate::Event::Created {
			who: 1,
			kitty_id: 0,
			kitty: Kitties::<Test>::get(0).unwrap(),
		}));
		System::assert_last_event(crate::mock::Event::KittiesModule(crate::Event::Created {
			who: 2,
			kitty_id: 1,
			kitty: Kitties::<Test>::get(1).unwrap(),
		}));
		assert_eq!(
			events(),
			[
				mock::Event::Balances(pallet_balances::Event::Reserved { who: 1, amount: 500 }),
				mock::Event::KittiesModule(crate::Event::Created {
					who: 1,
					kitty_id: 0,
					kitty: Kitties::<Test>::get(0).unwrap(),
				}),
				mock::Event::Balances(pallet_balances::Event::Reserved { who: 2, amount: 500 }),
				crate::mock::Event::KittiesModule(crate::Event::Created {
					who: 2,
					kitty_id: 1,
					kitty: Kitties::<Test>::get(1).unwrap(),
				})
			]
		);
	});
}

#[test]
fn create_fails() {
	new_test_ext().execute_with(|| {
		// 质押金额不足
		assert_noop!(
			KittiesModule::create(Origin::signed(4)),
			Error::<Test>::InsufficientStakeAmount
		);
		// 超出最大拥有数量
		for _i in 0..<Test as Config>::MaxOwnerKitty::get() {
			assert_ok!(KittiesModule::create(Origin::signed(10)));
			// 确保随机数不同
			System::set_block_number(System::block_number() + 1);
		}
		assert_noop!(
			KittiesModule::create(Origin::signed(10)),
			Error::<Test>::OverflowMaxOwnerKitty
		);
	});
}

#[test]
fn breed_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		System::set_block_number(System::block_number() + 1);
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		System::set_block_number(System::block_number() + 1);
		assert_ok!(KittiesModule::breed(Origin::signed(2), 0, 1));
		System::set_block_number(System::block_number() + 1);

		// kitty 繁育成功
		assert_ne!(Kitties::<Test>::get(2), None);
		// 小猫主人是否正确
		assert_eq!(KittyOwner::<Test>::get(2), Some(2));
		// kitty_id 自增
		assert_eq!(NextKittyId::<Test>::get(), 3);
		// 用户的小猫们
		assert_eq!(UserKitties::<Test>::get(1), vec![0, 1]);
		assert_eq!(UserKitties::<Test>::get(2), vec![2]);
		// 质押成功
		assert_eq!(Balances::reserved_balance(2), 500);
	});
}

#[test]
fn breed_fails() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// 相同的小猫不能繁育
		assert_noop!(KittiesModule::breed(Origin::signed(2), 0, 0), Error::<Test>::SameKittyId);
		System::set_block_number(System::block_number() + 1);
		// 余额不足
		assert_noop!(
			KittiesModule::breed(Origin::signed(4), 0, 1),
			Error::<Test>::InsufficientStakeAmount
		);
	});
}

#[test]
fn transfer_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		assert_ok!(KittiesModule::transfer(Origin::signed(1), 0, 2));
		let kitty_id = 0;
		// 小猫的主人改变
		let owner = KittyOwner::<Test>::get(kitty_id).unwrap();
		assert_eq!(owner, 2);
		// 小猫转移成功 UserKitties
		assert_eq!(UserKitties::<Test>::get(1), vec![]);
		assert_eq!(UserKitties::<Test>::get(2), vec![0]);
		// 旧的质押退回
		assert_eq!(Balances::reserved_balance(1), 0);
		assert_eq!(Balances::free_balance(1), 1000);
		// 新的质押成功
		assert_eq!(Balances::reserved_balance(2), 500);
	});
}

#[test]
fn transfer_fails() {
	new_test_ext().execute_with(|| {
		// 小猫不存在
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 0, 2),
			Error::<Test>::InvalidKittyId
		);
		
		assert_ok!(KittiesModule::create(Origin::signed(1)));
		// 小猫不属于调用者
		assert_noop!(KittiesModule::transfer(Origin::signed(4), 0, 2), Error::<Test>::NotOwner);
		// 超过最大拥有数量,无法转让
		for _i in 0..<Test as Config>::MaxOwnerKitty::get() {
			assert_ok!(KittiesModule::create(Origin::signed(10)));
			// 确保随机数不同
			System::set_block_number(System::block_number() + 1);
		}
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 0, 10),
			Error::<Test>::OverflowMaxOwnerKitty
		);
		// 余额不足
		assert_noop!(
			KittiesModule::transfer(Origin::signed(1), 0, 4),
			Error::<Test>::InsufficientStakeAmount
		);
	});
}
