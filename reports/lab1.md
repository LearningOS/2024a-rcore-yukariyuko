# Lab1实验报告
>荣誉准则
    1.在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
        无
    2.此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
        无
        3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
        4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。

## 实现功能
本实验中，我实现了系统调用~sys_task_info~以查询当前正在执行的任务信息，任务信息包括任务控制块相关信息（任务状态）、任务使用的系统调用及调用次数、系统调用时刻距离任务第一次被调度时刻的时长（单位ms）。为实现此功能，我主要修改了~os/src/task/mod.rs~在其中新增了一些函数，并为~TaskControlBlock~结构体增加了两个字段
## 简答作业
1. 正确进入 U 态后，程序的特征还应有：使用 S 态特权指令，访问 S 态寄存器后会报错。 请同学们可以自行测试这些内容（运行 三个 bad 测例 (ch2b_bad_*.rs) ）， 描述程序出错行为，同时注意注明你使用的 sbi 及其版本。
    终端输出如下：

        RustSBI-QEMU Version 0.2.0-alpha.3
        [kernel] Loading app_0
        [kernel] PageFault in application, kernel killed it.
        [kernel] Loading app_1
        [kernel] IllegalInstruction in application, kernel killed it.
        [kernel] Loading app_2
        [kernel] IllegalInstruction in application, kernel killed it.
    可以看到，app0访问了无权限访问的地址0x0,app1执行了无权限执行的指令sret，app2访问了无权限访问的寄存器sstatus，他们均被检查出来并拒绝了

2. 深入理解 trap.S 中两个函数 __alltraps 和 __restore 的作用，并回答如下问题:

    1. L40：刚进入 __restore 时，a0 代表了什么值。请指出 __restore 的两种使用情景。
        a0是传入的参数~&TrapContext~
        __restore用于:
        - 从保存在内核栈上的 Trap 上下文恢复寄存器
        - 从用户模式返回内核模式

    2. L43-L48：这几行汇编代码特殊处理了哪些寄存器？这些寄存器的的值对于进入用户态有何意义？请分别解释。

            ld t0, 32*8(sp)
            ld t1, 33*8(sp)
            ld t2, 2*8(sp)
            csrw sstatus, t0
            csrw sepc, t1
            csrw sscratch, t2
    sstatus 给出 Trap 发生之前 CPU 处在哪个特权级（S/U）等信息
    sepc 当 Trap 是一个异常的时候，记录 Trap 发生之前执行的最后一条指令的地址
	sscratch 在发生异常或中断时存储内核特权级的临时数据，通常用于保存栈指针或其他快速访问的控制信息。

    3. L50-L56：为何跳过了 x2 和 x4？

            ld x1, 1*8(sp)
            ld x3, 3*8(sp)
            .set n, 5
            .rept 27
               LOAD_GP %n
               .set n, n+1
            .endr
    
    x4除非手动出于一些特殊用途使用它，否则一般不会被用到。
    x2的信息保存在 sscratch 中
    
    4. L60：该指令之后，sp 和 sscratch 中的值分别有什么意义？

            csrrw sp, sscratch, sp
    sp：用户模式的堆栈指针。
    sscratch：内核模式的堆栈指针。

    5. __restore：中发生状态切换在哪一条指令？为何该指令执行之后会进入用户态？
    sret 指令执行之后，处理器会根据 sstatus 寄存器中的 SPP 位切换到用户模式，并从 sepc 寄存器中取出返回地址，继续执行用户模式下的代码。

    6. L13：该指令之后，sp 和 sscratch 中的值分别有什么意义？

            csrrw sp, sscratch, sp
    sp：内核模式的堆栈指针。
    sscratch：用户模式的堆栈指针。
    7. 从 U 态进入 S 态是哪一条指令发生的？
        
            csrr t2, sscratch
    这条指令读取 sscratch 寄存器的值到 t2 寄存器中。此时，处理器已经在内核态（S 态），并开始执行内核态的异常或中断处理程序