-- async-inspect LSP configuration for Neovim
-- Add this to your init.lua or lspconfig setup

local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Define async-inspect LSP if not already defined
if not configs.async_inspect then
  configs.async_inspect = {
    default_config = {
      cmd = { '/path/to/async-inspect/target/release/async-inspect-lsp' },
      filetypes = { 'rust' },
      root_dir = lspconfig.util.root_pattern('Cargo.toml', '.git'),
      settings = {},
    },
    docs = {
      description = [[
https://github.com/async-inspect/async-inspect

Language Server for async-inspect - provides diagnostics and code actions for Rust async code.
      ]],
      default_config = {
        root_dir = [[root_pattern("Cargo.toml", ".git")]],
      },
    },
  }
end

-- Setup the server
lspconfig.async_inspect.setup{
  on_attach = function(client, bufnr)
    -- Enable completion
    vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

    -- Keybindings
    local bufopts = { noremap=true, silent=true, buffer=bufnr }
    vim.keymap.set('n', 'K', vim.lsp.buf.hover, bufopts)
    vim.keymap.set('n', '<leader>ca', vim.lsp.buf.code_action, bufopts)
    vim.keymap.set('n', 'gd', vim.lsp.buf.definition, bufopts)
    vim.keymap.set('n', 'gr', vim.lsp.buf.references, bufopts)
  end,
  capabilities = require('cmp_nvim_lsp').default_capabilities(),
}

-- Auto-format on save (optional)
vim.api.nvim_create_autocmd('BufWritePre', {
  pattern = '*.rs',
  callback = function()
    vim.lsp.buf.format({ async = false })
  end,
})
