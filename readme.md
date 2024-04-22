# Whisper

A VST plugin written in Rust that provides 10 different kinds of noise.

A continuation of [this tutorial](https://rust.audio/articles/vst-tutorial/), using the [noise](https://docs.rs/noise/0.6.0/noise/) crate.



## Building for mac

```sh
cargo build --release
sh osx_vst_bundler.sh Whisper target/release/libwhisper.dylib 
 ```


![](./demo-bitwig.gif)
![](./demo-ableton.gif)
