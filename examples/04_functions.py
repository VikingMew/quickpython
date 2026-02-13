# 函数演示

# 1. 简单函数
print("=== 简单函数 ===")


def add(a, b):
    return a + b


result = add(10, 20)
print(result)

# 2. 无返回值函数
print("\n=== 无返回值函数 ===")


def greet(name):
    print("Hello, " + name)


greet("Alice")
greet("Bob")

# 3. 递归函数 - 阶乘
print("\n=== 递归函数 - 阶乘 ===")


def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)


print(factorial(5))
print(factorial(10))

# 4. 递归函数 - 斐波那契
print("\n=== 递归函数 - 斐波那契 ===")


def fibonacci(n):
    if n <= 1:
        return n
    else:
        return fibonacci(n - 1) + fibonacci(n - 2)


print(fibonacci(0))
print(fibonacci(1))
print(fibonacci(7))

# 5. 多个参数
print("\n=== 多个参数 ===")


def calculate(a, b, c):
    return a * b + c


print(calculate(2, 3, 4))

# 6. 函数调用其他函数
print("\n=== 函数调用其他函数 ===")


def square(x):
    return x * x


def sum_of_squares(a, b):
    return square(a) + square(b)


print(sum_of_squares(3, 4))
