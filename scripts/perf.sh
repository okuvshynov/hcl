sudo ~/bin/perf stat -a -A -x ',' -e cycles,instructions,branch-misses --log-fd 1 -I 1000 | awk -F',' -W interactive '{printf("%.0d ", $1); print $2":"$5 $3}' | awk -W interactive -f ./scripts/implode.awk
