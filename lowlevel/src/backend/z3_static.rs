use std::ffi::{CStr, CString};

use crate::Backend;

pub struct Z3Static {
    ctx: z3_sys::Z3_context,
}

impl Z3Static {
    pub fn new(timeout: &Option<u64>) -> Result<Self, std::io::Error> {
        Ok(Z3Static {
            ctx: unsafe {
                let cfg = z3_sys::Z3_mk_config();
                if let Some(timeout) = timeout {
                    z3_sys::Z3_set_param_value(cfg, CString::new("timeout").unwrap().as_ptr(), CString::new(timeout.to_string()).unwrap().as_ptr());
                }
                let ctx = z3_sys::Z3_mk_context_rc(cfg);
                z3_sys::Z3_set_error_handler(ctx, None);
                z3_sys::Z3_set_ast_print_mode(ctx, z3_sys::AstPrintMode::SmtLib2Compliant);

                // (set-option :smtlib2_compliant true)
                let s = CString::new("(set-option :smtlib2_compliant true)".to_string()).unwrap();
                z3_sys::Z3_eval_smtlib2_string(ctx, s.as_ptr());

                ctx
            },
        })
    }
}

impl Backend for Z3Static {
    fn exec(&mut self, s: &crate::Command) -> Result<String, crate::Error> {
        let s = CString::new(s.to_string()).unwrap();
        let res = unsafe { z3_sys::Z3_eval_smtlib2_string(self.ctx, s.as_ptr()) };
        let s = unsafe { CStr::from_ptr(res) }.to_str().unwrap().to_string();
        Ok(s)
    }
}
