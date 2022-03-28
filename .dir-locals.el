;;; Directory Local Variables
;;; For more information see (info "(emacs) Directory Variables")

((org-mode . ((eval (lambda ()
                      (defun org-babel-edit-prep:sql (babel-info)
                        (setq-local buffer-file-name (->> babel-info caddr (alist-get :tangle)))
                        (setq-local lsp-buffer-uri (->> babel-info caddr (alist-get :tangle) lsp--path-to-uri))
                        (setq-local lsp-headerline-breadcrumb-enable nil)
                        (lsp))))))
 (nil . ((eval (lambda ()
                 (setq lsp-sqls-connections `(((driver . "postgresql")
                                               (dataSourceName . ,(getenv "DATABASE_URL"))))))))))
