#!/usr/bin/env python3
import sys

with open(r'e:\Workspace\project\Rust\mir2-rust1\ui\app_window.slint', 'r', encoding='utf-8') as f:
    lines = f.readlines()

stack = []
for i, line in enumerate(lines, 1):
    for j, char in enumerate(line):
        if char == '{':
            stack.append(('open', i, j+1))
        elif char == '}':
            if stack:
                last = stack.pop()
            else:
                print(f'ERROR: Extra closing bracket at line {i}, column {j+1}')

if stack:
    print(f'ERROR: {len(stack)} unclosed brackets:')
    for item in stack[-5:]:
        print(f'  Opening at line {item[1]}, column {item[2]}')
elif len(stack) < 0:
    print(f'ERROR: Extra closing brackets')
else:
    print('OK: All brackets matched')
