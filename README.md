# HCL

Horizon charts provide a nice balance between information density and clarity in the case of multiple time series.
Interactive examples of horizon charts can be found on [observable](https://observablehq.com/@d3/horizon-chart-ii) and, specifically for system monitoring, in [Square's cubism](https://square.github.io/cubism/). 

hcl (Horizon CLi) is a command-line tool for visualizing arbitrary data in horizon format.
You can think of it as a chart version of a ```less +F``` for numeric data.

## Installation

### Build from source

```
$ git clone https://github.com/okuvshynov/hcl.git
$ cd hcl
$ cargo build --release
$ cat tests/sine.csv | ./target/release/hcl  # test on static data
```

### Binaries

Coming soon; Once CI is set up, they'll be here;

## Usage

### Data fetching

hcl reads data in two formats:
1) [default] CSV-like, with comma-separated values. Each column in the file represents individual series, and each row in the file becomes a 'column' in the chart: [example](tests/sine.csv)
2) title:value pairs; Empty line represents a separator between different 'columns' in the chart: [example](tests/rt_two_col.sh)

Data is provided to hcl either by piping it to stdin, or by providing file name as a last argument.


```
$ cat tests/sine.csv | hcl 
```
Equivalent to
```
$ hcl tests/sine.csv
```

If empty line is encountered, new set of titles is expected. hcl will attempt to 'merge' the data after that, filling missing values with NaN.
This can be useful in the cases like 'show top N processes reading from HDD, with 1 second granularity', when set of 'top N' might be different every time.
 Check this simple [example](tests/rt_new_series.sh) which simulates that.

There's an option to use one of the columns as an 'x' axis. Most commonly that would be some form of
time/date, but it's not required - it can be an arbitrary string. X is configured using -x <column_title> option.

All other columns need to have floating-point numbers as their values. Missing values or the ones failed to parse will be represented as '.' (NaN).

There's no filtering/aggregation functionality. If there's a need to do so, external tool (awk, xsv, sed, ...) should be used before piping the input to hcl.

### Scales
The scale is a piecewise linear map from an input domain to [-1;0] and [0;1] intervals. [-1;1] values will be mapped to Unicode block characters, full dark green representing '1' and full dark red representing '-1'.

Scales are defined in a comma-separated list, for example
```
0..10,cpu:100,ram:32G,cycles:auto
```
There're four scales here, one global [0..10], one with 'cpu' pattern, another with 'ram' pattern, and an autoscale for 'cycles' pattern.
       
Input domain for the scale is repesented by two intervals: [a;b] and [b;c].
[a;b] is mapped to [-1;0] and [b;c] is mapped to [0;1].
The format for the scale is '[a..[b..]]c' OR 'auto'.
Each number can have K/M/G/T suffix to allow 10^3, 10^6, 10^9, 10^12 scaling.
Examples:
* 10k          -- [0;10000] -> [0;1]
* -1k..1k      -- [-1000;1000] -> [0;1]
* -1M..0..1M   -- [-1000000;0] -> [-1; 0], [0; 1000000] -> [0; 1]
* 95..100..105 -- [95;100] -> [-1; 0], [100; 105] -> [0; 1]

Picking a scale for the series follow such procedure:
* if the title of the series matches one of the patterns, the first match is used;
* otherwise, if the global scale is provided, the global scale is used;
* otherwise, autoscale is built. 

Complete example:
'cpu:100,ram:16G,wow_change:95..100..105' will break down into 3 scales.
* Series with title 'cpu#1' will match first scale, thus, using [0;100] -> [0;1];
* Series 'ram_total', 'ram_free', 'ram_cache' will match second scale;
* Series 'revenue_wow_change' will use [95;100;105] -> [-1;0;1] scale;
* series 'errors' won't match anything, and as there's no default scale provided, autoscale will be used.

### Autoscaling

Autoscaling is an attempt to provide reasonable default behavior. Autoscaling works on individual series level by default; '-s auto' would force global autoscale.
After iterating over all values in a series, following logic is applied:
* if there's no data at all, return identity map [0;1] -> [0;1];
* if there're only zeroes in the data, return identity map;
* if both positive and negative values are present, find min and max values, and use piecewise mapping [min; 0; max] -> [-1; 0; 1];
* if only positive numbers present, use [0; max] -> [0; 1] map;
* if only negative numbers present, use [min; 0] -> [-1; 0] map.

In cases when 'auto' is used explicitly, for example: 'cpu:auto,ram:12G', all series matching 'cpu' would use same scale, computed based on combined data.

For example, we have following input data (load.csv):

```
cpu#1,cpu#2,ram_free_gb
50,20,1
100,20,2,
50,20,3
```

If we run ```cat load.csv | hcl```, each series will compute autoscale individually, so:
  * cpu#1 -> [50,100,50], [0;100] -> [0; 1] will be used, with values transformed to [0.5,1.0,0.5];
  * cpu#2 -> [20,20,20], [0;20] -> [0; 1] will be used. Values would be [1.0,1.0,1.0] Second cpu will look more 'loaded'.
  * ram_gb -> [1, 2, 3] -> [0; 3] will be used.

If, instead, we do ```cat load.csv | hcl -s cpu:auto```, both cpu series will be used together, and [0;100] will be used for both.
In case of cpu load, which often is represented as percentage, it might be easier to just specify 'cpu:100'.
For counters which might have no obviously good upper bound, auto could be benefitial. 
For example:
 * distribution of wait time in scheduler queue;
 * number of context switches;
 * CPU PMU counters: cycles, instructions, cache-misses, etc. 

Autoscale is recomputed on every data update, as new values might change the interval boundaries;
Autoscaling has its drawbacks, specifically old data might change its appearance as new data arrives, which could look confusing; 
Thus, it's often a better option to provide scale as an input. '-s core:100' will be sufficient for the example above.

