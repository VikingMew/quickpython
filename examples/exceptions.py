# Exception Handling Examples

# Basic try-except
print("Basic try-except:")
try:
    x = 1 / 0
except ZeroDivisionError:
    print("Cannot divide by zero!")

# Multiple except clauses
print("\nMultiple except clauses:")
numbers = [1, 2, 3]
try:
    result = numbers[10]
except IndexError:
    print("Index out of range")
except KeyError:
    print("Key not found")

# Exception with variable binding
print("\nException with binding:")
try:
    raise ValueError("Something went wrong")
except ValueError as e:
    print("Caught error")

# Try-finally
print("\nTry-finally:")
try:
    print("Trying...")
finally:
    print("Cleanup always runs")

# Try-except-finally
print("\nTry-except-finally:")
try:
    x = 1 / 0
except ZeroDivisionError:
    print("Handled division by zero")
finally:
    print("Cleanup complete")
