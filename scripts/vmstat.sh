vmstat -n 1 | awk -W interactive 'NR > 1 {print $9","$10","$11","$12","$13","$14 }'
