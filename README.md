# OpenSnitch TUI

A Terminal UI control plane for [OpenSnitch](https://github.com/evilsocket/opensnitch), an interactive application firewall for Linux inspired by Little Snitch.

![TUI screenshot](static/screenshot.png)

This TUI is built in Rust, namely using the `tokio`, `tonic`, and `ratatui` crates. This program currently implements a subset of functions that the [OpenSnitch GUI](https://github.com/evilsocket/opensnitch/wiki/Getting-started) supports. Some features may never be supported due to complexity (e.g. support for multiple nodes).

## Features

This TUI tries to replace the official OpenSnitch GUI in single-node environments where it may be inconvenient/impossible to use the GUI.

* View high-level daemon runtime stats
* View trapped connection attempts that require a disposition (allow/deny)
* Easy keybindings to allow/deny trapped network flows
* View incoming alerts

The GUI may still be used separately (see below) for features the TUI doesn't yet support.

## Usage

The OpenSnitch daemon connects to a control plane server (like this TUI) to talk gRPC. OpenSnitch's [default config](https://github.com/evilsocket/opensnitch/wiki/Configurations) uses a Unix domain socket for transport. Unfortunately, the HTTP+gRPC library stack used by the TUI cannot currently support domain sockets (see [open issue](https://github.com/hyperium/tonic/issues/742)).

As a result, usage of the TUI requires the gRPC transport to use TCP. In the OpenSnitch daemon config (`/etc/opensnitchd/default-config.json`), change the `Address` field to a loopback-assigned IP address and port like below:
```sh
$ head -n4 /etc/opensnitchd/default-config.json
{
    "Server":
    {
        "Address":"127.0.0.1:50051",
```

Remember to update your invocation of the official GUI (`opensnitch-ui`) to pass a new flag that binds to this IP and TCP port (`--socket "127.0.0.1:50051"`).

The corresponding flag for this TUI looks like `--bind "127.0.0.1:50051"`.

The instructions above apply when the OpenSnitch daemon and GUI/TUI are running on the same node (loopback address); that address can be modified to any other IP/port combination.

**Note that only one of the GUI or TUI can run at one time, so kill the `opensnitch-ui` or `opensnitch-tui` process to run the other.**

## Disclaimer

I'm pretty new to Rust and am using this project as an exercise to learn more. Use this software at your own risk. Contributions are welcome.
