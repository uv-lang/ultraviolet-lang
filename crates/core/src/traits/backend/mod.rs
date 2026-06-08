pub mod number_ops;

pub trait TypeOf {
    /// Returns string-representation of provided type
    fn typeof_str(&self) -> String;
}
