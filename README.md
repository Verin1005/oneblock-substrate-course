# 课程4作业

1. 在 Offchain Worker 中，使用 Offchain Indexing 特性实现从链上向 Offchain Storage 中写入数据
    - 主目录中运行 `make r`
    - ![作业1](./docs/task1.png)
2. 使用 js sdk 从浏览器 frontend 获取到前面写入 Offchain Storage 的数据
3. 回答链上随机数（如前面Kitties示例中）与链下随机数的区别
4. （可选）在 Offchain Worker
   中，解决向链上发起不签名请求时剩下的那个错误。参考：https://github.com/paritytech/substrate/blob/master/frame/examples/offchain-worker/src/lib.rs
5. （可选）构思一个应用场景，描述如何使用 Offchain Features 三大组件去实现它
6. （可选）如果有时间，可以实现一个上述原型