#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
    use std::fmt::Debug;
    use frame_support::log::info;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::{Currency, Randomness, ReservableCurrency};
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;
    use sp_runtime::traits::{Bounded};

    // ----------------------------------------------------------------
    type KittyDna = [u8; 16];

    /// 定义余额类型
    // type BalanceOf<T> =
    // <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Kitty(KittyDna);

    // ----------------------------------------------------------------
    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
        /// 随机数
        type Randomness: Randomness<Self::Hash, Self::BlockNumber>;
        /// 可质押的货币
        type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
        /// 作业2: 小猫 id 在 runtime 中指定具体类型
        type KittyIndex: Parameter
        + Member
        + sp_runtime::traits::AtLeast32BitUnsigned
        + codec::Codec
        + Default
        + Copy
        + MaybeSerializeDeserialize
        + Debug
        + MaxEncodedLen
        + TypeInfo;

        // 最大拥有的小猫数量
        #[pallet::constant]
        type MaxOwnerKitty: Get<u32>;
    }

    /// kitty 的自增 id, 从 1 开始
    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T: Config> = StorageValue<_, T::KittyIndex, ValueQuery>;

    /// kitty 的信息
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, Kitty>;
    /// kitty 的主人
    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, T::KittyIndex, T::AccountId>;
    /// 作业3: 获取账户下所有的小猫
    #[pallet::storage]
    #[pallet::getter(fn user_kitties)]
    pub type UserKitties<T: Config> = StorageMap<_, Blake2_128Concat,
        T::AccountId, BoundedVec<T::KittyIndex, T::MaxOwnerKitty>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created { who: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty },
        Bred { who: T::AccountId, kitty_id: T::KittyIndex, kitty: Kitty },
        Transferred { from: T::AccountId, to: T::AccountId, kitty_id: T::KittyIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// kitty_id 无效/溢出
        InvalidKittyId,
        /// kitty_id 相同
        SomeKittyId,
        /// 不是小猫的主人
        NotOwner,
        /// 超过最大拥有小猫数量
        OverflowMaxOwnerKitty,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 创建一只小猫
        #[pallet::weight(10_000)]
        pub fn create(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 获取小猫自增 id
            let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;
            // 生成小猫
            let dna = Self::gen_dna(&who);
            let kitty = Kitty(dna);
            // 入库
            Kitties::<T>::insert(&kitty_id, &kitty);
            KittyOwner::<T>::insert(&kitty_id, &who);
            NextKittyId::<T>::set(kitty_id + 1u32.into());
            UserKitties::<T>::try_mutate(&who, |vec|
                vec.try_push(kitty_id)).map_err(|_| Error::<T>::OverflowMaxOwnerKitty)?;

            Self::deposit_event(Event::<T>::Created { who, kitty_id, kitty });
            Ok(())
        }

        /// 繁育一只小猫
        #[pallet::weight(10_000)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: T::KittyIndex, kitty_id_2: T::KittyIndex) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 检查 kitty_id 对应的小猫是否存在
            ensure!(kitty_id_1 == kitty_id_2,Error::<T>::SomeKittyId);
            let kitty_1 = Self::get_kitty(kitty_id_1).map_err(|_| Error::<T>::InvalidKittyId)?;
            let kitty_2 = Self::get_kitty(kitty_id_2).map_err(|_| Error::<T>::InvalidKittyId)?;
            // 获取小猫自增 id
            let kitty_id = Self::get_next_id().map_err(|_| Error::<T>::InvalidKittyId)?;
            // 生成小猫 dna
            let dna = Self::gen_dna(&who);
            let mut new_dna = KittyDna::default();
            for i in 0..kitty_1.0.len() {
                let a = dna[i];
                let b = !dna[i];
                let c = kitty_1.0[i] & dna[i];
                info!("--------------------------------{a},{b},{c}");
                new_dna[i] = (kitty_1.0[i] & dna[i]) | (kitty_2.0[i] & !dna[i]);
            }
            info!("父小猫: {:?}",kitty_1.0);
            info!("母小猫: {:?}",kitty_2.0);
            info!("新小猫: {new_dna:?}");
            let kitty = Kitty(new_dna);
            // 入库
            Kitties::<T>::insert(&kitty_id, &kitty);
            KittyOwner::<T>::insert(&kitty_id, &who);
            NextKittyId::<T>::set(kitty_id + 1u32.into());
            UserKitties::<T>::try_mutate(&who, |vec|
                vec.try_push(kitty_id)).map_err(|_| Error::<T>::OverflowMaxOwnerKitty)?;

            Self::deposit_event(Event::<T>::Bred { who, kitty_id, kitty });
            Ok(())
        }

        /// 转让一只小猫
        #[pallet::weight(10_000)]
        pub fn transfer(origin: OriginFor<T>, kitty_id: T::KittyIndex, to: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 检查小猫是否自己的
            Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
            let owner = Self::kitty_owner(kitty_id);
            ensure!(owner==Some(who.clone()), Error::<T>::NotOwner);
            KittyOwner::<T>::set(kitty_id, Some(to.clone()));
            // 将之前的小猫数组清除
            UserKitties::<T>::try_mutate(&who, |vec| -> DispatchResult {
                // 获取索引
                if let Some(index) = vec.iter().position(|&x| x == kitty_id) {
                    vec.swap_remove(index);
                    Ok(())
                } else {
                    Err(Error::<T>::InvalidKittyId.into())
                }
            })?;
            UserKitties::<T>::try_mutate(&to, |vec|
                vec.try_push(kitty_id)).map_err(|_| Error::<T>::OverflowMaxOwnerKitty)?;

            Self::deposit_event(Event::<T>::Transferred { from: who, to, kitty_id });
            Ok(())
        }


        /// 购买一只小猫
        #[pallet::weight(10_000)]
        pub fn buy_kitty(origin: OriginFor<T>, kitty_id: T::KittyIndex, to: T::AccountId) -> DispatchResult {}
    }

    impl<T: Config> Pallet<T> {
        /// 生成小猫的 dna
        fn gen_dna(who: &T::AccountId) -> KittyDna {
            let payload = (T::Randomness::random_seed(), &who, frame_system::Pallet::<T>::extrinsic_index());
            payload.using_encoded(blake2_128)
        }
        /// 获取小猫的 id
        fn get_next_id() -> Result<T::KittyIndex, ()> {
            let max = T::KittyIndex::max_value();
            let kitty_id = Self::next_kitty_id();
            if kitty_id < max {
                Ok(kitty_id)
            } else {
                Err(())
            }
        }
        /// 获取小猫的 信息
        fn get_kitty(kitty_id: T::KittyIndex) -> Result<Kitty, ()> {
            match Self::kitties(kitty_id) {
                Some(kitty) => Ok(kitty),
                None => Err(())
            }
        }
    }
}
