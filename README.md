# Tauri Plugin serialport

A tauri plugin developed based on Serialport.

## Installation

There are three general methods of installation that we can recommend.

1. Pull sources directly from Github using git tags / revision hashes (most secure, good for developement, shown below)
2. Git submodule install this repo in your tauri project and then use file protocol to ingest the source
3. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)

For more details and usage see the example app. Please note, below in the dependencies you can also lock to a revision/tag in both the `Cargo.toml` and `package.json`

### RUST

`src-tauri/Cargo.toml`

```toml
[dependencies.tauri-plugin-serialport]
git = "https://github.com/hChauhan4862/tauri-plugin-serialport"
tag = "v0.1.0"
```

Use in `src-tauri/src/main.rs`:

```RUST
use tauri_plugin_serialport;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_serialport::init())
        .build()
        .run();
}
```

### WEBVIEW

`Install from a tagged release`

```
npm install github:hChauhan4862/tauri-plugin-serialport#v0.1.0
# or
yarn add github:hChauhan4862/tauri-plugin-serialport#v0.1.0
```

`Install from a branch (dev)`

```
npm install https://github.com/hChauhan4862/tauri-plugin-serialport\#master
# or
yarn add https://github.com/hChauhan4862/tauri-plugin-serialport\#master
```

`package.json`

```json
  "dependencies": {
    "tauri-plugin-serialport-api": "github:hChauhan4862/tauri-plugin-serialport#v0.1.0",
  }
```

Use within your JS/TS:

```JS
import { open } from 'tauri-plugin-serialport-api';
```
