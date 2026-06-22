//! Template substitution for AC_SUBST, AC_CONFIG_FILES (stub).
//!
//! Current status: Phase 1 — stub.

#[derive(Default)]
pub struct TemplateEngine;

impl TemplateEngine {
    pub fn new() -> Self {
        Self
    }
    pub fn substitute(&self, _template: &str, _vars: &[(String, String)]) -> String {
        _template.to_string()
    }
}
