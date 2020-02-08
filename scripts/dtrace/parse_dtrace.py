import re
import sys
import time

current_key = ""
data = {}
buckets = set()

pattern = re.compile(r"^\s+(\d+)\s\|@+\s+(\d+)")

try:
    for l in iter(sys.stdin.readline, b''):
        l = l.rstrip()
        if l == "":
            if len(data) > 0:
                t = time.strftime("%Y-%m-%d %H:%M:%S")
                # flush
                keys = sorted(data.keys())
                print("time,bucket," + ",".join(keys))
                for bucket in sorted(buckets):
                    print(t + "," + str(bucket) + "," + ",".join([str(data[k].get(bucket, 0)) for k in keys]))
                print("")
                sys.stdout.flush()
                data = {}
            continue

        if l.startswith("^"):
            current_key = l[1:]
            data[current_key] = {}
            continue
        
        m = pattern.match(l)
        if m == None:
            continue

        bucket = int(m.group(1))
        buckets.add(bucket)
        value = int(m.group(2))
        data[current_key][bucket] = value
except KeyboardInterrupt:
   sys.stdout.flush()