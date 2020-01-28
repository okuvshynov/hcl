atopsar -c -S -b $1 | grep -E "^[0-9][0-9]:" |  grep -v all | grep -v cpu | awk -f ./scripts/implode.awk
