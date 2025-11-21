---@diagnostic disable: undefined-global
local M = {}
local runner = require("ts-analyzer.runner")

-- Namespace for our virtual text
local ns = vim.api.nvim_create_namespace("ts-analyzer-virt")

-- Store original diagnostics for reference
local original_handler = vim.lsp.handlers["textDocument/publishDiagnostics"]

-- Define custom highlight groups
local function setup_highlights()
  vim.api.nvim_set_hl(0, "TsAnalyzerError", { fg = "#f38ba8", bold = true, bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerCode", { fg = "#fab387", bold = true, bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerBorder", { fg = "#585b70", bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerFile", { fg = "#89b4fa", bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerHelp", { fg = "#a6e3a1", bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerMessage", { fg = "#bac2de", bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerUnderline", { fg = "#f38ba8", bg = "#1e1e2e" })
  vim.api.nvim_set_hl(0, "TsAnalyzerHighlight", { bg = "#3e2e3e", underline = true, sp = "#f38ba8" })
end

-- Call setup on load
setup_highlights()

local function get_lsp_client_name_by_id(id)
  local client = vim.lsp.get_client_by_id(id)
  return client and client.name or "unknown"
end

-- Clear virtual text for a buffer
local function clear_virt_text(bufnr)
  vim.api.nvim_buf_clear_namespace(bufnr, ns, 0, -1)
end

-- Parse and colorize a single line
local function colorize_line(line)
  local chunks = {}
  
  -- Error code pattern: [TS6133]
  if line:match("^%[TS%d+%]") then
    local code = line:match("^(%[TS%d+%])")
    local rest = line:sub(#code + 1)
    table.insert(chunks, { code, "TsAnalyzerCode" })
    if rest:match("^ Error:") then
      table.insert(chunks, { " Error:", "TsAnalyzerError" })
      table.insert(chunks, { rest:sub(8), "TsAnalyzerMessage" })
    else
      table.insert(chunks, { rest, "TsAnalyzerMessage" })
    end
    return chunks
  end
  
  -- File path pattern: ╭─[ index.ts:1:10 ]
  if line:match("╭─%[") or line:match("───╯") then
    return {{ line, "TsAnalyzerBorder" }}
  end
  
  -- Help pattern
  if line:match("Help:") then
    local prefix_match = line:match("^(%s*│%s*)")
    if prefix_match then
      local help_text = line:sub(#prefix_match + 1)
      table.insert(chunks, { prefix_match, "TsAnalyzerBorder" })
      table.insert(chunks, { help_text, "TsAnalyzerHelp" })
      return chunks
    end
  end
  
  -- Line with underline/arrows (─┬─ or ╰─)
  if line:match("─") or line:match("╰") then
    return {{ line, "TsAnalyzerUnderline" }}
  end
  
  -- Border character at start
  if line:match("^%s*│") then
    local border = line:match("^(%s*│)")
    if border then
      local rest = line:sub(#border + 1)
      table.insert(chunks, { border, "TsAnalyzerBorder" })
      table.insert(chunks, { rest, "TsAnalyzerMessage" })
      return chunks
    end
  end
  
  -- Default
  return {{ line, "TsAnalyzerMessage" }}
end

-- Parse diagnostic output to extract only ASCII art and message
local function extract_diagnostic_snippet(text)
  local lines = vim.split(text, "\n", { plain = true })
  local snippet = {}
  
  for _, line in ipairs(lines) do
    -- Keep only the underline/arrow lines (without the │ prefix)
    if line:match("^%s*│") and (line:match("─") or line:match("╰")) then
      -- Remove the │ prefix and keep just the ASCII art
      local content = line:match("^%s*│(.*)$")
      if content then
        table.insert(snippet, content)
      end
    end
    
    ::continue::
  end
  
  return snippet
end

-- Store diagnostics data for each buffer
local buffer_diagnostics = {}

-- Highlight the error range in the buffer
local function highlight_error_range(bufnr, diag)
  local start_line = diag.range.start.line
  local start_col = diag.range.start.character
  local end_line = diag.range["end"].line
  local end_col = diag.range["end"].character
  
  -- Add highlight to the error range
  vim.api.nvim_buf_set_extmark(bufnr, ns, start_line, start_col, {
    end_row = end_line,
    end_col = end_col,
    hl_group = "TsAnalyzerHighlight",
    priority = 100,
  })
end

-- Display diagnostic as overlay virtual lines (only on hover)
local function show_enhanced_diagnostic(bufnr, diag, enhanced_text)
  local snippet = extract_diagnostic_snippet(enhanced_text)
  
  if #snippet == 0 then
    return
  end
  
  local line_num = diag.range.start.line
  
  -- Store diagnostic data for this line
  if not buffer_diagnostics[bufnr] then
    buffer_diagnostics[bufnr] = {}
  end
  
  buffer_diagnostics[bufnr][line_num] = {
    snippet = snippet,
    diag = diag,
  }
  
  -- Highlight the error range in the buffer
  highlight_error_range(bufnr, diag)
end

-- Show virtual text when cursor is on the diagnostic line
local function show_virt_text_on_hover(bufnr, line_num)
  local data = buffer_diagnostics[bufnr] and buffer_diagnostics[bufnr][line_num]
  if not data then
    return
  end
  
  -- Create virtual lines with colorized chunks
  local virt_lines = {}
  for _, line in ipairs(data.snippet) do
    table.insert(virt_lines, colorize_line(line))
  end
  
  -- Place virtual lines at the diagnostic position
  vim.api.nvim_buf_set_extmark(bufnr, ns, line_num, 0, {
    virt_lines = virt_lines,
    virt_lines_above = false,
    id = line_num + 1000000, -- Unique ID for this line's virtual text
  })
end

-- Clear virtual text when cursor leaves
local function clear_hover_virt_text(bufnr)
  vim.api.nvim_buf_clear_namespace(bufnr, ns, 0, -1)
  
  -- Re-apply highlights
  if buffer_diagnostics[bufnr] then
    for _, data in pairs(buffer_diagnostics[bufnr]) do
      highlight_error_range(bufnr, data.diag)
    end
  end
end

-- Setup cursor hover handlers
local function setup_hover_handlers(bufnr)
  local group = vim.api.nvim_create_augroup("TsAnalyzerVirtHover_" .. bufnr, { clear = true })
  
  vim.api.nvim_create_autocmd("CursorMoved", {
    group = group,
    buffer = bufnr,
    callback = function()
      local cursor = vim.api.nvim_win_get_cursor(0)
      local line_num = cursor[1] - 1 -- 0-indexed
      
      clear_hover_virt_text(bufnr)
      show_virt_text_on_hover(bufnr, line_num)
    end,
  })
  
  vim.api.nvim_create_autocmd("BufLeave", {
    group = group,
    buffer = bufnr,
    callback = function()
      clear_hover_virt_text(bufnr)
    end,
  })
end

local function setup_diagnostic_handler(opts)
  vim.lsp.handlers["textDocument/publishDiagnostics"] = function(err, result, ctx, config)
    if result and result.diagnostics then
      local client_name = get_lsp_client_name_by_id(ctx.client_id)
      
      if vim.tbl_contains(opts.servers, client_name) then
        local filepath = vim.uri_to_fname(result.uri)
        local bufnr = vim.uri_to_bufnr(result.uri)
        
        -- Clear previous virtual text and diagnostics
        clear_virt_text(bufnr)
        buffer_diagnostics[bufnr] = {}
        
        -- Setup hover handlers for this buffer (only once)
        setup_hover_handlers(bufnr)
        
        -- Process each diagnostic
        for _, diag in ipairs(result.diagnostics) do
          local line = diag.range.start.line + 1
          local column = diag.range.start.character + 1
          
          local code = diag.code
          if type(code) == "table" and code.value then
            code = code.value
          end
          code = tostring(code or "")
          
          if code ~= "" and not code:match("^TS") then
            code = "TS" .. code
          end
          
          -- Get enhanced diagnostic
          local enhanced = runner.format_diagnostic(
            filepath,
            line,
            column,
            code,
            diag.message
          )
          
          if enhanced then
            -- Store and highlight (show on hover)
            show_enhanced_diagnostic(bufnr, diag, enhanced)
            
            -- Keep original message short for the diagnostic list
            diag.message = string.format("[%s] %s", code, diag.message)
          end
        end
      end
    end
    
    -- Call original handler
    return original_handler(err, result, ctx, config)
  end
end

local DEF_OPTS = {
  attach = true,
  servers = {
    "ts_ls",
    "vtsls"
  }
}

M.config = DEF_OPTS

function M.setup(opts)
  opts = opts or {}
  M.config = vim.tbl_deep_extend("force", DEF_OPTS, opts)
  
  if M.config.attach then
    setup_diagnostic_handler({ servers = M.config.servers })
  end
end

-- Manually toggle virtual text display
function M.toggle()
  -- TODO: implement toggle logic
  vim.notify("Toggle not yet implemented", vim.log.levels.INFO)
end

-- Clear virtual text for current buffer
function M.clear()
  clear_virt_text(vim.api.nvim_get_current_buf())
end

return M
