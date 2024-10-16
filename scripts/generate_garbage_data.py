#!/usr/bin/env python

from random import choice
from string import printable
import os

folder = os.path.dirname(__file__) + "/.."

size_block = 5_000_000
total_size = 100_000_000
num_block = total_size // size_block

print(f"Generating {total_size // 1_000_000}MB of garbage data by block of {size_block // 1_000_000}MB")

for i in range(i, num_block+1):
    a = []
    for _ in range(size_block):
        a.append(choice(printable))

    with open(f"{folder}/tests/garbage_data", "a") as f:
        f.write("".join(a))
    print(f"Block {i:02} done ({i / num_block * 100:6.3f}%)", end="\r")
print(f"Block {i:02} done ({i / num_block * 100:6.3f}%)")
