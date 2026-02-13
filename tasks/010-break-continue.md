# 010: break 和 continue

**状态**: DONE  
**优先级**: P1

## 完成总结

✅ **所有功能实现完成！**

- ✅ break 可以跳出 for 和 while 循环
- ✅ continue 可以跳过当前迭代
- ✅ 嵌套循环中 break/continue 只影响内层循环
- ✅ 在循环外使用 break/continue 正确报错
- ✅ 添加了 12 个测试用例，全部通过
- ✅ 总测试数：70 passed; 0 failed

## 任务概述

支持 break 和 continue 语句，让循环控制更灵活。

## 可运行功能

完成后用户可以：
```rust
let mut ctx = Context::new();

// break 跳出循环
ctx.eval(r#"
for i in range(10):
    if i == 5:
        break
    print(i)
"#).unwrap();
// 输出: 0 1 2 3 4

// continue 跳过当前迭代
ctx.eval(r#"
for i in range(10):
    if i % 2 == 0:
        continue
    print(i)
"#).unwrap();
// 输出: 1 3 5 7 9

// while 循环中的 break
ctx.eval(r#"
i = 0
while True:
    if i >= 5:
        break
    print(i)
    i = i + 1
"#).unwrap();
// 输出: 0 1 2 3 4

// while 循环中的 continue
ctx.eval(r#"
i = 0
while i < 10:
    i = i + 1
    if i % 2 == 0:
        continue
    print(i)
"#).unwrap();
// 输出: 1 3 5 7 9

// 嵌套循环中的 break（只跳出内层）
ctx.eval(r#"
for i in range(3):
    for j in range(3):
        if j == 1:
            break
        print(i, j)
"#).unwrap();
// 输出: (0,0) (1,0) (2,0)

// 查找元素示例
ctx.eval(r#"
numbers = [1, 2, 3, 4, 5]
target = 3
found = False
for n in numbers:
    if n == target:
        found = True
        break
"#).unwrap();
let found = ctx.get("found").unwrap();
assert_eq!(found.as_bool(), Some(true));
```

**现在循环控制更灵活了！**

## 依赖任务

- 004: while 循环
- 009: for 循环和迭代器

## 需要实现的模块

### 1. 扩展字节码
- Break 指令
- Continue 指令

### 2. 扩展编译器
- break 语句编译
- continue 语句编译
- 循环标签管理（记录循环开始和结束位置）
- 嵌套循环处理

### 3. 扩展 VM
- 执行 Break 指令（跳转到循环结束）
- 执行 Continue 指令（跳转到循环开始）
- 循环栈管理

## 验收条件

- [ ] break 可以跳出 for 循环
- [ ] break 可以跳出 while 循环
- [ ] continue 可以跳过 for 循环当前迭代
- [ ] continue 可以跳过 while 循环当前迭代
- [ ] 嵌套循环中 break 只跳出内层循环
- [ ] 嵌套循环中 continue 只影响内层循环
- [ ] 在非循环中使用 break/continue 报错

## 测试要求

### 单元测试
- [ ] break 跳出 for 循环
- [ ] break 跳出 while 循环
- [ ] continue 在 for 循环中
- [ ] continue 在 while 循环中
- [ ] 嵌套循环的 break
- [ ] 嵌套循环的 continue

### 集成测试
- [ ] 查找元素（找到后 break）
- [ ] 过滤元素（不符合条件 continue）
- [ ] 复杂嵌套循环控制

## 增量实现步骤

### Step 1: 循环标签系统
- 编译器维护循环栈
- 记录每个循环的开始和结束位置

### Step 2: break 语句
- 添加 Break 指令
- 编译器生成跳转到循环结束的指令
- VM 执行 Break

### Step 3: continue 语句
- 添加 Continue 指令
- 编译器生成跳转到循环开始的指令
- VM 执行 Continue

### Step 4: 嵌套循环
- 确保 break/continue 只影响最内层循环
- 测试多层嵌套

### Step 5: 错误处理
- 在非循环中使用 break/continue 时报错

## 后续任务

完成后可以开始：
- 011: 更多运算符 (//, %, and, or, not)
- 012: elif 多分支条件
