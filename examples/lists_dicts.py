# List operations
numbers = [1, 2, 3, 4, 5]
print(numbers)

# List indexing
print(numbers[0])
print(numbers[2])

# List assignment
numbers[1] = 99
print(numbers)

# List methods
numbers.append(6)
print(numbers)

x = numbers.pop()
print(x)
print(numbers)

# Dictionary with string keys
person = {"name": "Alice", "age": 30}
print(person)

print(person["name"])
print(person["age"])

# Dictionary with int keys
scores = {1: 100, 2: 95, 3: 88}
print(scores)
print(scores[1])

# Dictionary assignment
person["city"] = "Beijing"
scores[4] = 92
print(person)
print(scores)

# Dictionary keys
keys = person.keys()
print(keys)

# len() function
print(len(numbers))
print(len(person))
print(len("hello"))

# Nested structures
matrix = [[1, 2], [3, 4], [5, 6]]
print(matrix)
print(matrix[0])
print(matrix[1][0])
