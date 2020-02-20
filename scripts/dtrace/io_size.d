#!/usr/sbin/dtrace -s

/* 
 * This script is inspired by bitesize.d 
 * (https://github.com/brendangregg/DTrace-book-scripts/blob/master/Chap4/bitesize.d),
 * but written manually, rather than using aggregations;
 */

#pragma D option quiet

dtrace:::BEGIN
{
  printf("time,0,1,2,4,8,16,32,64,128,256,512,1k,2k,4k,8k,16k,32k,64k,128k,256k,512k,1m\n");
}

io:::start
{
  this->size = args[0]->b_bcount;
  this->size_original = this->size;
  /* 
   * Manually computing next power of 2.
   * https://graphics.stanford.edu/~seander/bithacks.html#RoundUpPowerOf2
   */
  this->size--;
  this->size |= this->size >> 1;
  this->size |= this->size >> 2;
  this->size |= this->size >> 4;
  this->size |= this->size >> 8;
  this->size |= this->size >> 16;
  this->size++;
  this->size = this->size_original >= (1 << 20) ? (1 << 21) : this->size;
  sizes[this->size]++;
}

profile:::tick-1sec
{
  printf("%Y,", walltimestamp);
  printf("%d,", sizes[0]);
  printf("%d,", sizes[1 << 1]);
  printf("%d,", sizes[1 << 2]);
  printf("%d,", sizes[1 << 3]);
  printf("%d,", sizes[1 << 4]);
  printf("%d,", sizes[1 << 5]);
  printf("%d,", sizes[1 << 6]);
  printf("%d,", sizes[1 << 7]);
  printf("%d,", sizes[1 << 8]);
  printf("%d,", sizes[1 << 9]);
  printf("%d,", sizes[1 << 10]);
  printf("%d,", sizes[1 << 11]);
  printf("%d,", sizes[1 << 12]);
  printf("%d,", sizes[1 << 13]);
  printf("%d,", sizes[1 << 14]);
  printf("%d,", sizes[1 << 15]);
  printf("%d,", sizes[1 << 16]);
  printf("%d,", sizes[1 << 17]);
  printf("%d,", sizes[1 << 18]);
  printf("%d,", sizes[1 << 19]);
  printf("%d,", sizes[1 << 20]);
  printf("%d\n", sizes[1 << 21]);

  sizes[0]=0;
  sizes[1 << 1]=0;
  sizes[1 << 2]=0;
  sizes[1 << 3]=0;
  sizes[1 << 4]=0;
  sizes[1 << 5]=0;
  sizes[1 << 6]=0;
  sizes[1 << 7]=0;
  sizes[1 << 8]=0;
  sizes[1 << 9]=0;
  sizes[1 << 10]=0;
  sizes[1 << 11]=0;
  sizes[1 << 12]=0;
  sizes[1 << 13]=0;
  sizes[1 << 14]=0;
  sizes[1 << 15]=0;
  sizes[1 << 16]=0;
  sizes[1 << 17]=0;
  sizes[1 << 18]=0;
  sizes[1 << 19]=0;
  sizes[1 << 20]=0;
  sizes[1 << 21]=0;
}
