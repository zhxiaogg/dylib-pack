# dylib-pack

A MacOS/iOS app dev tool that collects `*.dylib` deps(recursively) and pack them into installer.

## caution
Use it for caution.

## usage

Given a xcode MacOS app build artifact `MyDemoApp.app`. It may relies on some `.dylib`s which some of libs themself may also relies on another set of `.dylib`s.

Say we want collect all the `.dylib`s transcendently into the result `MyDemoApp.app`, eg. `/tmp/MyDemoApp.app/Contents/libs`. The following command will help us do it:

```shell
dylib-pack /tmp/MyDemoApp.app/Contents/MacOS/MyDemoApp /tmp/MyDemoApp.app/Contents/libs @executable_path/../libs/
```

## build

since dylib-pack is written using rust.

```shell
cargo build --release
```
