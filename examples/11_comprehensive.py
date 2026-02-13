# 综合示例 - 学生成绩管理（简化版）

print("=== 综合示例：学生成绩管理 ===")

# 使用整数成绩避免浮点数比较问题
students = [
    {"name": "Alice", "score": 88},
    {"name": "Bob", "score": 75},
    {"name": "Charlie", "score": 95},
]


# 评级函数（使用整数）
def get_grade(score):
    if score >= 90:
        return "A"
    else:
        if score >= 80:
            return "B"
        else:
            if score >= 70:
                return "C"
            else:
                if score >= 60:
                    return "D"
                else:
                    return "F"


# 打印学生信息
print("\n学生成绩报告：")
for student in students:
    name = student["name"]
    score = student["score"]
    grade = get_grade(score)

    print("\n学生：")
    print(name)
    print("成绩：")
    print(score)
    print("等级：")
    print(grade)

# 查找最高分
print("\n=== 查找最高分 ===")
max_score = 0
best_student = ""

for student in students:
    score = student["score"]
    if score > max_score:
        max_score = score
        best_student = student["name"]

print("最佳学生：")
print(best_student)
print("最高分：")
print(max_score)

# 统计优秀学生（>=90）
print("\n=== 统计优秀学生 ===")
excellent_count = 0
for student in students:
    if student["score"] >= 90:
        excellent_count = excellent_count + 1

print("优秀学生数：")
print(excellent_count)

# 使用 break 查找第一个及格学生
print("\n=== 查找第一个及格学生 ===")
found = 0
for student in students:
    if student["score"] >= 60:
        print("找到：")
        print(student["name"])
        found = 1
        break

if found == 0:
    print("未找到及格学生")

# 使用 continue 跳过不及格学生
print("\n=== 打印及格学生 ===")
for student in students:
    if student["score"] < 60:
        continue
    print(student["name"])

# 异常处理示例
print("\n=== 异常处理示例 ===")


def safe_get_student(index):
    try:
        return students[index]
    except IndexError:
        print("错误：索引越界")
        return None


result = safe_get_student(0)
print("找到学生：")
print(result["name"])

result = safe_get_student(10)
print("索引 10 越界")

# 递归函数 - 计算阶乘
print("\n=== 递归函数 - 阶乘 ===")


def factorial(n):
    if n <= 1:
        return 1
    else:
        return n * factorial(n - 1)


print("5! =")
print(factorial(5))

# while 循环应用
print("\n=== while 循环统计 ===")
i = 0
count = 0
while i < len(students):
    if students[i]["score"] >= 80:
        count = count + 1
    i = i + 1

print("80分以上学生数：")
print(count)

# try-finally 示例
print("\n=== 处理数据（带清理） ===")
processed = 0
try:
    for student in students:
        score = student["score"]
        processed = processed + 1
finally:
    print("已处理学生数：")
    print(processed)

# 嵌套循环示例
print("\n=== 打印九九乘法表（部分） ===")
for i in range(1, 4):
    for j in range(1, 4):
        result = i * j
        print(i)
        print(j)
        print(result)

# 列表操作
print("\n=== 列表操作 ===")
scores = []
for student in students:
    scores.append(student["score"])

print("所有成绩：")
print(scores)

# 字典操作
print("\n=== 构建姓名到成绩的映射 ===")
name_to_score = {}
for student in students:
    name = student["name"]
    score = student["score"]
    name_to_score[name] = score

print(name_to_score)
print("Alice 的成绩：")
print(name_to_score["Alice"])

print("\n=== 综合示例完成！ ===")
