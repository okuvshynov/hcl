#!/usr/sbin/dtrace -s

#pragma D option quiet

dtrace:::BEGIN
{
/* 
 * printf("time,0,1,2,4,8,16,32,64,128,256,512,1k,2k,4k,8k,16k,32k,64k,128k,256k,512k,1m\n");
 */
}

io:::start
{
  this->size = args[0]->b_bcount;
  @sizes[execname] = quantize(this->size);
}

profile:::tick-1sec
{
  printa("^%S%@d", @sizes);
  printf("\n");
}
