//! TemplateRegistry — in-memory catalog of all registered templates (§6.2).

use crate::template::Template;
use std::collections::HashMap;

/// Central registry — holds Default and Stakeholder templates by name.
pub struct TemplateRegistry {
    templates: HashMap<&'static str, Template>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        TemplateRegistry {
            templates: HashMap::new(),
        }
    }

    /// Register a template. Returns false if the name already exists.
    pub fn register(&mut self, t: Template) -> bool {
        if self.templates.contains_key(t.name) {
            return false;
        }
        self.templates.insert(t.name, t);
        true
    }

    pub fn get(&self, name: &str) -> Option<&Template> {
        self.templates.get(name)
    }

    pub fn len(&self) -> usize {
        self.templates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    pub fn names(&self) -> impl Iterator<Item = &&'static str> {
        self.templates.keys()
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{FieldSpec, Template};

    fn make(name: &'static str) -> Template {
        Template::default_template(name, "desc", &[FieldSpec::new(0x1A4B, "state", true)])
    }

    #[test]
    fn register_and_get() {
        let mut reg = TemplateRegistry::new();
        assert!(reg.register(make("p.basic")));
        assert!(reg.get("p.basic").is_some());
    }

    #[test]
    fn duplicate_register_returns_false() {
        let mut reg = TemplateRegistry::new();
        reg.register(make("p.basic"));
        assert!(!reg.register(make("p.basic")));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn unknown_name_returns_none() {
        let reg = TemplateRegistry::new();
        assert!(reg.get("does.not.exist").is_none());
    }
}
