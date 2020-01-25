last_h=`date  "+%R" -d "15 min ago"`
./scripts/atop_cpu.sh $last_h > /tmp/hcl_atop_cpu
./scripts/atop_dsk.sh $last_h | awk -F',' '{ print $2 "," $3}'  > /tmp/hcl_atop_dsk
./scripts/atop_net.sh $last_h | awk -F',' '{ print $2 "," $3}'  > /tmp/hcl_atop_net
cpu_len=`wc -l /tmp/hcl_atop_cpu | awk '{print $1}'`
dsk_len=`wc -l /tmp/hcl_atop_dsk | awk '{print $1}'`
net_len=`wc -l /tmp/hcl_atop_net | awk '{print $1}'`

min_len=$(($cpu_len < $dsk_len ? $cpu_len : $dsk_len))
min_len=$(($min_len < $net_len ? $min_len : $net_len))

head -n $min_len /tmp/hcl_atop_net > /tmp/hcl_atop_net2
head -n $min_len /tmp/hcl_atop_cpu > /tmp/hcl_atop_cpu2
head -n $min_len /tmp/hcl_atop_dsk > /tmp/hcl_atop_dsk2

paste -d',' /tmp/hcl_atop_cpu2 /tmp/hcl_atop_dsk2 /tmp/hcl_atop_net2
rm /tmp/hcl_atop_net /tmp/hcl_atop_cpu /tmp/hcl_atop_dsk
rm /tmp/hcl_atop_net2 /tmp/hcl_atop_cpu2 /tmp/hcl_atop_dsk2

