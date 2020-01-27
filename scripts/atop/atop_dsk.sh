atopsar -d -S -b $1 | grep -E "^[0-9][0-9]:" |  grep -v _dsk_ | awk '
BEGIN { 
  current_time="";
  f=0;
  r=0;
  w=0;
} 
{ 
  if ($1 != current_time) { 
    if (current_time != "") {
      if (f == 0) {
        f = 1;
        printf "time,disk read(KB/s),disk write(KB/s)\n";
      }
      printf current_time "," r "," w "\n";
      r = 0;
      w = 0;
    }
  } 
  current_time = $1;
  r = r + $5;
  w = w + $7;
} 
END { 
    if (current_time != "") {
      if (f == 0) {
        printf "time,disk read(KB/s),disk write(KB/s)\n";
      }
      printf current_time "," r "," w "\n";
    }
}'
