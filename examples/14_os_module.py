# Example 14: OS module

import os

# Get current working directory
cwd = os.getcwd()
print(cwd)

# List directory contents
files = os.listdir(".")
print(len(files))

# Path operations
path = os.path.join("examples", "test.py")
print(path)

exists = os.path.exists("Cargo.toml")
print(exists)

basename = os.path.basename("/path/to/file.txt")
print(basename)

dirname = os.path.dirname("/path/to/file.txt")
print(dirname)

# Check file types
is_file = os.path.isfile("Cargo.toml")
print(is_file)

is_dir = os.path.isdir("src")
print(is_dir)

# Environment variables
home = os.getenv("HOME", "/default")
print(home)

# System information
print(os.name)

# Create and remove directory
os.mkdir("test_dir_example")
exists_before = os.path.exists("test_dir_example")
print(exists_before)

os.rmdir("test_dir_example")
exists_after = os.path.exists("test_dir_example")
print(exists_after)
