sudo ~/bin/perf stat -a -A -x ',' -e cycles,instructions --log-fd 1 -I 1000 | awk -F',' -W interactive '{ print $2" "$5" "$3}' | awk -W interactive '
BEGIN {
  printf "time";
  c = "getconf _NPROCESSORS_ONLN";
  c | getline cpus;
  close(c);
  for (i = 0; i < cpus; i++) {
    printf(",cpu#" i ":ipc");
  }
  print "";
  l = 0;
}
{
  counter[$1, $2] = $3;
  l += 1;


  if (l == cpus * 2) {
    c = "date +%H:%M:%S";
    c | getline t;
    printf t;
    close(c);
    for (i = 0; i < cpus; i++) {
      inst = counter["CPU" i, "instructions"];
      cycles = counter["CPU" i, "cycles"];
      if (cycles == 0) {
        cycles = 1;
      }
      printf("," counter["CPU" i, "instructions"] / counter["CPU" i, "cycles"]);
    }
    print "";
    l = 0;
  }
}
'
