# 字典操作演示

# 1. 字典字面量 - 字符串键
print("=== 字典字面量 - 字符串键 ===")
person = {"name": "Alice", "age": 30, "city": "Beijing"}
print(person)

# 2. 字典字面量 - 整数键
print("\n=== 字典字面量 - 整数键 ===")
scores = {1: 100, 2: 95, 3: 88}
print(scores)

# 3. 空字典
print("\n=== 空字典 ===")
empty = {}
print(empty)

# 4. 字典访问 - 字符串键
print("\n=== 字典访问 - 字符串键 ===")
print(person["name"])
print(person["age"])

# 5. 字典访问 - 整数键
print("\n=== 字典访问 - 整数键 ===")
print(scores[1])
print(scores[2])

# 6. 字典赋值 - 添加新键
print("\n=== 字典赋值 - 添加新键 ===")
person["job"] = "Engineer"
print(person)

scores[4] = 92
print(scores)

# 7. 字典赋值 - 修改现有键
print("\n=== 字典赋值 - 修改现有键 ===")
person["age"] = 31
print(person)

# 8. 字典方法 - keys()
print("\n=== 字典方法 - keys() ===")
keys = person.keys()
print(keys)

# 9. len() 函数
print("\n=== len() 函数 ===")
print(len(person))
print(len(scores))

# 10. 嵌套字典
print("\n=== 嵌套字典 ===")
users = {
    "alice": {"age": 30, "city": "Beijing"},
    "bob": {"age": 25, "city": "Shanghai"},
}
print(users)
print(users["alice"])
print(users["alice"]["city"])
