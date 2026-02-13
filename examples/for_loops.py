# For loops and iterators examples

# Example 1: Simple range iteration
print("Example 1: Sum of 0 to 9")
sum = 0
for i in range(10):
    sum = sum + i
print(sum)  # 45

# Example 2: Range with start and stop
print("Example 2: Sum from 5 to 9")
sum = 0
for i in range(5, 10):
    sum = sum + i
print(sum)  # 35

# Example 3: Range with step
print("Example 3: Sum of even numbers 0 to 8")
sum = 0
for i in range(0, 10, 2):
    sum = sum + i
print(sum)  # 20

# Example 4: Iterating over a list
print("Example 4: Sum of list elements")
numbers = [10, 20, 30, 40, 50]
sum = 0
for num in numbers:
    sum = sum + num
print(sum)  # 150

# Example 5: Iterating over dictionary keys
print("Example 5: Sum of dictionary keys")
scores = {1: 100, 2: 95, 3: 88}
sum = 0
for key in scores:
    sum = sum + key
print(sum)  # 6

# Example 6: Nested for loops
print("Example 6: Multiplication table")
for i in range(1, 4):
    for j in range(1, 4):
        product = i * j
        print(product)

# Example 7: For loop in a function
print("Example 7: Factorial using for loop")


def factorial(n):
    result = 1
    for i in range(1, n + 1):
        result = result * i
    return result


print(factorial(5))  # 120

# Example 8: Building a list with for loop
print("Example 8: Squares of numbers")
squares = []
for i in range(5):
    squares.append(i * i)
print(squares)  # [0, 1, 4, 9, 16]
