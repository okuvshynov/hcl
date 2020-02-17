#!/usr/sbin/dtrace -s

/*
 * This script counts the syscalls and prints out the aggregate
 * count by syscall.
 */ 
#pragma D option quiet

syscall:::entry
{
  @io[probefunc] = count()
}

profile:::tick-5sec
{
  printa("%S:%@d\n", @io);
  printf("\n");
  clear(@io);
}
