# take list like
# a 1
# b 2
# c 3
# <empty line>
# and transforms it to a,b,c\n1,2,3

import sys

try:
    k = []
    v = []
    for l in sys.stdin:
        l = l.strip()
        if len(l) == 0:
            print(",".join(k))
            print(",".join(v))
            print("")
            sys.stdout.flush()
            k = []
            v = []
        p = l.split(":")
        if len(p) == 2 and p[1] != "0":
            k.append(p[0])
            v.append(p[1])
    if len(k) > 0:
        print(",".join(k))
        print(",".join(v))
        sys.stdout.flush()
except KeyboardInterrupt:
   sys.stdout.flush()
   pass