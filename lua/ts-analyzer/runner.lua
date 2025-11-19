---@diagnostic disable: undefined-global
local M = {}

-- Get the directory where this file is located
local source = debug.getinfo(1, "S").source:sub(2)
local script_dir = source:match("(.*/)")

-- Go up from lua/ts-analyzer/ to the plugin root
local root = script_dir:match("(.*/)lua/ts%-analyzer/$")
if not root then
  -- Fallback: go up 2 directories
  root = script_dir:match("(.*/)"):match("(.*/)")
end

local bin = root .. "target/release/ts-analyzer"

---Run ts-analyzer in LSP mode with a single diagnostic
---@param filepath string The path to the TypeScript file
---@param line number Line number (1-indexed)
---@param column number Column number (1-indexed) 
---@param code string Error code (e.g., "TS2322")
---@param message string Error message
---@return string|nil Enhanced diagnostic message or nil on error
function M.format_diagnostic(filepath, line, column, code, message)
  if not filepath or filepath == "" then
    return nil
  end

  -- Check if binary exists
  if vim.fn.filereadable(bin) ~= 1 then
    vim.notify("ts-analyzer binary not found at: " .. bin, vim.log.levels.WARN)
    return nil
  end

  -- Build command with LSP mode flags
  local cmd = string.format(
    "%s --from-lsp --file %s --line %d --column %d --code %s --message %s 2>&1",
    bin,
    vim.fn.shellescape(filepath),
    line,
    column,
    vim.fn.shellescape(code),
    vim.fn.shellescape(message)
  )

  -- Run the binary
  local handle = io.popen(cmd)
  if not handle then
    return nil
  end

  local result = handle:read("*a")
  handle:close()

  -- Strip ANSI color codes for display in diagnostics
  result = result:gsub("\27%[[%d;]*m", "")

  return result ~= "" and result or nil
end

return M
