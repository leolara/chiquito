use core::fmt::Debug;

pub mod expression;
pub mod statement;
pub mod tl;

/// Debug symbol reference, points to the source file, where a AST node comes from.
// TODO: scafolding struct that should be implemented fully.
#[derive(Clone, Debug)]
pub struct DebugSymRef {
    /// Start char position on the source file.
    pub start: usize,
    /// End char position on the source file.
    pub end: usize,
    // TODO: more fields will be added as needed, like file name, etc...
}

impl DebugSymRef {
    pub fn new(start: usize, end: usize) -> DebugSymRef {
        DebugSymRef { start, end }
    }
}

pub struct Variable(pub String, pub i32);

impl Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.1 == 0 {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{}#{}", self.0, self.1)
        }
    }
}

impl From<String> for Variable {
    fn from(value: String) -> Self {
        Variable(value.name(), value.rotation())
    }
}

impl From<&str> for Variable {
    fn from(value: &str) -> Self {
        Variable::from(value.to_string())
    }
}

pub trait Identifier {
    fn rotation(&self) -> i32;
    fn name(&self) -> Self;
}

impl Identifier for String {
    fn rotation(&self) -> i32 {
        assert!(!self.is_empty());
        let last = self.chars().last().unwrap();

        if last == '\'' {
            1
        } else {
            0
        }
    }

    fn name(&self) -> Self {
        let rot = self.rotation();

        match rot {
            0 => self.clone(),
            1 => {
                let mut chars = self.chars();
                chars.next_back();

                chars.as_str().to_string()
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parser::ast::Variable;

    #[test]
    fn test_from_string() {
        let result = Variable::from("abc");

        assert_eq!(result.0, "abc");
        assert_eq!(result.1, 0);

        let result = Variable::from("abc'");

        assert_eq!(result.0, "abc");
        assert_eq!(result.1, 1);
    }
}
