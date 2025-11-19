---@diagnostic disable: undefined-global
local M = {}
local v = vim
local runner = require("ts-analyzer.runner")

-- Auto-build binary if missing
local function ensure_binary()
  local source = debug.getinfo(1, "S").source:sub(2)
  local script_dir = source:match("(.*/)")
  local root = script_dir:match("(.*/)lua/ts%-analyzer/$")
  if not root then
    root = script_dir:match("(.*/)"):match("(.*/)")
  end
  if root and not root:match("/$") then
    root = root .. "/"
  end
  
  local bin = root and (root .. "target/release/ts-analyzer")
  if not bin or v.fn.filereadable(bin) ~= 1 then
    v.notify("ts-analyzer binary not found. Building...", v.log.levels.INFO)
    local build_cmd = string.format("cd %s && cargo build --release --quiet 2>&1", root)
    local result = v.fn.system(build_cmd)
    if v.v.shell_error == 0 then
      v.notify("ts-analyzer binary built successfully", v.log.levels.INFO)
    else
      v.notify("Failed to build ts-analyzer binary: " .. result, v.log.levels.ERROR)
    end
  end
end

---@class Config
---@field attach boolean Auto-attach to LSP servers (default: true)
---@field servers string[] LSP server names to translate diagnostics for

-- All of this is stolen from the goat @dmmulroy
local function get_lsp_client_name_by_id(id)
  local client = v.lsp.get_client_by_id(id)
  local name = client and client.name or "unknown"
  return name
end

-- All of this is stolen from the goat @dmmulroy
local function setup_diagnostic_handler(opts)
  local original_diagnostics = v.lsp.handlers["textDocument/publishDiagnostics"]

  v.lsp.handlers["textDocument/publishDiagnostics"] = function(err, result, ctx, config)
    if result and result.diagnostics then
      local client_name = get_lsp_client_name_by_id(ctx.client_id)

      if v.tbl_contains(opts.servers, client_name) then
        -- Get the file path from the URI
        local filepath = v.uri_to_fname(result.uri)

        -- Enhance each diagnostic individually using LSP mode
        for _, diag in ipairs(result.diagnostics) do
          -- LSP lines are 0-indexed, convert to 1-indexed
          local line = diag.range.start.line + 1
          -- LSP columns are 0-indexed, convert to 1-indexed
          local column = diag.range.start.character + 1
          
          -- Extract error code from diagnostic
          local code = diag.code
          if type(code) == "table" and code.value then
            code = code.value
          end
          code = tostring(code or "")
          
          -- Get enhanced diagnostic from ts-analyzer in LSP mode
          local enhanced = runner.format_diagnostic(
            filepath,
            line,
            column,
            code,
            diag.message
          )

          if enhanced then
            -- Replace the diagnostic message with the enhanced one
            diag.message = enhanced
          end
        end
      end
    end
    return original_diagnostics(err, result, ctx, config)
  end
end

-- @type Config
local DEF_OPTS = {
  -- auto attach to the lsp
  attach = true,
  -- list of lsp server names to attach to
  -- only supporting these two for now cause im lazy and dont want to deal with testing
  servers = {
    "ts_ls",
    "vtsls"
  }
}

-- @type Config
M.config = DEF_OPTS


function M.setup(opts)
  opts = opts or {}

  M.config = v.tbl_deep_extend("force", DEF_OPTS, opts)

  -- Ensure binary exists, build if missing
  ensure_binary()

  if M.config.attach then
    local diag_cfg = { servers = M.config.servers }
    setup_diagnostic_handler(diag_cfg)
  end
end

return M
