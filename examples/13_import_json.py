# Example 13: Import json module

# Basic import
import json

# Parse JSON string
data = json.loads('{"name": "Alice", "age": 30, "active": true}')
print(data["name"])
print(data["age"])
print(data["active"])

# Serialize to JSON
obj = {"x": 1, "y": 2, "z": 3}
text = json.dumps(obj)
print(text)

# Parse JSON array
arr = json.loads("[1, 2, 3, 4, 5]")
print(len(arr))
print(arr[0])
print(arr[4])

# Nested JSON
nested = json.loads('{"user": {"name": "Bob", "id": 123}, "status": "active"}')
print(nested["user"]["name"])
print(nested["user"]["id"])
print(nested["status"])

# From import
from json import dumps, loads

data2 = loads('{"value": 42}')
print(data2["value"])

result = dumps({"test": True, "count": 10})
print(result)
