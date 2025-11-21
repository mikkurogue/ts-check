# ts-analyzer-virt

A Neovim plugin that displays enhanced TypeScript diagnostics with beautiful, hover-activated virtual text overlays.

## ✨ Features

- 🎯 **Hover-activated diagnostics** - Virtual text appears only when cursor is on the error line
- 🎨 **Syntax-highlighted ASCII art** - Beautiful box-drawing characters with colorized output
- 💡 **In-buffer highlighting** - Subtle background highlight on error ranges
- 🧹 **Clean, minimal output** - Shows only the essential: ASCII pointers and error messages
- 🚀 **Zero overhead** - Diagnostics only appear on hover, keeping your buffer clean

## 📦 Installation

### Using lazy.nvim

```lua
{
  "mikkurogue/ts-analyzer",
  dependencies = {
    "neovim/nvim-lspconfig", -- Required for LSP
  },
  build = "cargo build --release",
  config = function()
    require("ts-analyzer-virt").setup({
      attach = true,
      servers = { "ts_ls", "vtsls" }, -- TypeScript LSP servers to enhance
    })
  end,
}
```

### Using Neovim's native package manager

Add this to your `init.lua`:

```lua
-- Add the plugin
vim.pack.add({
  { src = "https://github.com/mikkurogue/ts-analyzer" },
})

-- Auto-build the Rust binary on install/update
vim.api.nvim_create_autocmd("PackChanged", {
  callback = function(ev)
    local spec = ev.data.spec
    if spec and spec.name == "ts-analyzer" and (ev.data.kind == "install" or ev.data.kind == "update") then
      local ts_analyzer_path = vim.fn.stdpath("data") .. "/site/pack/core/opt/ts-analyzer"
      vim.fn.jobstart({ "cargo", "build", "--release" }, {
        cwd = ts_analyzer_path,
        on_exit = function(_, code)
          if code == 0 then
            vim.notify("[ts-analyzer] Build completed successfully", vim.log.levels.INFO)
          else
            vim.notify("[ts-analyzer] Build failed", vim.log.levels.ERROR)
          end
        end,
      })
    end
  end,
})

-- Setup the plugin
require("ts-analyzer-virt").setup({
  attach = true,
  servers = { "ts_ls", "vtsls" },
})
```

### Local development setup

If you're developing locally:

```lua
-- Add local plugin to runtimepath
vim.opt.runtimepath:append("/path/to/ts-analyzer")

-- Setup the plugin
require("ts-analyzer-virt").setup({
  attach = true,
  servers = { "ts_ls", "vtsls" },
})
```

## 🎮 Usage

Once installed, the plugin works automatically! Simply:

1. Open a TypeScript file with errors
2. Move your cursor over a line with a diagnostic
3. Watch the enhanced diagnostic appear below with ASCII art pointers

### Commands

```lua
-- Clear virtual text for current buffer
:lua require("ts-analyzer-virt").clear()
```

## 🎨 How it works

1. **Intercepts LSP diagnostics** - Hooks into `textDocument/publishDiagnostics` 
2. **Enhances with ts-analyzer** - Runs each diagnostic through the Rust binary
3. **Parses ASCII output** - Extracts the relevant pointer lines and messages
4. **Highlights error ranges** - Adds subtle background to the problematic code
5. **Shows on hover** - Displays virtual text only when cursor is on the error line

## 📸 Example

When you hover over an error, instead of just:
```
'notExported' is not exported from the module
```

You see:
```typescript
import { anotherExported, exported, notExported } from "./export-smth";
//                                   ~~~~~~~~~~~~  ← highlighted in buffer
                                     ─────┬─────
                                          ╰─────── `notExported` is not exported from the module.
```

The error range is highlighted in your buffer, and the ASCII art appears below on hover!

## ⚙️ Configuration

### Default configuration

```lua
require("ts-analyzer-virt").setup({
  attach = true,                    -- Auto-attach to LSP servers
  servers = { "ts_ls", "vtsls" },  -- LSP servers to enhance
})
```

### Customize highlight colors

```lua
-- Override highlight groups to match your theme
vim.api.nvim_set_hl(0, "TsAnalyzerError", { fg = "#f38ba8", bold = true, bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerCode", { fg = "#fab387", bold = true, bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerBorder", { fg = "#585b70", bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerHelp", { fg = "#a6e3a1", bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerMessage", { fg = "#bac2de", bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerUnderline", { fg = "#f38ba8", bg = "#1e1e2e" })
vim.api.nvim_set_hl(0, "TsAnalyzerHighlight", { bg = "#3e2e3e", underline = true, sp = "#f38ba8" })
```

## 🔧 Requirements

- Neovim >= 0.10.0
- Rust and Cargo (for building the binary)
- A TypeScript LSP server (`ts_ls` or `vtsls`)
- Git (for package manager installation)

## 🤝 Contributing

This is a POC plugin. Feel free to fork and customize for your needs!

## 📝 License

MIT
