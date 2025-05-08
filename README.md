# Supipi 🐰

A lightweight daemon that does cool things with Super key presses (double-tap, long-press, etc.). Written in Rust for minimal resource usage and blazing speed! 🚀

Currently, Supipi can launch **Wofi** with a double-tap of the Super key. It's a work-in-progress, built by a beginner, so please be kind! 🙏

> **Tested on**: ArchLinux with Hyprland.  
> **Requirements**: Event detection and `uinput` permissions (see Setup below).  
> **Future plans**: Service setup, better docs, and more features!

---

## Features
- Double-tap the Super key to launch Wofi.
- Lightweight and fast, thanks to Rust.
- Hackable! Customize it for your environment.

## Setup
Supipi is still rough around the edges (no fancy docs or config files yet, sorry!). Here's how to get it running:

1. **Check permissions** for `/dev/uinput` and `/dev/input/event*`:
   ```bash
   ls -l /dev/uinput  # Should show: crw-rw---- yourname uinput
   ls -l /dev/input/event4  # Should show: crw-rw---- yourname uinput

If permissions aren't set, you may need to tweak udev rules for your setup. Check out this guide for help.

    Customize the code:
        Edit the source to match your keyboard's input events.
        Example: Modify the event detection logic in src/main.rs to suit your device.
    Build and run:
    bash

    cargo build --release
    ./target/release/supipi

    Note: This is a beginner project, so expect some tinkering! If you're stuck, open an issue or ping me.

Current Limitations

    No proper documentation or config files yet (working on it!).
    Only tested on ArchLinux with Hyprland.
    Service setup for background running is still TBD.

Contributing

I'm new to this, so any feedback, bug reports, or code contributions are super welcome! 🌟

Here's how you can help:

    Hack the code to add new key combos or features.
    Share tips on udev rules or systemd service setup.
    Open an issue or PR on GitHub.

Feel free to throw ideas at me! 😄
Why Supipi?

"Supipi" is a cute name for a daemon that's always working in the background, listening for your Super key presses. I chose Rust to make it fast and light, perfect for a daemon that runs 24/7.

Thanks for checking out Supipi! 🐰