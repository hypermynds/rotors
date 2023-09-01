# Rotors

[![Latest Version](https://img.shields.io/crates/v/rotors.svg)](https://crates.io/crates/rotors)
[![Latest Version](https://docs.rs/rotors/badge.svg)](https://docs.rs/rotors)
![Apache 2.0 OR MIT licensed](https://img.shields.io/badge/license-Apache2.0%2FMIT-blue.svg)


`Proto`cols in Rust without the Pain

## Why should I use this library?

You probably shouldn't, because this library is under development and
has not gone through serious scrutiny yet.

However, if you would like an easier way to use native Rust types inside a
`tonic` project, this might be useful to you!

## How should I use this library?

Instead of writing a gRPC `xyz.proto` file and using the `tonic-build` tools in your `build.rs`,
you can just write a simple macro and let it do most of the heavy lifting.

```rust
use rotors::rotors;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HelloReply {
    pub message: String,
}

rotors! {
    package helloworld;

    service Greeter {
        rpc SayHello (super::HelloRequest) returns (super::HelloReply);
    }
}
```

The `rotors!()` macro will define two new rust modules, containing the tonic client
and server structures. From there onwards, everything will be just like
the tonic you are used to, except that you are finally free to write your
own `enum`s without the tedious boilerplate required by gRPC.


## When should I use this library?

When using `rotors` you are creating some RPC code that is now *NOT COMPATIBLE*
with gRPC. If this is fine for you, then go for it!

On the other hand, if you are writing a big project, or want to communicate between
instances running different versions of your software, or even sofware written
in different languages, this crate won't be of much use to you.