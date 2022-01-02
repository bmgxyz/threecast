# threecast

*Like a forecast, but smaller*

`threecast` is a [precipitation nowcaster][nowcaster]. It's a program that
predicts how much precipitation will fall at a given location over the coming
hour or so.

This is a much smaller scale than typical forecasts, which usually cover a large
area and a period of a few days. `threecast` doesn't try to tell you whether
it'll rain in your city tomorrow, but it can try to predict whether it'll rain
at your house in the next twenty minutes. Both scales are useful.

`threecast` is a free software alternative to existing precipitation nowcasters,
such as [DarkSky][darksky] and [RainViewer][rainviewer].

[nowcaster]: https://en.wikipedia.org/wiki/Nowcasting_(meteorology)
[darksky]: https://darksky.net/
[rainviewer]: https://www.rainviewer.com/

## Usage

`threecast` is a library that exposes a useful API for precipitation nowcasting.
`threecast-cli` is a tool for running predictions for a given location on the
command line. `threecast-data-tool` (abbreviated `tcdt`) is a helper program
that allows you to download data for arbitrary use.

To build and run `threecast-cli`, follow these steps:

1) Install [Rust][rust].
2) Clone this repo and run `cargo build -p threecast-cli --release`.
3) Run a prediction like this: `./target/release/threecast-cli --longitude -69.068597 --latitude 44.387473`.

See the help text for more information. (Run `threecast-cli --help`.)

[rust]: https://rustup.rs/

## Limitations

`threecast` only supports locations that are within the coverage area of a
National Weather Service (NWS) radar station. This includes nearly all of the
populated areas in the United States. I don't plan to support other parts of the
world, but in principle it should be possible to do that. The only criteria are
that the desired area must have some kind of high-resolution precipitation radar
system and that the data files can be ingested and parsed in a reasonable
fashion. After all the upstream stuff is handled, the downstream logic
(translating a location to a radar station and doing the predictions) would
probably be mostly unchanged.

`threecast` uses the simplest possible optical flow algorithm I could think of
for prediction. I call it DumbFlow. DumbFlow assumes that there is some
translation vector between two consecutive precipitation scans. To find it,
DumbFlow simply tries every possible translation vector with integer components
and chooses the one with the smallest pixel-wise mean squared error. Then, it
"runs time forward" by translating the second input image by some multiple of
the translation vector according to the desired prediction time. DumbFlow is
dumb because it completely ignores the fact that storms change shape, but it's
fast and not completely useless. Someday I hope to implement a testing
subcommand for `tcdt` that computes some metrics for different algorithms,
including DumbFlow.

## License

Copyright 2021-2022 Bradley Gannon

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
