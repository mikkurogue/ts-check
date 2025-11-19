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
        print(string.format("[ts-analyzer parser] Saved diagnostic for line %d", current_line))
      end
      
      -- Start new error
      current_line = tonumber(line_num)
      current_error = {}
      print(string.format("[ts-analyzer parser] Found error at line %d", current_line))
    end
    
    -- Collect all lines for current error (skip "Total errors:" line)
    if current_line and not line:match("^Total errors:") then
      table.insert(current_error, line)
    end
  end
  
  -- Save last error
  if current_line and #current_error > 0 then
    diagnostics[current_line] = table.concat(current_error, "\n")
    print(string.format("[ts-analyzer parser] Saved diagnostic for line %d", current_line))
  end
  
  print(string.format("[ts-analyzer parser] Parsed %d diagnostics from lines: %s", 
    vim.tbl_count(diagnostics), 
    vim.inspect(vim.tbl_keys(diagnostics))))
  
  return diagnostics
end

---Run ts-analyzer binary on a file and return parsed diagnostics
---@param filepath string The path to the TypeScript file
---@return table<number, string>|nil Map of line numbers to diagnostic messages or nil on error
function M.run(filepath)
  if not filepath or filepath == "" then
    print("[ts-analyzer] No filepath provided")
    return nil
  end

  -- Check if binary exists
  if vim.fn.filereadable(bin) ~= 1 then
    vim.notify("ts-analyzer binary not found at: " .. bin, vim.log.levels.WARN)
    return nil
  end

  print(string.format("[ts-analyzer] Running binary: %s %s", bin, filepath))

  -- Run the binary with the file path
  local handle = io.popen(bin .. " " .. vim.fn.shellescape(filepath) .. " 2>&1")
  if not handle then
    print("[ts-analyzer] Failed to open process")
    return nil
  end

  local result = handle:read("*a")
  handle:close()

  print(string.format("[ts-analyzer] Got output (%d bytes)", #result))

  -- Parse the output into line-based diagnostics
  return parse_output(result)
end

return M
