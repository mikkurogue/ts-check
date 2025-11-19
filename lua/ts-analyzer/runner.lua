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

---Parse ts-analyzer output into a table of diagnostics by line number
---@param output string The output from ts-analyzer
---@return table<number, string> Map of line numbers to enhanced diagnostic messages
local function parse_output(output)
  local diagnostics = {}

  -- Strip ANSI color codes
  output = output:gsub("\27%[[%d;]*m", "")

  -- Split output into individual error blocks (separated by blank lines)
  local current_error = {}
  local current_line = nil

  for line in output:gmatch("[^\r\n]+") do
    -- Match line number from the location pattern: "╭─[ file.ts:LINE:COL ]"
    -- Updated pattern to handle the actual format with spaces
    local line_num = line:match("╭─%[%s*[^:]+:(%d+):%d+%s*%]")

    if line_num then
      -- Save previous error if exists
      if current_line and #current_error > 0 then
        diagnostics[current_line] = table.concat(current_error, "\n")
      end
      -- Start new error
      current_line = tonumber(line_num)
      current_error = {}
    end

    -- Collect all lines for current error (skip "Total errors:" line)
    if current_line and not line:match("^Total errors:") then
      table.insert(current_error, line)
    end
  end

  -- Save last error
  if current_line and #current_error > 0 then
    diagnostics[current_line] = table.concat(current_error, "\n")
  end

  return diagnostics
end

---Run ts-analyzer binary on a file and return parsed diagnostics
---@param filepath string The path to the TypeScript file
---@return table<number, string>|nil Map of line numbers to diagnostic messages or nil on error
function M.run(filepath)
  if not filepath or filepath == "" then
    return nil
  end

  -- Check if binary exists
  if vim.fn.filereadable(bin) ~= 1 then
    vim.notify("ts-analyzer binary not found at: " .. bin, vim.log.levels.WARN)
    return nil
  end

  -- Run the binary with the file path
  local handle = io.popen(bin .. " " .. vim.fn.shellescape(filepath) .. " 2>&1")
  if not handle then
    return nil
  end

  local result = handle:read("*a")
  handle:close()

  -- Parse the output into line-based diagnostics
  return parse_output(result)
end

return M
