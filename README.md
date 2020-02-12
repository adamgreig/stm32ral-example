# stm32ral-example

This crate is an example embedded project using
[stm32ral](https://github.com/adamgreig/stm32ral).

It targets an STM32F405 microcontroller but is readily adapted to other
STM32 devices.

## Building It Yourself

You should be able to:

```
$ git clone https://github.com/adamgreig/stm32ral-example
$ cd stm32ral-example
$ cargo build --release
```

If that didn't work, you might need to set up your Rust environment:

```
$ rustup target add thumbv7em-none-eabihf
```
