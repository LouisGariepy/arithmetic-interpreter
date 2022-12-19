use crate::parser::{BinaryOperation, Expression, UnaryOperation};

/// Recursively evaluates an expression
pub fn evaluate(expr: Expression) -> f64 {
    match expr {
        // Binary expressions
        Expression::Binary {
            operation,
            lhs,
            rhs,
        } => match operation {
            BinaryOperation::Addition => evaluate(*lhs) + evaluate(*rhs),
            BinaryOperation::Subtraction => evaluate(*lhs) - evaluate(*rhs),
            BinaryOperation::Multiplication => evaluate(*lhs) * evaluate(*rhs),
            BinaryOperation::Division => evaluate(*lhs) / evaluate(*rhs),
        },
        // Unary expressions
        Expression::Unary { operation, operand } => match operation {
            UnaryOperation::Negation => -evaluate(*operand),
        },
        // Atoms
        Expression::Atom(num) => num,
    }
}
