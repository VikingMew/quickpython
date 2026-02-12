# Float literals and arithmetic
pi = 3.14159
radius = 5.0
area = pi * radius * radius
print(area)

# Mixed int and float operations
x = 10
y = 3.0
result = x / y
print(result)

# Type conversions
a = 3.14
b = int(a)
print(b)

c = 42
d = float(c)
print(d)


# More complex calculations
def circle_area(r):
    return 3.14159 * r * r


print(circle_area(10.0))
print(circle_area(5))
