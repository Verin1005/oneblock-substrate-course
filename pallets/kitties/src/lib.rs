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
    use frame_support::log::info;
    use frame_support::pallet_prelude::*;
    use frame_support::traits::Randomness;
    use frame_system::pallet_prelude::*;
    use sp_io::hashing::blake2_128;

    // ----------------------------------------------------------------
    type KittyIndex = u32;
    type KittyDna = [u8; 16];

    #[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct Kitty(KittyDna);

    #[pallet::type_value]
    pub fn GetDefaultKittyIndex() -> KittyIndex {
        1_u32
    }

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
    }

    /// kitty 的自增 id, 从 1 开始
    #[pallet::storage]
    #[pallet::getter(fn next_kitty_id)]
    pub type NextKittyId<T> = StorageValue<_, KittyIndex, ValueQuery, GetDefaultKittyIndex>;

    /// kitty 的信息
    #[pallet::storage]
    #[pallet::getter(fn kitties)]
    pub type Kitties<T> = StorageMap<_, Blake2_128Concat, KittyIndex, Kitty>;
    /// kitty 的主人
    #[pallet::storage]
    #[pallet::getter(fn kitty_owner)]
    pub type KittyOwner<T: Config> = StorageMap<_, Blake2_128Concat, KittyIndex, T::AccountId>;

    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        Created { who: T::AccountId, kitty_id: KittyIndex, kitty: Kitty },
        Bred { who: T::AccountId, kitty_id: KittyIndex, kitty: Kitty },
        Transferred { from: T::AccountId, to: T::AccountId, kitty_id: KittyIndex },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// kitty_id 无效/溢出
        InvalidKittyId,
        /// kitty_id 相同
        SomeKittyId,
        /// 不是小猫的主人
        NotOwner,
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
            NextKittyId::<T>::set(kitty_id + 1);

            Self::deposit_event(Event::<T>::Created { who, kitty_id, kitty });
            Ok(())
        }

        /// 繁育一只小猫
        #[pallet::weight(10_000)]
        pub fn breed(origin: OriginFor<T>, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> DispatchResult {
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
            NextKittyId::<T>::set(kitty_id + 1);

            Self::deposit_event(Event::<T>::Bred { who, kitty_id, kitty });
            Ok(())
        }

        /// 转让一只小猫
        #[pallet::weight(10_000)]
        pub fn transfer(origin: OriginFor<T>, kitty_id: KittyIndex, to: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;
            // 检查小猫是否自己的
            Self::get_kitty(kitty_id).map_err(|_| Error::<T>::InvalidKittyId)?;
            let owner = Self::kitty_owner(kitty_id);
            ensure!(owner==Some(who.clone()), Error::<T>::NotOwner);
            KittyOwner::<T>::set(kitty_id, Some(to.clone()));

            Self::deposit_event(Event::<T>::Transferred { from: who, to, kitty_id });
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        /// 生成小猫的 dna
        fn gen_dna(who: &T::AccountId) -> KittyDna {
            let payload = (T::Randomness::random_seed(), &who, frame_system::Pallet::<T>::extrinsic_index());
            payload.using_encoded(blake2_128)
        }
        /// 获取小猫的 id
        fn get_next_id() -> Result<KittyIndex, ()> {
            match Self::next_kitty_id() {
                KittyIndex::MAX => Err(()),
                val => Ok(val)
            }
        }
        /// 获取小猫的 信息
        fn get_kitty(kitty_id: KittyIndex) -> Result<Kitty, ()> {
            match Self::kitties(kitty_id) {
                Some(kitty) => Ok(kitty),
                None => Err(())
            }
        }
    }
}
