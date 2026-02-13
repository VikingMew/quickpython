# Break and Continue Examples

# Break example - find first even number
print("Find first even number:")
numbers = [1, 3, 5, 8, 9, 10]
for n in numbers:
    if n / 2 * 2 == n:
        print("Found:", n)
        break

# Continue example - skip odd numbers
print("\nPrint only even numbers:")
for i in range(10):
    if i / 2 * 2 != i:
        continue
    print(i)

# Nested loops with break
print("\nNested loops with break:")
for i in range(3):
    for j in range(3):
        if j == 1:
            break
        print(i, j)
