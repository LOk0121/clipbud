<div align="center">

# Clipboard Buddy

[![Release](https://img.shields.io/github/release/evilsocket/clipbud.svg?style=flat-square)](https://github.com/evilsocket/clipbud/releases/latest)
[![Rust Report](https://rust-reportcard.xuri.me/badge/github.com/evilsocket/clipbud)](https://rust-reportcard.xuri.me/report/github.com/evilsocket/clipbud)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/clipbud/ci.yml)](https://github.com/evilsocket/clipbud/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/clipbud/blob/master/LICENSE.md)
![Human Coded](https://img.shields.io/badge/human-coded-brightgreen?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBjbGFzcz0ibHVjaWRlIGx1Y2lkZS1wZXJzb24tc3RhbmRpbmctaWNvbiBsdWNpZGUtcGVyc29uLXN0YW5kaW5nIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjUiIHI9IjEiLz48cGF0aCBkPSJtOSAyMCAzLTYgMyA2Ii8+PHBhdGggZD0ibTYgOCA2IDIgNi0yIi8+PHBhdGggZD0iTTEyIDEwdjQiLz48L3N2Zz4=)

</div>

<div align="center">
    <img alt="Clipboard Buddy" src="https://raw.githubusercontent.com/evilsocket/clipbud/main/assets/icon-256.png" width="150"/>
</div>

Clipboard Buddy (`clipbud`) is a cross platform utility that interacts with your system clipboard and augments it with AI capabilities. You can register a set of custom action prompts and recall them at any time on your clipboard contents using a custom global hotkey. The power of this mechanism is that it works with any application since it reads and writes directly from and to the system clipboard. Avoid repetitive copy/paste to and from your LLM!

<div align="center">
  <img alt="Clipboard Buddy Demo" src="https://raw.githubusercontent.com/evilsocket/clipbud/main/assets/demo.gif" />
</div>

## Quick Start

If you have [Cargo installed](https://rustup.rs/), you can:

```sh
cargo install --git https://github.com/evilsocket/clipbud.git
```

This will compile its sources and install the binary in `$HOME/.cargo/bin/clipbud`. You are now ready to go! 🚀


```bash
# you can also set this via config file
export OPENAI_API_KEY=...

# default configuration loaded from ~/.clipbud/config.yml
clipbud
```

An example configuration file can be found in [config.yml](https://github.com/evilsocket/clipbud/blob/main/config.yml). 
For a list of all supported LLM providers [refer to this page](https://docs.rig.rs/docs/integrations/model_providers).

## Contributors

<a href="https://github.com/evilsocket/clipbud/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=evilsocket/clipbud" alt="Clipboard Buddy project contributors" />
</a>

<div align="center">
  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>
</div>

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/clipbud&type=Timeline)](https://www.star-history.com/#evilsocket/clipbud&Timeline)

## License

Clipboard Buddy is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.
