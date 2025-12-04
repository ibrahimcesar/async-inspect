;; async-inspect LSP configuration for Emacs (lsp-mode)
;; Add this to your init.el or .emacs

(require 'lsp-mode)

;; Register async-inspect LSP server
(lsp-register-client
 (make-lsp-client
  :new-connection (lsp-stdio-connection '("/path/to/async-inspect/target/release/async-inspect-lsp"))
  :major-modes '(rust-mode rustic-mode)
  :server-id 'async-inspect
  :priority 1))

;; Enable LSP in Rust buffers
(add-hook 'rust-mode-hook #'lsp-deferred)
(add-hook 'rustic-mode-hook #'lsp-deferred)

;; Optional: Configure lsp-ui for better diagnostics display
(use-package lsp-ui
  :commands lsp-ui-mode
  :config
  (setq lsp-ui-doc-enable t
        lsp-ui-doc-position 'at-point
        lsp-ui-sideline-enable t
        lsp-ui-sideline-show-diagnostics t
        lsp-ui-sideline-show-code-actions t))

;; Optional: Keybindings
(define-key rust-mode-map (kbd "C-c l h") 'lsp-describe-thing-at-point)
(define-key rust-mode-map (kbd "C-c l a") 'lsp-execute-code-action)
(define-key rust-mode-map (kbd "C-c l r") 'lsp-rename)
