<div align="center">

# Clipboard Buddy

[![Release](https://img.shields.io/github/release/evilsocket/clipbud.svg?style=flat-square)](https://github.com/evilsocket/clipbud/releases/latest)
[![Rust Report](https://rust-reportcard.xuri.me/badge/github.com/evilsocket/clipbud)](https://rust-reportcard.xuri.me/report/github.com/evilsocket/clipbud)
[![CI](https://img.shields.io/github/actions/workflow/status/evilsocket/clipbud/ci.yml)](https://github.com/evilsocket/clipbud/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-GPL3-brightgreen.svg?style=flat-square)](https://github.com/evilsocket/clipbud/blob/master/LICENSE.md)
![Human Coded](https://img.shields.io/badge/human-coded-brightgreen?logo=data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIyNCIgaGVpZ2h0PSIyNCIgdmlld0JveD0iMCAwIDI0IDI0IiBmaWxsPSJub25lIiBzdHJva2U9IiNmZmZmZmYiIHN0cm9rZS13aWR0aD0iMiIgc3Ryb2tlLWxpbmVjYXA9InJvdW5kIiBzdHJva2UtbGluZWpvaW49InJvdW5kIiBjbGFzcz0ibHVjaWRlIGx1Y2lkZS1wZXJzb24tc3RhbmRpbmctaWNvbiBsdWNpZGUtcGVyc29uLXN0YW5kaW5nIj48Y2lyY2xlIGN4PSIxMiIgY3k9IjUiIHI9IjEiLz48cGF0aCBkPSJtOSAyMCAzLTYgMyA2Ii8+PHBhdGggZD0ibTYgOCA2IDIgNi0yIi8+PHBhdGggZD0iTTEyIDEwdjQiLz48L3N2Zz4=)
 
  <small>Join the project community on our server!</small>
  <br/><br/>
  <a href="https://discord.gg/btZpkp45gQ" target="_blank" title="Join our community!">
    <img src="https://dcbadge.limes.pink/api/server/https://discord.gg/btZpkp45gQ"/>
  </a>

</div>

Clipboard Buddy (`clipbud`) is a cross platform utility that interacts with your system clipboard and augments it with AI capabilities. You can register a set of custom action prompts and recall them at any time on your clipboard contents using a custom global hotkey. The power of this mechanism is that it works with any application since it reads and writes directly from and to the system clipboard. Avoid repetitive copy/paste to and from your LLM!

<div align="center">
  <img alt="Clipboard Buddy" src="https://raw.githubusercontent.com/evilsocket/clipbud/main/clipbud.gif" />
</div>

## Quick Start

`clipbud` is published as a binary crate on [crates.io](https://crates.io/crates/clipbud). If you have [Cargo installed](https://rustup.rs/), you can:

```sh
cargo install clipbud
```

This will compile its sources and install the binary in `$HOME/.cargo/bin/clipbud`. You are now ready to go! 🚀

```bash
export OPENAI_API_KEY=...

clipbud -c /path/to/config.yml
```

An example configuration file:

```yaml
# if this is not set, clipbud will show itself at every clipboard change
hotkey: "CMD+CTRL+C"

actions:
  - label: "Fix"
    prompt: "Fix typos and grammar of the following text, but keep the original meaning and structure, only return the fixed text and nothing else:"
    key: "T" # optional shortcut key
    model: "gpt-4o"
    provider: "openai"

  - label: "Summarize"
    prompt: "Summarize the following text in less than 200 words, only return the summary and nothing else:"
    key: "S"
    model: "gpt-4o"
    provider: "openai"

  - label: "Friendly"
    prompt: "Make the following text more friendly, only return the friendly text and nothing else:"
    key: "F"
    model: "gpt-4o"
    provider: "openai"

  - label: "ELI5"
    prompt: "Explain the following text in a way that is easy to understand for a 5 year old, only return the explanation and nothing else:"
    key: "E"
    model: "gpt-4o"
    provider: "openai"
```

## Contributors

<a href="https://github.com/evilsocket/clipbud/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=evilsocket/clipbud" alt="Clipboard Buddy project contributors" />
</a>

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=evilsocket/clipbud&type=Timeline)](https://www.star-history.com/#evilsocket/clipbud&Timeline)

## License

Clipboard Buddy is released under the GPL 3 license. To see the licenses of the project dependencies, install cargo license with `cargo install cargo-license` and then run `cargo license`.