# Iterator Safety Examples

# Safe iteration - no modification
print("Safe iteration:")
numbers = [1, 2, 3, 4, 5]
for n in numbers:
    print(n)

# Modification after loop is OK
print("\nModification after loop:")
numbers = [1, 2, 3]
for n in numbers:
    print(n)
numbers.append(4)
print("List after loop:", numbers)

# Unsafe iteration - modification during loop
print("\nUnsafe iteration (will raise IteratorError):")
try:
    numbers = [1, 2, 3]
    for n in numbers:
        numbers.append(10)  # This will raise IteratorError
except IteratorError:
    print("Caught IteratorError: cannot modify list during iteration")

# Another unsafe example - pop during iteration
print("\nAnother unsafe example:")
try:
    items = [1, 2, 3, 4, 5]
    for item in items:
        items.pop()  # This will raise IteratorError
except IteratorError:
    print("Caught IteratorError: cannot pop from list during iteration")
