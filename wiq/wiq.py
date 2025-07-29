#!/usr/bin/env python3

import sys
from functools import cmp_to_key
from dataclasses import dataclass
import subprocess

RESET = "\x1b[0m"
BOLD = "\x1b[1m"
GREEN = "\x1b[32m"
YELLOW = "\x1b[33m"
BLUE = "\x1b[34m"
CYAN = "\x1b[36m"


@dataclass
class User:
    running: int
    pending: int
    partitions: set[str]


queue: dict[str, User] = {}


def parse_job_id(jobid: str) -> int:
    index = jobid.find("[")
    if index < 0:
        return 1
    jobid = jobid[index + 1 : -1]
    njob = 0
    for block in jobid.split(","):
        if "-" not in block:
            njob += 1
            continue
        start, end = block.split("-")
        for i, c in enumerate(end):
            if not c.isdigit():
                end = end[:i]
                break
        njob += int(end) - int(start) + 1
    return njob


def compare_users(name1: str, name2: str) -> int:
    user1 = queue[name1]
    user2 = queue[name2]
    total1 = user1.running + user1.pending
    total2 = user2.running + user2.pending
    return total2 - total1


def main() -> None:
    cmd = ["squeue", "--noheader", "-o %.20u %t %P %i"]
    msg_end = "the queue"
    if len(sys.argv) == 2:
        part = sys.argv[1]
        cmd.append(f"--partition={part}")
        msg_end = f"partition {part}"
    result = subprocess.run(cmd, stdout=subprocess.PIPE, text=True)
    n_total = 0
    for line in result.stdout.splitlines():
        line = line.strip()
        name, status, partition, jobid = line.split()
        if name not in queue:
            queue[name] = User(0, 0, set())
        user = queue[name]
        for p in partition.split(","):
            user.partitions.add(p.strip())
        if status == "R":
            user.running += 1
            n_total += 1
        else:
            n_pending = parse_job_id(jobid)
            user.pending += n_pending
            n_total += n_pending
    if n_total == 0:
        print(f"🥳🎉 There are no jobs in {msg_end} 🎉🥳")
        return
    names = list(queue.keys())
    names.sort(key=cmp_to_key(compare_users))
    print(f"There are {BOLD}{n_total}{RESET} jobs in {msg_end}:")
    for name in names:
        user = queue[name]
        parts = ", ".join(sorted(user.partitions))
        print(f"-> {BLUE}{name:<12s}{RESET}: ", end="")
        print(f"{GREEN}{BOLD}{user.running:>5}{RESET} running, ", end="")
        print(f"{BOLD}{YELLOW}{user.pending:>5}{RESET} pending  ", end="")
        print(f"({CYAN}{parts}{RESET})")


if __name__ == "__main__":
    main()
