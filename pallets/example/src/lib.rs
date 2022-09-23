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
    use frame_support::log::{info};
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use sp_runtime::offchain::storage::StorageValueRef;
    use sp_std::vec::Vec;

    // ----------------------------------------------------------------

    /// off-chain index 传输的数据封装.
    #[derive(Debug, Encode, Decode, Default)]
    struct IndexingData(Vec<u8>, u64);

    /// index 唯一 key 的前缀
    const ONCHAIN_TX_KEY: &[u8] = b"pallet_example::indexing::";

    // ----------------------------------------------------------------
    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::event]
    // #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        SomethingStored(u32, T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        // 链下工作者入口
        fn offchain_worker(block_number: BlockNumberFor<T>) {
            info!("---Entering off-chain worker");
            // 获取 off-chain indexing 数据.
            let key = Self::derived_key(block_number);
            let index_storage_info = StorageValueRef::persistent(&key);
            if let Ok(Some(data)) = index_storage_info.get::<IndexingData>() {
                info!(
					"--off-chain indexing data: {:?}, {:?}",
					sp_std::str::from_utf8(&data.0).unwrap_or("error"),
					data.1
				);
            } else {
                info!("--no off-chain indexing data retrieved")
            }
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// 将数据通过 oci 传输
        #[pallet::weight(10_000)]
        pub fn extrinsic(origin: OriginFor<T>, number: u64) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            let key = Self::derived_key(frame_system::Pallet::<T>::block_number());
            // 封装 index 数据
            let data = IndexingData(b"extrinsic".to_vec(), number);
            sp_io::offchain_index::set(&key, &data.encode());
            info!("-- key = {:?}",key);
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        // 根据 block_number 生产 off-chain index key
        fn derived_key(block_number: T::BlockNumber) -> Vec<u8> {
            block_number.using_encoded(|encoded_block_number| {
                ONCHAIN_TX_KEY
                    .clone()
                    .into_iter()
                    // .chain(b"/".into_iter()) //将两个迭代器链接在一起创建新的迭代器
                    .chain(encoded_block_number)
                    .copied() //复制所有元素到新创建新的迭代器中。这很有用,当您有一个基于 &T
                    // 的迭代器时,但您需要一个基于 T 的迭代器.
                    .collect::<Vec<u8>>()
            })
        }
    }
}
