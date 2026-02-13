# break 和 continue 演示

# 1. break - 跳出 for 循环
print("=== break - 跳出 for 循环 ===")
for i in range(10):
    if i == 5:
        break
    print(i)

# 2. break - 跳出 while 循环
print("\n=== break - 跳出 while 循环 ===")
i = 0
while True:
    if i >= 5:
        break
    print(i)
    i = i + 1

# 3. continue - 跳过 for 循环
print("\n=== continue - 跳过 for 循环 ===")
for i in range(10):
    if i % 2 == 0:
        continue
    print(i)

# 4. continue - 跳过 while 循环
print("\n=== continue - 跳过 while 循环 ===")
i = 0
while i < 10:
    i = i + 1
    if i % 2 == 0:
        continue
    print(i)

# 5. break - 查找元素
print("\n=== break - 查找元素 ===")
numbers = [1, 3, 5, 7, 9, 11]
target = 7
found = False
for n in numbers:
    if n == target:
        found = True
        break
if found:
    print("Found!")
else:
    print("Not found")

# 6. 嵌套循环中的 break（只跳出内层）
print("\n=== 嵌套循环中的 break ===")
for i in range(3):
    for j in range(5):
        if j == 2:
            break
        print(i)
        print(j)

# 7. continue - 过滤偶数
print("\n=== continue - 过滤偶数 ===")
numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
for n in numbers:
    if n % 2 == 0:
        continue
    print(n)

# 8. break 和 continue 组合
print("\n=== break 和 continue 组合 ===")
for i in range(20):
    if i % 3 == 0:
        continue
    if i > 10:
        break
    print(i)