'-s auto' is a way to treat all series together, and pick a single scale for all of them.

### Series ordering
In progress;

### Command-line arguments

USAGE:
* hcl [OPTIONS] [input_file]

FLAGS:
* -h, --help       Prints help information;
* -V, --version    Prints version information.
* -p               Use key:value pair format instead of CSV
* -t               sort by titles (numerically). Useful for distribution plotting.

OPTIONS:
* -s <scales>        scale information, global and per series, according to scale format above;
* -x <x>             name of the field to use for X axis values. Only one of -i/-x can be used.

ARGS:
    <input_file>

### Interactive commands

Interactive commands resemble those of applications like 'less'.

Vertical navigation:
* f - forward (page down)
* b - backward (page up)
* u - half-page forward
* d - half-page backward
* arrow down or j - down one series
* arrow up or k - up one series
* g - top
* G - bottom

Horizontal navigation:
* arrow right or l - move the cursor right
* L - move the cursor to the right-most visible position
* arrow left or h - move the cursor left
* H - move the cursor to the left-most visible position
* Ctrl+l - shift view window to the right
* Ctrl+h - shift view window to the left
* $ - shift view window to latest available data;
* 0 - shift view window to the earliest available data;

Mouse:
* Wheel scrolls series up and down, if not all series fit on the screen.
* Left button click updates cursor location.

Other:
* p -- pause/resume auto-scroll to new data. Pausing can be useful when inspecting older data, to avoid refresh.
* c -- show/hide cursor

## Examples

### static CSV file

```
cat tests/sine.csv | hcl 
```

![static demo](https://github.com/okuvshynov/hcl/raw/master/static/sine.png "static demo")

### vmstat
[vmstat](https://linux.die.net/man/8/vmstat) is a convenient tool to monitor current CPU/Memory/IO on Linux/BSD. 

```
$ vmstat -n 1 | awk -W interactive -v OFS=',' '{if (NR>1) { $1=$1; print; }}' | hcl
```

![vmstat demo](https://github.com/okuvshynov/hcl/raw/master/static/vmstat.png "vmstat demo")

### atop

[atop](https://linux.die.net/man/1/atop) can be very useful to look into historical data on a single host.

cpu usage per core:
```
$ atopsar -c -S -b 21:25 | awk '$1 ~ /[0-9][0-9]:[0-9][0-9]/ && $2 != "cpu" { if ($2 == "all") { print ""} else {print "cpu"$2":"$3+$5}}' | hcl -p -s cpu:100
```

top 3 processes by RAM:
```
$ atopsar -G -S -b 21:35 | awk '! /_top3_/ && $1 ~ /[0-9][0-9]:[0-9][0-9]/ { print "t," $3 "-" $2 "," $7 "-" $6 "," $11 "-" $10 "\n" $1","$4+0","$8+0","$12+0"\n"}' | hcl -x t
```

cpu usage by executable name since 11am:
```
atop -PPRC -r -b 11:00 | awk '{if ($8 != "") { if ($11+$12>0)print $8":"$11+$12; } else {print "";}}' | hcl -p -s auto
```

### dtrace oneliners

hcl can work with [dtrace](http://dtrace.org/blogs/about/) and display dynamic tracing information in realtime. The following example is from running dtrace on MacOS 

syscalls count by syscall
```
$ dtrace -q -n 'syscall:::entry { @n[probefunc] = count(); } profile:::tick-1sec { printa("%S:%@d\n", @n); printf("\n"); clear(@n)}' | hcl -p -s auto
```

io count by executable name
```
$ dtrace -q -n 'io:::start { @n[execname] = count(); } profile:::tick-1sec { printa("%S:%@d\n", @n); printf("\n"); clear(@n)}' | hcl -p -s auto
```

io size by executable name
```
$ dtrace -q -n 'io:::start { @n[execname] = sum(args[0]->b_bcount); } profile:::tick-1sec { printa("%S:%@d\n", @n); printf("\n"); clear(@n)}' | hcl -p -s auto
```

Collect io size distribution for 10 seconds and show the result.
```
$ dtrace -q -n 'io:::start { @ = quantize(args[0]->b_bcount); } profile:::tick-1sec { printa("%@d", @); clear(@)} profile:::tick-10s {exit(0);}' | awk '{if (NF==3) print (0+$1)":"(0+$3); if ($0=="") print "";}' | hcl -s auto -p
```

### bpftrace one-liners

Visualize page faults by process, update every second:
```
$ bpftrace -e 'software:faults:1 { @[comm] = count(); } interval:s:1 { print(@); clear(@)}' | hcl -p -s auto
```

### perf one-liners
For CPU counters, [linux perf](https://perf.wiki.kernel.org/index.php/Main_Page) can be used to print out PMU events.

Show IPC by CPU:

```
$ perf stat -a -A -x ',' -e cycles,instructions --log-fd 1 -I 1000 | awk -v cores=`getconf _NPROCESSORS_ONLN` -F',' -W interactive '$0 ~ /instructions/ {print $2":"$8; if (NR % cores == 0) { print ""; } }' | hcl -p -s 2
```

### other use-cases
In theory, HCL can handle many use-cases, like
* Distributed systems monitoring;
* A/B tests visualization;
* Various forms of BI, notably w/w changes of business metrics.

However, for most such cases there're usually better tools available; hcl is most useful for visualizing the data which is 'right here on the machine'.

## Requirements
* terminal needs to support Unicode characters;
* terminal needs to support 256 color mode;
* font must have block characters available: (' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█');
* [termion](https://github.com/redox-os/termion) is used for UI, thus, Windows is not supported.
