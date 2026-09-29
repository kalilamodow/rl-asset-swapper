# upk swapper

A standalone app, based on [rlbuddy](https://github.com/kalilamodow/rlbuddy)'s upk swapping backend, which only does asset swapping. It's meant as a standalone alternative if you don't want all the other extra cool and awesome features of rlbuddy.

### development

It's made with egui/eframe (same as actual rlbuddy), based on the eframe template so that it can build to webassembly (using trunk).

local native: `cargo run`
local native deploy: `cargo build --release`

local web: `trunk serve`
local web deploy: `trunk build --release`

(get trunk with `cargo install --locked trunk`)
