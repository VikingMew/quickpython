# 异常处理演示

# 1. try-except - 基本用法
print("=== try-except - 基本用法 ===")
try:
    x = 1 / 0
except ZeroDivisionError:
    print("Cannot divide by zero!")

# 2. try-except - 捕获异常并绑定到变量
print("\n=== try-except - 捕获异常并绑定到变量 ===")
try:
    numbers = [1, 2, 3]
    x = numbers[10]
except IndexError as e:
    print("Caught IndexError")
    print(e)

# 3. try-except - 多个 except 子句
print("\n=== try-except - 多个 except 子句 ===")


def test_exception(error_type):
    try:
        if error_type == 1:
            x = 1 / 0
        else:
            x = [1][10]
    except ZeroDivisionError:
        print("Caught ZeroDivisionError")
    except IndexError:
        print("Caught IndexError")


test_exception(1)
test_exception(2)

# 4. try-except - 捕获所有异常
print("\n=== try-except - 捕获所有异常 ===")
try:
    raise ValueError("Some error")
except Exception:
    print("Caught an exception")

# 5. raise - 抛出异常
print("\n=== raise - 抛出异常 ===")
try:
    raise ValueError("Custom error message")
except ValueError as e:
    print("Caught ValueError:")
    print(e)

# 6. try-finally - 确保清理代码执行
print("\n=== try-finally - 确保清理代码执行 ===")
try:
    print("Doing something")
    x = 10
finally:
    print("Cleanup code always runs")

# 7. try-except-finally - 完整形式
print("\n=== try-except-finally - 完整形式 ===")
try:
    x = 1 / 0
except ZeroDivisionError:
    print("Caught exception")
finally:
    print("Finally block")

# 8. 嵌套 try-except
print("\n=== 嵌套 try-except ===")
try:
    try:
        x = 1 / 0
    except ValueError:
        print("Inner: ValueError")
except ZeroDivisionError:
    print("Outer: ZeroDivisionError")

# 9. 函数中的异常处理
print("\n=== 函数中的异常处理 ===")


def safe_divide(a, b):
    try:
        return a / b
    except ZeroDivisionError:
        print("Cannot divide by zero")
        return None


result = safe_divide(10, 2)
print(result)

result = safe_divide(10, 0)
print(result)

# 10. 不同的异常类型
print("\n=== 不同的异常类型 ===")

# ValueError
try:
    raise ValueError("Invalid value")
except ValueError:
    print("Caught ValueError")

# TypeError
try:
    raise TypeError("Type error")
except TypeError:
    print("Caught TypeError")

# KeyError
try:
    d = {"a": 1}
    x = d["b"]
except KeyError:
    print("Caught KeyError")

# RuntimeError
try:
    raise RuntimeError("Runtime error")
except RuntimeError:
    print("Caught RuntimeError")
