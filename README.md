# oneblock substrate course

## 第一题

> 为 proof of existence (poe) 模块的可调用函数 create_claim, revoke_claim, transfer_claim 添加 benchmark 用例，并且将 benchmark 运行的结果应用在可调用函数上；

- `make bench` 生成`weights`文件

### 编写 `benchmarking.rs`

```
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
```

### `weights.rs` 应用到可调度函数上

```
#[pallet::weight(T::WeightInfo::revoke_claim(claim.len() as u32))]
```

## 第二题

> 选择 node-template 或者其它节点程序，生成 Chain Spec 文件（两种格式都需要）

- 生成可编辑的 Chain Spec 文件

```
cargo build --release && ./target/release/node-template build-spec --disable-default-bootnode --chain dev > customSpec.json
```

- 转换为原始规范格式才能使用

```
./target/release/node-template build-spec --chain=customSpec.json --raw --disable-default-bootnode > customSpecRaw.json
```

1. （附加题）根据 Chain Spec，部署公开测试网络
