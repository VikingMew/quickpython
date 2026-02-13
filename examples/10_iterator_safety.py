# 迭代器安全演示

# 1. 安全的迭代（不修改）
print("=== 安全的迭代（不修改） ===")
numbers = [1, 2, 3, 4, 5]
for n in numbers:
    print(n)
print("Iteration completed successfully")

# 2. 循环后修改列表（允许）
print("\n=== 循环后修改列表（允许） ===")
numbers = [1, 2, 3]
for n in numbers:
    print(n)
numbers.append(4)
print(numbers)

# 3. 循环中修改列表 - append（检测到错误）
print("\n=== 循环中修改列表 - append ===")
numbers = [1, 2, 3]
try:
    for n in numbers:
        print(n)
        numbers.append(10)
except IteratorError:
    print("Caught IteratorError: list modified during iteration")

# 4. 循环中修改列表 - pop（检测到错误）
print("\n=== 循环中修改列表 - pop ===")
numbers = [1, 2, 3, 4, 5]
try:
    for n in numbers:
        print(n)
        if n == 2:
            numbers.pop()
except IteratorError:
    print("Caught IteratorError: list modified during iteration")

# 5. 循环中修改列表 - 索引赋值（检测到错误）
print("\n=== 循环中修改列表 - 索引赋值 ===")
numbers = [1, 2, 3]
try:
    for n in numbers:
        print(n)
        numbers[0] = 99
except IteratorError:
    print("Caught IteratorError: list modified during iteration")

# 6. 正确的做法 - 遍历副本
print("\n=== 正确的做法 - 遍历副本（暂不支持） ===")
print("注意：当前版本不支持列表切片，需要手动创建副本")

# 7. 嵌套循环中的修改检测
print("\n=== 嵌套循环中的修改检测 ===")
outer = [1, 2, 3]
try:
    for x in outer:
        print(x)
        inner = [10, 20, 30]
        for y in inner:
            print(y)
            outer.append(99)
except IteratorError:
    print("Caught IteratorError in nested loop")

# 8. 版本号机制说明
print("\n=== 版本号机制说明 ===")
print("QuickPython 使用版本号跟踪列表修改")
print("每次修改（append/pop/索引赋值）都会增加版本号")
print("迭代器在每次迭代时检查版本号是否改变")
print("如果改变，抛出 IteratorError")
