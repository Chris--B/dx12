
## DX12 From Rust

This is a sandbox of sorts. Don't pay it too much mind.

Run `cargo run -- --help` for a full list of options.

```
Dx12 Demo
Draw things with DX12

USAGE:
    dx12.exe [OPTIONS]

OPTIONS:
    -h, --help                             Prints help information
    -w, --force-warp                       Force using the warp adapter
        --debug-layer                      Enable the DX12 runtime debug layer
        --no-debug-layer                   Disable the DX12 runtime debug layer
        --feature-level <feature-level>    Force using a specific feature level for CreateDevice [default: 11_0]
                                           [possible values: 11, 11.0, 11_0, 11.1, 11_1, 12, 12.0, 12_0, 12.1, 12_1]
        --fullscreen                       Create a fullscreen swapchain
        --width <window-width>             Set the application window width. Invalid numbers default to 0.
        --height <window-height>           Set the application window height. Invalid numbers default to 0.
```


Clone and edit locally:
winapi-rs URL: `https://github.com/Chris--B/winapi-rs.git`
