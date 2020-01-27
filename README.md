# HCL

Horizon charts provide a nice balance between information density and clarity in the case of multiple time series.
Interactive examples of horizon charts can be found on [observable](https://observablehq.com/@d3/horizon-chart-ii) and, specifically for system monitoring, in [Square's cubism](https://square.github.io/cubism/). 

hcl is a command-line application that reads arbitrary CSV dataset and presents it as an interactive group of horizon charts right in the terminal. 
You can think of it as a chart version of a ```less +F``` for numeric CSV data.

Obvious use-case of a atop-based load monitor:

![atop demo](https://github.com/okuvshynov/hcl/raw/master/static/atop.gif "atop demo")

## Installation
TODO: to generate binaries for Mac/Linux/BSD

## Usage

### Data fetching

hcl reads data in CSV format, either from stdin or as a result of another command execution.
There're two main modes of operation: incremental fetch and full replacement. 

In case of full replacement, the data is read until EOF, and old data is dropped.
This could be useful when the underlying source might have a lag, and it's cleaner to just get a full copy of it. 
To use the mode, supply -r <frequency_ms> argument:
``` 
$ hcl -r 1000 ./somecsv.sh # read stdout of somecsv.sh, overwrite old data; repeat every 1 second
```

An alternative 'incremental' mode appends whatever is read to each series. It's very natural with a pipe, but can be used with command execution as well - just do not supply -r.
```
$ some.sh | hcl # read line-by-line and append as soon as the new line arrives.
```
Equivalent to
```
$ hcl somecsv.sh
```

There's an option to use one of the columns in CSV as an 'x' axis. Most commonly that would be some form of
time, but it's not required, it can be an arbitrary string. This is configured using -x or -i options.
All other columns need to have floating-point numbers as their values. Missing values or the ones failed to parse will be represented as '!' (NaN).

### Scales
The scale is a piecewise linear map from an input domain to [-1;0] and [0;1] intervals. [-1;1] values will be mapped to colored Unicode block characters, full dark green representing '1' and full dark red representing '-1'.

Scales are defined in a comma-separated list, for example
```
0..10,cpu:100,ram:32G
```
There're three scales here, one global [0..10], one with 'cpu' pattern, another with 'ram' pattern.
       
Input domain for the scale is repesented by two intervals: [a;b] and [b;c].
[a;b] is mapped to [-1;0] and [b;c] is mapped to [0;1].
The format for the scale is [a..[b..]]c.
Each number can have K/M/G/T prefix to allow 10^3, 10^6, 10^9, 10^12 scaling.
Examples:
* 10k          -- [0;10000] -> [0;1]
* -1k..1k      -- [-1000;1000] -> [0;1]
* -1M..0..1M   -- [-1000000;0] -> [-1; 0], [0; 1000000] -> [0; 1]
* 95..100..105 -- [95;100] -> [-1; 0], [100; 105] -> [0; 1]

Picking a scale for the series happens using the following algorithm:
* if the title of the series matches one of the patterns, the first match is used
* otherwise, if the global scale is provided, the global scale is used.
* otherwise, autoscale is built. 

Complete example:
'cpu:100,ram:16G,wow_change:95..100..105' will break down into 3 scales.
* Series with title 'cpu#1' will match first scale, thus, using [0;100] -> [0;1];
* Series 'ram_total', 'ram_free', 'ram_cache' will match second scale.
* Series 'revenue_wow_change' will use [95;100;105] -> [-1;0;1] scale.
* series 'errors' won't match anything, and as there's no default scale provided, autoscale will be used.

### Autoscaling

Autoscaling is an attempt to provide reasonable default behavior. 
Autoscaling works on individual series level. After iterating over all values in a series,
following logic is applied:
* if there's no data at all, return identity map [0;1] -> [0;1]
* if there're only zeroes in the data, return identity map
* if both positive and negative values are present, find min and max values, and use piecewise mapping [min; 0; max] -> [-1; 0; 1]
* if only positive numbers present, use [0; max] -> [0; 1] map
* if only negative numbers present, use [min; 0] -> [-1; 0] map.
 
Autoscale is recomputed on every data update, as new values might change the result;
It has its drawbacks, specifically:
* old data might change its appearance as new data arrives, which could look confusing;
* if multiple series have similar semantics, like CPU load by core, autoscale won't know that; different scales could be used for different cores, thus, making data confusing visually. 
  * core#1 has load [50,100,50], [0;100] -> [0; 1] will be used, with values transformed to [0.5,1.0,0.5]
  * core#2 has load [20,20,20], [0;20] -> [0; 1] will be used. Values would be [1.0,1.0,1.0] Second core will look more 'loaded'.
  
Thus, it's often a better option to provide scale as an input. '-s core:100' will be sufficient for the example above.

### Command-line arguments

USAGE:
* hcl [OPTIONS] [command]...

FLAGS:
* -h, --help       Prints help information
* -V, --version    Prints version information

OPTIONS:
* -i <index>         index of the field to use for X axis values. Only one of -i/-x can be used.
* -r <rate>          refresh rate, milliseconds. 0 for no refresh. [default: 0]
* -s <scales>        Scale information, global and per series, according to scale format above.
* -x <x>             name of the field to use for X axis values. Only one of -i/-x can be used.

ARGS:
    <command>...

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
* p -- pause/resume auto-scroll to new data.

## Examples

### static CSV file

```
cat scripts/sine.csv | hcl 
```

![static demo](https://github.com/okuvshynov/hcl/raw/master/static/sine.png "static demo")

### vmstat
[vmstat](https://linux.die.net/man/8/vmstat) is a convenient tool to monitor current CPU/Memory/IO on Linux/BSD. 

```
$ vmstat -n 1 | awk -W interactive -v OFS=',' '{if (NR>1) { $1=$1; print; }}' | hcl
```

![vmstat demo](https://github.com/okuvshynov/hcl/raw/master/static/vmstat.png "vmstat demo")

### atop

[atop](https://linux.die.net/man/1/atop) can be very useful to look into historical data on a single host. In this example [atopsar](https://linux.die.net/man/1/atopsar) reads log files produced by atop and hcl shows CPU/disk/network utilization over the last 1 hour.

```
$ hcl -r 1000 -x time -s 'cpu:100,network:500' ./scripts/atop/atop.sh
```

Every second, hcl will call the aggregation script. Here we can see some of the configuration options in action:
* -r 1000 tells 'how often to query for new data', in this case, 1000 ms = 1s;
* -x time -- the name of the column to use for 'x' axis. Usually, that will be some form of time/date;
* -s -- defines a scale for the series. If series title matches the filter ('cpu') the values will be scaled as if the domain for the values is [0; 100];
* ./scripts/atop/atop.sh -- represents a script to run to generate the data.

![atop demo](https://github.com/okuvshynov/hcl/raw/master/static/atop.gif "atop demo")


### dtrace

dtrace could be made work with hcl. The following example is from running dtrace on MacOS 

```
sudo scripts/dtrace/bitesize1s.d | hcl -x time -s 50
```

This traces all disk IO events and shows how the distribution of the size of the IO operation is changing over time.
Series names (1k, 2k, ...) mean 'IO of this size in bytes', and each value in the chart represent 'how many IO operations of this size happened during that second'.

![dtrace demo](https://github.com/okuvshynov/hcl/raw/master/static/dtrace.png "dtrace demo")

### eBPF

TODO: record screenshot/video.

### perf
For CPU counters, [linux perf](https://perf.wiki.kernel.org/index.php/Main_Page) can be used to print out PMU events.


```
$ perf stat -a -A -x ',' -e cycles,instructions,branch-misses --log-fd 1 -I 1000
     1.000644593,CPU0, 6160149,,cycles,1004000000,100.00,,
     1.000644593,CPU1, 7411186,,cycles,1004000000,100.00,,
     1.000644593,CPU2, 5117161,,cycles,1004000000,100.00,,
     1.000644593,CPU3, 5025018,,cycles,1004000000,100.00,,
     1.000644593,CPU4, 4781416,,cycles,1004000000,100.00,,
     1.000644593,CPU5, 5511300,,cycles,1004000000,100.00,,
     1.000644593,CPU6, 4865673,,cycles,1004000000,100.00,,
     1.000644593,CPU7, 5189923,,cycles,1004000000,100.00,,
     1.000644593,CPU0, 3482091,,instructions,1004000000,100.00,0.57,insn per cycle
     1.000644593,CPU1, 3978030,,instructions,1004000000,100.00,0.65,insn per cycle
     1.000644593,CPU2, 3039905,,instructions,1004000000,100.00,0.49,insn per cycle
     1.000644593,CPU3, 2959086,,instructions,1004000000,100.00,0.48,insn per cycle
    ...
```

produces a data series with instructions/cycles per each CPU available, writing new data every second.

perf.sh script implodes data into CSV format, which can be visualized with hcl;

```
$ scripts/perf/perf.sh | hcl -x time
```

![perf demo](https://github.com/okuvshynov/hcl/raw/master/static/perf.gif "perf demo")

### other use-cases
HCL can handle many use-cases, like
* Distributed systems monitoring;
* A/B tests visualization;
* BI, notably w/w changes;

For most such use-cases there're usually better tools available though, hcl is most useful for visualizing the data which is 'right here on the machine'.

## OS Support & Requirements
* terminal needs to support Unicode characters;
* terminal needs to support 256 color mode;
* font must have block characters available: (' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█');
* [termion](https://github.com/redox-os/termion) is used for UI, thus, Windows is not supported.
