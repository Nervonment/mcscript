# MCScript

MCScript 是一门简易的编程语言, 它的目标语言为 Minecraft 命令 (`.mcfunction`). MCScript 的目标是让 Minecraft 命令的编写变得容易, 从而让不熟悉命令的 Minecraft 玩家也能容易地自定义自己的游戏体验. 

## 使用
MCScript 支持输出 Minecraft Java 版 1.21 版本的数据包 (数据包版本 48). 以下命令指定了输入文件为 `my_datapack.mcs`, 输出名为 `my_datapack` 的数据包 (包含输入文件中的函数). 

```sh
mcscript my_datapack.mcs -o my_datapack
```

此外, 编译器还会输出名为 `mcscript` 的数据包, 其中包含一些通用函数. 

将两个数据包复制到 `.minecraft/saves/<存档名字>/datapacks/` 后, 进入游戏. 首次使用 mcscript 生成的数据包, 需要运行命令 `/function mcscript:init` 进行初始化. 要调用 `my_datapack.mcs` 中名为 `func` 的函数, 输入以下命令即可: 

```
/function my_datapack:func
```

目前 MCScript 暂未支持输入和输出. 要查看函数运行后各个寄存器(尤其是 `return_value` 寄存器)的值, 请运行命令 `/scoreboard objectives setdisplay sidebar registers`.

## 语法示例

MCScript 的语法很大程度上参考了 Rust (但是语义和 Rust 并不相同). 

### 函数

MCScript 源代码的文件扩展名是 `mcs`. 一个 `mcs` 文件由零个或几个函数构成. 一个函数可以有零个或多个参数. 以下是一个函数定义的示例: 

```
fn func(param1, param2, param3) {
    return param1;
}
```

函数的返回值会保存在寄存器 `return_value` 中, 你可以通过命令 `/scoreboard players get return_value registers` 查看. 

### 变量

由于 Minecraft 命令中算术运算只能通过记分板较好地实现, MCScript 中所有的变量都是 32 位整数类型. 声明一个变量时, 必须指定它的初始值: 

```
let a = 114514;
```

MCScript 暂只支持局部变量. 

### 运算

MCScript 支持的一元运算符有 `+` `-`; 支持的二元运算符有 `+` `-` `*` `/` `%` `>` `>=` `<` `<=` `==` `!=`. 支持的赋值运算符有 `=` `+=` `-=` `*=` `/=` `%=`. 

### 分支

下面是一些分支语句的示例: 

```
// 无 else 分支
if a > 10 {
    return a - 10;
}
return a;
```

``` 
// 有 else 分支
if a > 10 {
    return a - 10;
} else {
    return a;
}
```

```
// 有多个分支
if a > 10 {
    return a - 10;
} else if a > 5 {
    return a - 5;
} else {
    return a;
}
```

与 Rust 相似, 分支语句中的判别表达式的括号不是必需的, 但是后面的语句块的大括号是必需的. 

### 循环

MCScript 中有 `while` 循环. 下面是一个示例: 

```
let sum = 0;
let n = 100;
while n > 0 {
    sum += n;
    n -= 1;
}
```

`while` 循环中可以使用 `break;` 和 `continue;` 语句. 

### 递归

下面是使用递归方式计算斐波那契数列通项的示例: 

```
fn fib(n) {
    if n < 3 {
        return 1;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() {
    return fib(10);
}
```

请注意, 由于游戏规则 `maxCommandChainLength` 的限制, 同一游戏刻内执行的最大命令数量为 65536. 在默认情况下, 使用递归方法计算像 `fib(16)` 这样的值可能会发生栈溢出. (如果在计算之后寄存器 `base_index` 的值不为 -1, 则发生了栈溢出, 此时需要重新运行命令 `/function mcscript:init` 进行初始化. ) 此时, 你可以通过 `/gamerule maxCommandChainLength 2147483647` 来放宽这个限制. 

