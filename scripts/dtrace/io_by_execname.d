#!/usr/sbin/dtrace -s

#pragma D option quiet

io:::start
{
  @io[execname] = count()
}

profile:::tick-1sec
{
  printa("%S:%@d\n", @io);
  printf("\n");
  clear(@io);
}
