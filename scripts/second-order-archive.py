#!/usr/bin/env python3
"""Move closed rows (status done or decided) from SECOND-ORDER.md to
notes/second-order-archive.md, so agents read only live rows. Run it when
rows close: at each merge and progress report. `--check` only counts them."""
import sys

LIVE, ARCHIVE = "SECOND-ORDER.md", "notes/second-order-archive.md"


def closed(line):
    cells = [c.strip() for c in line.split("|")[1:-1]]
    status = cells[5].lower() if len(cells) > 5 else ""
    return status.startswith(("done", "decided"))


lines = open(LIVE).read().split("\n")
rule = next(i for i, l in enumerate(lines) if l.startswith("| ---"))
keep = lines[: rule + 1]
moved = []
for line in lines[rule + 1 :]:
    (moved if line.startswith("| ") and closed(line) else keep).append(line)
if "--check" in sys.argv:
    print(f"{len(moved)} closed rows to archive")
    sys.exit(0)
if moved:
    open(LIVE, "w").write("\n".join(keep))
    with open(ARCHIVE, "a") as f:
        f.write("\n".join(moved) + "\n")
print(f"archived {len(moved)} rows")
