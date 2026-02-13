# 列表操作演示

# 1. 列表字面量
print("=== 列表字面量 ===")
numbers = [1, 2, 3, 4, 5]
print(numbers)

empty = []
print(empty)

mixed = [1, "hello", True, 3.14]
print(mixed)

# 2. 列表索引访问
print("\n=== 列表索引访问 ===")
print(numbers[0])
print(numbers[2])
print(numbers[4])

# 3. 负数索引
print("\n=== 负数索引 ===")
print(numbers[-1])
print(numbers[-2])

# 4. 列表索引赋值
print("\n=== 列表索引赋值 ===")
numbers[1] = 99
print(numbers)

# 5. 列表方法 - append
print("\n=== 列表方法 - append ===")
fruits = ["apple", "banana"]
fruits.append("orange")
print(fruits)

# 6. 列表方法 - pop
print("\n=== 列表方法 - pop ===")
last = fruits.pop()
print(last)
print(fruits)

# 7. len() 函数
print("\n=== len() 函数 ===")
print(len(numbers))
print(len(fruits))
print(len([]))

# 8. 嵌套列表
print("\n=== 嵌套列表 ===")
matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]
print(matrix)
print(matrix[0])
print(matrix[1][1])

# 9. 修改嵌套列表
print("\n=== 修改嵌套列表 ===")
matrix[0][0] = 99
print(matrix)
