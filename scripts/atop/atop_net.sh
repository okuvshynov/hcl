atopsar -i -S -b $1 | grep -E "^[0-9][0-9]:" |  grep -v maxmbps_if_ | awk '
BEGIN { 
  current_time="";
  f=0;
  r=0;
  s=0;
} 
{ 
  if ($1 != current_time) { 
    if (current_time != "") {
      if (f == 0) {
        f = 1;
        printf "time,network recvd(KB/s),network sent(KB/s)\n";
      }
      printf current_time "," r "," s "\n";
      r = 0;
      s = 0;
    }
  } 
  current_time = $1;
  r = r + $6;
  s = s + $7;
} 
END { 
    if (current_time != "") {
      if (f == 0) {
        printf "time,network recvd(KB/s),network sent(KB/s)\n";
      }
      printf current_time "," r "," s "\n";
    }
}'
