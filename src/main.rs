mod app;
mod data;
mod ui;

use crate::data::scale_config::ScalesConfig;
use app::settings::{Column, Settings, SortingMode};
use clap::{App, AppSettings, Arg, ArgGroup};

fn main() -> Result<(), failure::Error> {
    let matches: clap::ArgMatches = App::new("hcl")
        .version("0.1")
        .setting(AppSettings::TrailingVarArg)
        .author("Oleksandr K. <okuvshynov@gmail.com>")
        .arg(
            Arg::with_name("p")
                .short("p")
                // TODO: better help message
                .help("use by pair format instead of csv"),
        )
        .arg(
            Arg::with_name("t")
                .short("t")
                .help("sort by titles (numerically). Useful for distribution plotting."),
        )
        .arg(
            Arg::with_name("x")
                .short("x")
                .help("name of the series to use for X axis values.")
                .takes_value(true),
        )
        .group(ArgGroup::with_name("xg").args(&["x", "i"]).required(false))
        .arg(
            Arg::with_name("scales")
                .short("s")
                .help(
                    "Scale information, global and per series.
Scale maps input domain to [-1;0] and [0;1] intervals.
Format for scale config is global_scale,pattern:scale,...;
Picking a scale for series works in the following sequence:
- if title of the series matches one of the patterns, the first match is used
- otherwise, if global scale is provided, global scale is used.
- otherwise, autoscale is built. 
Input domain for the scale is repesented by two intervals: [a;b] and [b;c].
[a;b] is linearly mapped to [-1;0], extrapolated if needed.
[b;c] is linearly mapped to [0;1], extrapolated if needed.
The format for the scale is the following: [a..[b..]]c.
Each number can have K/M/G/T prefix to allow 10^3, 10^6, 10^9, 10^12 scaling.
Examples:
    * 10k          -- [0;10000] -> [0;1]
    * -1k..1k      -- [-1000;1000] -> [0;1]
    * -1M..0..1M   -- [-1000000;0] -> [-1; 0], [0; 1000000] -> [0; 1]
    * 95..100..105 -- [95;100] -> [-1; 0], [100; 105] -> [0; 1]

Complete example:
cpu:100,ram:16G,wow_change:95..100..105 will break down into 3 scales.
Series with titles like 'cpu#1' will match first scale, thus, using [0;100] -> [0;1];
Series like 'ram_total', 'ram_free', 'ram_cache' will match second.
Series 'revenue_wow_change' will use [95;100;105] -> [-1;0;1] scale.

It's possible to have autoscale groups configured. This could be very useful when several
series have same semantics and should share the scale. For example, in
'cycles:auto,instructions:auto,ram:32G', all series with titles matching 'cycles' 
will compute one shared scale. Another group of series matching 'instructions' will have 
another single shared autoscale.
    ",
                )
                .validator(|s| {
                    ScalesConfig::new(&s)
                        .map(|_| ())
                        .map_err(|e| format!("{}", e))
                })
                .takes_value(true),
        )
        .arg(Arg::with_name("input_file"))
        .get_matches();

    let input_file = matches
        .values_of("input_file")
        .map(|o| o.map(ToOwned::to_owned).collect());

    let settings = Settings {
        input_file,
        scales: matches.value_of("scales").map(ToOwned::to_owned),
        x: match matches.value_of("x") {
            Some(title) => Column::Title(title.to_owned()),
            _ => Column::None,
        },
        paired: matches.is_present("p"),
        sort_mode: if matches.is_present("t") {
            SortingMode::TitlesNumericAsc
        } else {
            SortingMode::ValuesDesc
        },
    };

    app::event_loop::EventLoop::start(settings)
}
