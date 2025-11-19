# TS Analyzer 

Prettify errors from tsc type checking with very basic suggestions 

The idea behind this project is to create a "in between" layer for the LSP and the client where we can hook in and replace diagnostics in the buffer, to a more readable and most importantly actionable error.

## Modes

### CLI Mode (for CI/CD pipelines)
Runs `tsc --noEmit` and formats the output with enhanced diagnostics and suggestions.

```bash
# Check entire project
ts-analyzer

# Check specific file
ts-analyzer index.ts
```

### LSP Mode (for editor integrations)
Formats diagnostics directly from LSP events without re-running tsc, providing zero-overhead pretty formatting.

```bash
ts-analyzer --from-lsp \
  --file index.ts \
  --line 2 \
  --column 7 \
  --code TS2322 \
  --message "Type 'string' is not assignable to type 'number'."
```

This mode is used by the Neovim plugin to enhance LSP diagnostics in real-time without the performance hit of re-running the typechecker.

Example output;
<img width="1291" height="704" alt="image" src="https://github.com/user-attachments/assets/fd2ae01b-bc47-470d-bbdf-8d97242577b1" />


## Installation

### Neovim Plugin

Add the following to your Neovim configuration:

```lua
vim.pack.add({
  { src = "https://github.com/mikkurogue/ts-analyzer" },
})

-- configure - these are also the only options lol
require("ts-analyzer").setup({
  attach = true,
  servers = {
    "ts_ls",
    "vtsls"
  }
})
```

Restart Neovim to download the repository. The plugin will automatically build the binary on first load if it's not found.

**Optional: Auto-build on git pull**

To automatically rebuild the binary when you pull updates, install the git hook:

```bash
cd ~/.local/share/nvim/site/pack/core/opt/ts-analyzer
cp post-merge.hook .git/hooks/post-merge
chmod +x .git/hooks/post-merge
```

**Manual build**

You can also manually build at any time:

```bash
cd ~/.local/share/nvim/site/pack/core/opt/ts-analyzer
cargo build --release
```

**Note:** Lazy.nvim and Packer instructions coming soon.

### CLI Usage

Currently making the assumption that you will clone the repo and run `cargo install --path .`. then just simply run `ts-analyzer` in a typescript project with a tsconfig.json, or to analyze a single file run `ts-analyzer <path-to-file>.ts`


Inspired by the GOAT [Dillon Mulroy](https://github.com/dmmulroy), where he made a nicer tsc reporter neovim plugin and i stole half of the stuff from him to even get it running in neovim now :D.
