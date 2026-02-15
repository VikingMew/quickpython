# Example: Using asyncio.sleep for true async behavior

import asyncio


# Basic async function with sleep
async def greet_after_delay(name, delay):
    print(f"Starting to greet {name}...")
    await asyncio.sleep(delay)
    print(f"Hello, {name}!")
    return f"Greeted {name}"


# Test basic async/await
print("=== Basic Async/Await ===")
result = await greet_after_delay("Alice", 0.1)
print(f"Result: {result}")


# Multiple sequential sleeps
async def count_with_delays():
    for i in range(3):
        print(f"Count: {i}")
        await asyncio.sleep(0.05)
    return "Done counting"


print("\n=== Sequential Sleeps ===")
result = await count_with_delays()
print(f"Result: {result}")


# Async function calling another async function
async def inner_task(x):
    await asyncio.sleep(0.05)
    return x * 2


async def outer_task(x):
    print(f"Outer task starting with {x}")
    result = await inner_task(x)
    print(f"Inner task returned {result}")
    return result + 1


print("\n=== Nested Async Calls ===")
result = await outer_task(5)
print(f"Final result: {result}")

# Demonstrating that sleep actually blocks
print("\n=== Timing Test ===")


async def timed_operation():
    print("Starting 200ms sleep...")
    await asyncio.sleep(0.2)
    print("Sleep completed!")
    return "done"


result = await timed_operation()
print(f"Result: {result}")
