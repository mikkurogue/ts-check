---@diagnostic disable: undefined-global
local M = {}
local v = vim
local runner = require("ts-analyzer.runner")

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
        
        -- Run ts-analyzer on the file
        local enhanced_diagnostics = runner.run(filepath)
        
        if enhanced_diagnostics then
          -- Match diagnostics by line number and replace messages
          for _, diag in ipairs(result.diagnostics) do
            -- LSP lines are 0-indexed, convert to 1-indexed for matching
            local line = diag.range.start.line + 1
            
            if enhanced_diagnostics[line] then
              -- Replace the diagnostic message with the enhanced one
              diag.message = enhanced_diagnostics[line]
            end
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

  if M.config.attach then
    local diag_cfg = { servers = M.config.servers }
    setup_diagnostic_handler(diag_cfg)
  end
end

return M
