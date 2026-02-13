# 比较运算符类型支持演示

print("=== 浮点数比较 ===")
x = 3.14
y = 2.71

print("x = 3.14, y = 2.71")
if x > y:
    print("x > y: True")
if x == y:
    print("x == y: True")
else:
    print("x == y: False")

print("\n=== 混合类型比较（int 和 float） ===")
score = 87.5
print("score = 87.5")

if score >= 90:
    print("Grade: A")
elif score >= 80:
    print("Grade: B")
elif score >= 70:
    print("Grade: C")
else:
    print("Grade: F")

print("\n10 == 10.0:", 10 == 10.0)
print("5 < 5.5:", 5 < 5.5)

print("\n=== 字符串比较 ===")
print('"hello" == "hello":', "hello" == "hello")
print('"hello" == "world":', "hello" == "world")
print('"apple" < "banana":', "apple" < "banana")
print('"zebra" > "apple":', "zebra" > "apple")

print("\n=== 布尔值比较 ===")
print("True == True:", True == True)
print("True == False:", True == False)

print("\n=== None 比较 ===")
x = None
print("x = None")
print("x == None:", x == None)
print("x != None:", x != None)

print("\n=== 不同类型比较 ===")
print('"hello" == 5:', "hello" == 5)
print("True == 5:", True == 5)

print("\n=== 实际应用：评分系统 ===")


def get_grade(score):
    if score >= 90.0:
        return "A"
    elif score >= 80.0:
        return "B"
    elif score >= 70.0:
        return "C"
    elif score >= 60.0:
        return "D"
    else:
        return "F"


scores = [95.5, 87.0, 72.5, 58.0]
print("分数:", scores)
for s in scores:
    grade = get_grade(s)
    print(s)
    print(grade)

print("\n=== 实际应用：字符串排序 ===")
names = ["Charlie", "Alice", "Bob"]
print("排序前:", names)

sorted_names = []
for name in names:
    inserted = False
    i = 0
    while i < len(sorted_names):
        if name < sorted_names[i]:
            sorted_names[i]
            inserted = True
            break
        i = i + 1
    if inserted == 0:
        sorted_names.append(name)

print("手动排序演示完成")
