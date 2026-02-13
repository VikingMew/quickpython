# for 循环演示

# 1. range() - 单参数
print("=== range() - 单参数 ===")
for i in range(5):
    print(i)

# 2. range() - 两个参数
print("\n=== range() - 两个参数 ===")
for i in range(1, 6):
    print(i)

# 3. range() - 三个参数（步长）
print("\n=== range() - 三个参数（步长） ===")
for i in range(0, 10, 2):
    print(i)

# 4. 遍历列表
print("\n=== 遍历列表 ===")
fruits = ["apple", "banana", "orange"]
for fruit in fruits:
    print(fruit)

# 5. 遍历列表求和
print("\n=== 遍历列表求和 ===")
numbers = [1, 2, 3, 4, 5]
total = 0
for n in numbers:
    total = total + n
print(total)

# 6. 遍历字典（遍历键）
print("\n=== 遍历字典（遍历键） ===")
person = {"name": "Alice", "age": 30}
for key in person:
    print(key)

# 7. 遍历字典的 keys()
print("\n=== 遍历字典的 keys() ===")
for key in person.keys():
    print(key)
    print(person[key])

# 8. 嵌套 for 循环
print("\n=== 嵌套 for 循环 ===")
for i in range(3):
    for j in range(3):
        print(i)
        print(j)

# 9. 使用 for 构建列表
print("\n=== 使用 for 构建列表 ===")
squares = []
for i in range(1, 6):
    squares.append(i * i)
print(squares)
