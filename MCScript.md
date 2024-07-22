## 语法示例

MCScript 的语法很大程度上参考了 Rust (但是语义和 Rust 并不相同). 

### 局部变量

MCScript 中有两种数据类型: 整数和数组. 
声明一个变量时, 必须指定它的初始值: 

```
let length = 10; // 声明一个整数
let value = 114514; // 声明另一个整数
let arr = new Array(length, value); // 声明一个整数的数组, 长度为 length, 用 value 填充. 
let mat = new Array(length, new Array(length, value)); // 声明一个整数的数组的数组
```

声明局部变量时不用指定类型, MCScript 编译器会自动推导局部变量的类型. 

### 运算

MCScript 支持的一元运算符有 `+` `-`; 支持的二元运算符有 `+` `-` `*` `/` `%` `>` `>=` `<` `<=` `==` `!=`. 支持的赋值运算符有 `=` `+=` `-=` `*=` `/=` `%=`. 

### 函数和命名空间

MCScript 源代码的文件扩展名是 `mcs`. 一个 `mcs` 文件代表了一个命名空间, 其中包含零个或以上的函数. 一个函数可以有零个或以上的参数. 函数的参数类型和返回值类型需要显式标记. 以下是一个函数定义的示例: 

```
fn func(param1: int, param2: Array<int>) -> Array<Array<int>> {
    return new Array(param1, param2);
}
```

函数的名字不能为 `init`.

如果函数没有返回值, 省略返回值类型标记即可: 

```
fn main() {
    return 0;
}
```

函数的返回值会保存在寄存器 `return_value` (整数)或者命令存储 `memory:temp return_object` (数组)中, 你可以通过命令 `/scoreboard players get return_value registers` 或 `/data get storage memory:temp return_object` 查看. 

调用同一个文件(命名空间)内的函数时, 可以直接使用函数的名字; 调用其他文件(命名空间)内的函数时, 需要加上命名空间前缀: 

```
// namespace1.mcs
fn f1() {}
```

```
// namespace2.mcs
fn f2() {}
fn f3() {
    namespace1::f1(); // 调用命名空间 namespace1 中的函数
    f2(); // 调用本命名空间中的函数
}
```

MCScript 中的命名空间不能嵌套, 这是因为 Minecraft 数据包中的命名空间不能嵌套. 

### 作用域

一对大括号 `{` `}` 中间的部分构成了一个作用域. 子作用域中的变量会掩盖父作用域中的变量. 例如下面的例子中的函数会返回1.

```
fn main() -> int {
    let a = 0;
    {
        let a = 1;
        return a;
    }
}
```

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
fn fib(n: int) -> int {
    if n < 3 {
        return 1;
    }
    return fib(n - 1) + fib(n - 2);
}

fn main() -> int {
    return fib(10);
}
```

请注意, 由于游戏规则 `maxCommandChainLength` 的限制, 同一游戏刻内执行的最大命令数量为 65536. 在默认情况下, 使用递归方法计算像 `fib(16)` 这样的值可能会发生栈溢出. (如果在计算之后寄存器 `base_index` 的值不为 -1, 则发生了栈溢出, 此时需要重新运行命令 `/function mcscript:init` 进行初始化. ) 此时, 你可以通过 `/gamerule maxCommandChainLength 2147483647` 来放宽这个限制. 

### 数组

MCScript 中的数组通过 `new Array(<length>, <value>)` 创建. 通过 `[]` 取数组中某个元素: 

```
let arr = new Array(10, 1);
arr[2] = arr[1] * arr[0];
```

MCScript 中的数组的类型写作 `Array<element_type>`, 其中 `element_type` 是数组的元素的类型. 

MCScript 中的数组在赋值, 作为函数参数和返回值时, 是按值传递的. 这与其他一些带有 GC 的语言不同, 请注意区别. 

下面是使用数组计算斐波那契数列通项的示例: 

```
fn fib(n: int) -> int {
    let res = new Array(n + 1, 1);
    let i = 3;
    while i <= n {
        res[i] = res[i - 1] + res[i - 2];
        i += 1;
    }
    return res[n];
}

fn main() -> int {
    return fib(40);
}
```

### 内联命令

MCScript 与 Minecraft 世界交互的方式是通过内联命令. 你可以在函数内通过 `run_command!` 运行一条游戏内命令: 

```
fn hello_world() {
    run_command!("say Hello, world! ");
}
```

`run_command!` 可以接受格式化参数: 

```
fn show_result() {
    let x = 1;
    let y = 2;
    run_command!("say {} + {} = {}", x, y, x + y);
}
```

### 全局变量

在 MCScript 中声明全局变量时需要指定初始值和类型: 

```
// namespace1.mcs
let g_a: int = 1;
let g_arr_1: Array<int> = new Array(10, 1);
```

使用相同命名空间中的全局变量时, 可以省略命名空间; 使用其他命名空间中的全局变量时, 需要指定命名空间: 

```
// namespace1.mcs
fn f1() {
    g_a += 1;
}
```

```
// namespace2.mcs
fn f1() {
    namespace1::g_a += 1;
}
```

在游戏内, 需要手动运行 `/function <namespace>:init` 来初始化命名空间 `<namespace>` 中声明的全局变量. 例如运行 `/function namespace1:init` 来初始化上面示例中的全局变量. 