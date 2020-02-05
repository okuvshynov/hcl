mod app;
mod data;
mod platform;
mod ui;

use crate::data::scale_config::ScalesConfig;
use app::settings::{Column, Settings};
use clap::{App, AppSettings, Arg, ArgGroup};

fn main() -> Result<(), failure::Error> {
    let matches: clap::ArgMatches = App::new("hcl")
        .version("0.1")
        .setting(AppSettings::TrailingVarArg)
        .author("Oleksandr K. <okuvshynov@gmail.com>")
        .arg(
            Arg::with_name("t")
                .short("t")
                .help("name of the field to use for epoch tracking.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("x")
                .short("x")
                .help("name of the field to use for X axis values.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("i")
                .short("i")
                .help("index of the field to use for X axis values.")
                .validator(|v| {
                    v.parse::<u64>()
                        .map(|_| ())
                        .map_err(|_| "unable to parse".to_owned())
                })
                .takes_value(true),
        )
        .group(ArgGroup::with_name("xg").args(&["x", "i"]).required(false))
        .arg(
            Arg::with_name("r")
                .short("r")
                .help("refresh rate, milliseconds. 0 for no refresh.")
                .validator(|v| {
                    v.parse::<u64>()
                        .map(|_| ())
                        .map_err(|_| "unable to parse".to_owned())
                })
                .default_value("0"),
        )
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
        .arg(Arg::with_name("command").multiple(true))
        .get_matches();

    let r = matches.value_of("r").unwrap().parse::<u64>().unwrap();

    let x = (matches.value_of("x"), matches.value_of("i"));

    let cmd = matches
        .values_of("command")
        .map(|o| o.map(ToOwned::to_owned).collect());

    let settings = Settings {
        cmd,
        refresh_rate: std::time::Duration::from_millis(r),
        scales: matches.value_of("scales").map(ToOwned::to_owned),
        x: match x {
            (Some(title), None) => Column::Title(title.to_owned()),
            (None, Some(index)) => Column::Index(index.parse::<usize>().unwrap()),
            _ => Column::None,
        },
        epoch: match matches.value_of("t") {
            Some(title) => Column::Title(title.to_owned()),
            _ => Column::None,
        },
    };

    app::event_loop::EventLoop::start(settings)
}
