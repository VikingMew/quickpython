# Example 15: Regular expressions (re module)

import re

# Basic match - matches from start of string
m = re.match(r"hello", "hello world")
if m:
    print(m.group(0))

# Search - finds pattern anywhere in string
m = re.search(r"world", "hello world")
if m:
    print(m.group(0))

# Find all matches
matches = re.findall(r"\d+", "abc 123 def 456 ghi")
print(len(matches))
print(matches[0])
print(matches[1])

# Replace pattern
result = re.sub(r"\d+", "X", "abc 123 def 456")
print(result)

# Split by pattern
parts = re.split(r"\s+", "hello  world   test")
print(len(parts))
print(parts[0])
print(parts[1])
print(parts[2])

# Capture groups
m = re.search(r"(\d+)-(\d+)", "Phone: 123-456")
if m:
    print(m.group(0))
    print(m.group(1))
    print(m.group(2))

# Match position
m = re.search(r"world", "hello world")
if m:
    print(m.start())
    print(m.end())
    span = m.span()
    print(span[0])
    print(span[1])

# No match returns None
m = re.match(r"world", "hello world")
print(m)
