" async-inspect LSP configuration for Vim (vim-lsp)
" Add this to your .vimrc or init.vim

" Install vim-lsp if not already installed:
" Plug 'prabirshrestha/vim-lsp'
" Plug 'mattn/vim-lsp-settings'

" Register async-inspect LSP server
if executable('/path/to/async-inspect/target/release/async-inspect-lsp')
    au User lsp_setup call lsp#register_server({
        \ 'name': 'async-inspect-lsp',
        \ 'cmd': {server_info->['/path/to/async-inspect/target/release/async-inspect-lsp']},
        \ 'allowlist': ['rust'],
        \ })
endif

" Enable LSP for Rust files
function! s:on_lsp_buffer_enabled() abort
    setlocal omnifunc=lsp#complete
    setlocal signcolumn=yes
    if exists('+tagfunc') | setlocal tagfunc=lsp#tagfunc | endif

    " Keybindings
    nmap <buffer> gd <plug>(lsp-definition)
    nmap <buffer> gr <plug>(lsp-references)
    nmap <buffer> K <plug>(lsp-hover)
    nmap <buffer> <leader>ca <plug>(lsp-code-action)
    nmap <buffer> <leader>rn <plug>(lsp-rename)
endfunction

augroup lsp_install
    au!
    autocmd User lsp_buffer_enabled call s:on_lsp_buffer_enabled()
augroup END

" Optional: Enable diagnostics
let g:lsp_diagnostics_enabled = 1
let g:lsp_signs_enabled = 1
let g:lsp_diagnostics_echo_cursor = 1
