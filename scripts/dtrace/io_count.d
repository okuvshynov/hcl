#!/usr/sbin/dtrace -s

/* 
 * This script counts I/O by executable name,
 * and prints out it every 5 seconds.
 */

#pragma D option quiet

io:::start
{
  @io[execname] = count()
}

profile:::tick-5sec
{
  printa("%S:%@d\n", @io);
  printf("\n");
  clear(@io);
}
