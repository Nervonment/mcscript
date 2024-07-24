## 使用说明

假如你已经编写了一些代码, 源代码文件是 `hello.mcs` `hi.mcs`. 想要将它编译为名为 `my_datapack` 的 Minecraft 数据包, 可以运行以下命令: 

```
mcsc hello.mcs hi.mcs -o my_datapack
```

之后, 编译器会输出两个数据包, 一个名为 `my_datapack`, 包含了你在 `hello.mcs` 和 `hi.mcs` 中编写的函数. 另一个名为 `mcscript`, 包含了运行 MCScript 所生成的数据包所依赖的一些函数. 

接下来, 将两个数据包复制到你的存档文件夹的 `datapack` 目录 (`.minecraft/saves/<存档名字>/datapacks/`) 下, 然后打开游戏, 进入存档. (如果在已经进入了游戏的时候更新了数据包, 需要在游戏内运行命令 `/reload` 重新加载. )

在使用任何 MCScript 生成的数据包中的函数前, 需要先运行一次以下命令来初始化: 

```
/function mscript:init
```

这条命令每个存档只用运行一次即可. *(除非你的代码出现了异常, 导致代表栈的命令存储 `memory:stack frame` 未能复位. 你可以使用 `/data get storage memory:stack frame` 查看栈是否正常, 正常情况下它的值应为 `[]`. )*

假设 `hello.mcs` 的内容如下: 

```
// hello.mcs

fn foo() {
    run_command!("say Hello, world! ");
}
```

在游戏内, 想要运行函数 `foo`, 你需要运行如下命令: 

```
/function hello:foo
```

然后你就可以在聊天栏看到消息 "Hello, world! ". 

注意, 如果你的源代码中定义了全局变量, 想要把全局变量设为你设定的初始值, 需要手动运行一些命令. 例如, 假如 `hi.mcs` 的内容如下: 

```
// hi.mcs

let c: int = 0;

fn bar() {
    c += 1;
    run_command!("say c = {}", c);
}
```

要将全局变量 `c` 的值设置为 `0`, 需要运行如下命令: 

```
/function hi:init
```

然后你可以尝试多次运行 `/function hi:bar`, 便可以看到 `c` 的值依次递增. 