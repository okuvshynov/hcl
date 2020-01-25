# $1 -- x (time)
# $2 -- key (series) to implode by
# $3 -- value
BEGIN {
  t = "";
  f = 0;
  h = 0;
  printf "time"
}
{
  if (t != $1) {
    if (t != "") {
      if (f == 0) {
        f = 1;
        print "";
      }

      printf t;
      for (i = 0; i < h; i++) {
        printf "," c[hdr[i]];
      }
      print "";
    }
    t = $1;
  }
  c[$2] = $3;
  if (f == 0) {
    hdr[h] = $2;
    printf("," $2);
    h++;
  }
}
END {
  if (t != "") {
    printf t;
    for (i = 0; i < h; i++) {
      printf "," c[hdr[i]];
    }
    print "";
  }
}
