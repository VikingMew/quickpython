# 控制流演示

# 1. if 语句
print("=== if 语句 ===")
x = 10
if x > 5:
    print("x is greater than 5")

# 2. if-else 语句
print("\n=== if-else 语句 ===")
y = 3
if y > 5:
    print("y is greater than 5")
else:
    print("y is not greater than 5")

# 3. 嵌套 if
print("\n=== 嵌套 if ===")
score = 85
if score >= 60:
    print("Pass")
    if score >= 90:
        print("Excellent!")
    else:
        print("Good")
else:
    print("Fail")

# 4. while 循环
print("\n=== while 循环 ===")
i = 0
while i < 5:
    print(i)
    i = i + 1

# 5. while 循环计算总和
print("\n=== while 循环计算总和 ===")
sum = 0
n = 1
while n <= 10:
    sum = sum + n
    n = n + 1
print(sum)
