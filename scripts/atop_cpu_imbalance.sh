# produces total CPU load + deltas (imbalance between logical CPUs)
last_h=`date  "+%R" -d "15 min ago"`
atopsar -c -S -b $last_h | grep -E "^[0-9][0-9]:" |  grep -v all | grep -v cpu | awk -f ./scripts/implode.awk | awk -F',' '
BEGIN {
  f = 0;
}
{
  if (NF > 1) {
    total_cpu = 0;
    for (i = 2; i <= NF; i++) {
      total_cpu += $i;
    }
    if (f == 0) {
      printf("time,cpu_avg");
      for (i = 2; i <= NF; i++) {
        printf(",cpu#" (i - 2))
      }
      print "";
      f = 1;
    }
  
    avg_cpu = total_cpu / (NF - 1);
    printf($1","avg_cpu);
    for (i = 2; i <= NF; i++) {
      printf("," $i - avg_cpu)
    }
    print "";

  }
}
'
