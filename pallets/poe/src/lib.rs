#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_std::vec::Vec;

	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		/// claim 最大长度
		#[pallet::constant]
		type MaxClaimLength: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn get_proofs)]
	pub type Proofs<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		BoundedVec<u8, T::MaxClaimLength>,
		(T::AccountId, T::BlockNumber),
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// 存证创建成功 (创建者, 存证)
		ClaimCreated(T::AccountId, Vec<u8>),
		/// 存证撤销成功 (创建者, 存证)
		ClaimRevoked(T::AccountId, Vec<u8>),
		/// 存证转移成功 (老拥有者, 新拥有者, 存证)
		ClaimTransfer(T::AccountId, T::AccountId, Vec<u8>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// 文件 hash 值超过指定最大长度 MaxClaimLength
		ClaimTooLong,
		/// 文件存证已存在
		ClaimAlreadyExists,
		/// 文件存证不存在
		ClaimNotExists,
		/// 不是存证所有者
		NotClaimOwner,
		/// 不能将存证自己转给自己
		TransferToSelf,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// 用户创建一个文件存在证明 [claim: 文件的 hash 值(Vec<u8>)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn create_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			// 文件存在证明声明者
			let who = ensure_signed(origin)?;
			// 将 claim vec 转换成有长度限制的 BoundedVec
			let bounded_claim = Self::claim_vec_to_bounded_vec(claim.clone())?;
			// 判断是否存在
			ensure!(!Proofs::<T>::contains_key(&bounded_claim), Error::<T>::ClaimAlreadyExists);
			// 将文件声明者 与 hash 值绑定入库
			Proofs::<T>::insert(&bounded_claim, (&who, frame_system::Pallet::<T>::block_number()));
			// 发送通知事件
			Self::deposit_event(Event::ClaimCreated(who, claim));
			Ok(())
		}

		/// 用户撤销存证 [claim: 文件的 hash 值(Vec<u8>)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn revoke_claim(origin: OriginFor<T>, claim: Vec<u8>) -> DispatchResult {
			// 文件存证的拥有者
			let who = ensure_signed(origin)?;
			// 将 claim vec 转换成有长度限制的 BoundedVec
			let bounded_claim = Self::claim_vec_to_bounded_vec(claim.clone())?;
			// 判断是否拥有者调用的
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExists)?;
			ensure!(owner == who, Error::<T>::NotClaimOwner);
			// 将文件存证删除
			Proofs::<T>::remove(&bounded_claim);
			// 发送通知事件
			Self::deposit_event(Event::ClaimRevoked(who, claim));
			Ok(())
		}

		/// 用户转移存证给他人 [claim: 文件的 hash 值(Vec<u8>, dest: 转移的目标用户)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn transfer_claim(
			origin: OriginFor<T>,
			claim: Vec<u8>,
			dest: T::AccountId,
		) -> DispatchResult {
			// 文件存证的拥有者
			let who = ensure_signed(origin)?;
			// 将 claim vec 转换成有长度限制的 BoundedVec
			let bounded_claim = Self::claim_vec_to_bounded_vec(claim.clone())?;
			// 判断是否拥有者调用的
			let (owner, _) = Proofs::<T>::get(&bounded_claim).ok_or(Error::<T>::ClaimNotExists)?;
			ensure!(owner == who, Error::<T>::NotClaimOwner);
			// 是否转给了自己?
			ensure!(owner != dest, Error::<T>::TransferToSelf);
			// 将文件存证转移
			Proofs::<T>::insert(
				&bounded_claim,
				(dest.clone(), frame_system::Pallet::<T>::block_number()),
			);
			// 发送通知事件
			Self::deposit_event(Event::ClaimTransfer(who, dest, claim));
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// 将 claim vec 转换成有长度限制的 BoundedVec. 统一处理
		fn claim_vec_to_bounded_vec(
			claim: Vec<u8>,
		) -> Result<BoundedVec<u8, T::MaxClaimLength>, Error<T>> {
			let bounded_claim = BoundedVec::<u8, T::MaxClaimLength>::try_from(claim.clone())
				.map_err(|_| Error::<T>::ClaimTooLong)?;
			Ok(bounded_claim)
		}
	}
}
