# 实现的功能

实验实现了`sys_trace`

 扩展了`TaskManagerInner` ，在`TaskControlBlock`中添加了结构体来保存系统调用次数。

在`mod.rs`中添加了`record_syscall_num`方法和`get_syscall_num`方法

在`syscall`调用`record_syscall_num`记录次数

`trace_request`为0时读取当前任务 `id` 地址处一个字节；为1时写入一个字节；为2时调用`get_syscall_num`,查询并返回指定系统调用的次数，其他请求返回-1

# 简答题

1正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 [三个 bad 测例 (ch2b_bad_*.rs)](https://github.com/LearningOS/rCore-Tutorial-Test/tree/master/src/bin) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。

sbi及其版本:

```text
[rustsbi] RustSBI version 0.3.0-alpha.2, adapting to RISC-V SBI v1.0.0
[rustsbi] Implementation     : RustSBI-QEMU Version 0.2.0-alpha.2
```

第一个

```
[kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
```

`ch2b_bad_address.rs`要向0x0写入一个值，bad addr = 0x0是说内存访问不合法，然后就触发了`PageFault`的异常

第二个

```
[kernel] IllegalInstruction in application, kernel killed it.
```

这里已经进入了U态，但是`  core::arch::asm!("sret");`这里还是使用了S态特权指令，所以触发了`IllegalInstruction`的异常

第三个

```
[kernel] IllegalInstruction in application, kernel killed it.
```

这里`   core::arch::asm!("csrr {}, sstatus", out(reg) sstatus);`在U态要读取S态的CSR，会触发`IllegalInstruction`的异常



2深入理解 [trap.S](https://github.com/LearningOS/rCore-Tutorial-Code/blob/ch3/os/src/trap/trap.S) 中两个函数 `__alltraps` 和 `__restore` 的作用，并回答如下问题:

1. L40：刚进入 `__restore` 时，`sp` 代表了什么值。请指出 `__restore` 的两种使用情景。

刚进入时候`sp`代表`TrapContext`的起始地址。`__restore`两种用途分别是首次启动应用程序和处理Trap后返回用户态

2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

   ```
   ld t0, 32*8(sp)
   ld t1, 33*8(sp)
   ld t2, 2*8(sp)
   csrw sstatus, t0
   csrw sepc, t1
   csrw sscratch, t2
   ```

   特殊处理了`sstatus` `sepc` `sscratch`这几个寄存器。

   `sstatus`决定了返回哪一个特权级。

   `sepc`决定了执行`sret`之后，CPU跳到的地址，进入用户态后从哪里执行。

   `sscratch`中保存的是之前的`sp` L13中`csrrw sp, sscratch, sp`在一开始交换了`sp`和`sscratch`。`sscratch`交换后保存用户栈指针。

3. L50-L56：为何跳过了 `x2` 和 `x4`？

   ```
   ld x1, 1*8(sp)
   ld x3, 3*8(sp)
   .set n, 5
   .rept 27
      LOAD_GP %n
      .set n, n+1
   .endr
   ```

   `x2`是`sp`，在L58处`  addi sp, sp, 34*8`还用了`sp`,所以这里跳过了。`x4`是`tp`,L22提到`# skip tp(x4), application does not use it`，我们没有用它，所以可以跳过

4. L60：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

   ```
   csrrw sp, sscratch, sp
   ```

   `sp`和`sscratch`在L60交换了，`sp`保存用户栈指针，给U态使用，`sscratch`保存内核栈指针，给S态使用

5. `__restore`：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？

发生在L61的`sret` 执行时根据`sstatus`中`SPP` = 0切换到用户态

6. L13：该指令之后，`sp` 和 `sscratch` 中的值分别有什么意义？

   ```
   csrrw sp, sscratch, sp
   ```

   `sp`保存内核栈指针，用来给`TrapContext`分配空间，`sscratch`保存`Trap`前的用户栈指针，L60`sp`和`sscratch`交换，就可以给返回用户态使用

7. 从 U 态进入 S 态是哪一条指令发生的？

 是`ecall`发生的

# 荣誉规则

1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 **以下各位** 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：

   > *《你交流的对象说明》*
   >
   > 主要的交流对象为Chatgpt。主要用它帮我翻译和讲解了**《RISC-V 指令集手册，第二卷：特权架构》** **《RISC-V 特权架构》讲义**和**《RISC-V 手册：一本开源指令集的指南》**中关于CSR寄存器，汇编指令的部分。

2. 我也参考了 **以下资料** 

   > *《你参考的资料说明》*
   >
   > 参考了《RISC-V 指令集手册，第二卷：特权架构》《RISC-V 特权架构》讲义和《RISC-V 手册：一本开源指令集的指南》以及rCore Tutorial Book v3

3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。

4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
