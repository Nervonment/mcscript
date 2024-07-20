## 运行测试

### 依赖

在运行测试前, 你需要安装 Minecraft 1.21 Java 版服务器和 mcrcon. 

首先, 下载 [server.jar](https://piston-data.mojang.com/v1/objects/450698d1863ab5180c25d7c804ef0fe6369dd1ba/server.jar) 并将其移动到 [test_server](test_server) 目录下. 然后运行 server.jar (`java -jar server.jar` 或者直接双击运行). 

然后, 下载 [mcrcon](https://github.com/Tiiffi/mcrcon/releases/tag/v0.7.2), 将其放到你自己喜欢的目录下, 并将此目录加入环境变量 Path. 

### 运行

运行测试时, 你需要确保服务器正在运行. 测试的用例在 [example/tests.mcs](example/tests.mcs) 中. [src/tests.rs](src/tests.rs) 中定义了测试的预期结果. 写好测试用例后, 运行 `cargo test -- --nocapture` 开始测试. 