// crates/descar-core/src/error/error_code.rs

//! Error codes, severity levels, and compiler phases of the Descar compiler.
//!
//! [`ErrorCode`] gives each diagnostic a unique code, for example `E0001`. A code has a
//! text form ([`ErrorCode::code`]), a numeric form ([`ErrorCode::numeric_code`]), a severity
//! ([`ErrorCode::severity`]), a compiler phase ([`ErrorCode::phase`]), a short message
//! ([`ErrorCode::message`]), an explanation ([`ErrorCode::explanation`]), and a list of
//! suggestions ([`ErrorCode::suggestions`]).
//!
//! # Error Code Ranges
//!
//! The first digit of the numeric code selects the compiler phase.
//!
//! | Range | Phase | Description |
//! |-------|-------|-------------|
//! | E0001-E0999 | Lexical Analysis | Token recognition, literals, comments |
//! | E1001-E1999 | Parsing | Syntax structure, grammar violations |
//! | E2001-E2999 | Semantic Analysis | Types, scopes, declarations |
//! | E3001-E3999 | IR Generation | CFG, SSA, control flow |
//! | E4001-E4999 | Code Generation | Assembly, ABI, registers |
//! | E5001-E5999 | I/O & System | File operations, CLI |
//!
//! # Usage Example
//!
//! ```rust
//! use descar_core::error::error_code::{ErrorCode, Severity, CompilerPhase};
//!
//! let code = ErrorCode::E2023;
//! println!("Error {}: {}", code.code(), code.message());
//! println!("Severity: {:?}", code.severity());
//! println!("Phase: {:?}", code.phase());
//! ```

use std::fmt;

/// The severity level of a diagnostic.
///
/// The variants are in order from the lowest level, [`Severity::Note`], to the highest level,
/// [`Severity::Fatal`]. The `Display` implementation writes the name of the level in lowercase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Severity {
    /// An informational note.
    Note = 0,
    /// A warning about a possible problem.
    Warning = 1,
    /// An error.
    Error = 2,
    /// A fatal error.
    Fatal = 3,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note => write!(f, "note"),
            Self::Warning => write!(f, "warning"),
            Self::Error => write!(f, "error"),
            Self::Fatal => write!(f, "fatal"),
        }
    }
}

/// The compiler phase that an error code belongs to.
///
/// [`ErrorCode::phase`] returns this value. The `Display` implementation writes `lexer`,
/// `parser`, `semantic`, `ir-gen`, `codegen`, or `system`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum CompilerPhase {
    /// Lexical analysis (tokenization).
    Lexer = 0,
    /// Parsing (construction of the syntax tree).
    Parser = 1,
    /// Semantic analysis (type checking and scopes).
    Semantic = 2,
    /// Generation of the intermediate representation.
    IrGeneration = 3,
    /// Generation of assembly code.
    CodeGeneration = 4,
    /// I/O and system operations.
    System = 5,
}

impl fmt::Display for CompilerPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lexer => write!(f, "lexer"),
            Self::Parser => write!(f, "parser"),
            Self::Semantic => write!(f, "semantic"),
            Self::IrGeneration => write!(f, "ir-gen"),
            Self::CodeGeneration => write!(f, "codegen"),
            Self::System => write!(f, "system"),
        }
    }
}

/// The unique code of a compiler diagnostic.
///
/// Each variant is named `E` followed by a four-digit number, for example `E0001`. The first
/// digit of the number selects the compiler phase. See the module documentation for the
/// ranges. The enum is `#[non_exhaustive]`, so new codes can be added later.
///
/// # Notes
///
/// The compiler does not report all codes. The lexer reports `E0001` to `E0010`. The parser
/// reports `E1001`, `E1002`, `E1003`, `E1004`, `E1005`, `E1008`, and `E2001`. The type checker
/// and the symbol table report `E1003`, `E2002` to `E2014`, `E2016` to `E2031`, and `E2032`.
/// No other code appears in a diagnostic. The documentation of such a code describes the
/// reserved meaning of the code. For example, the parser reports a missing closing `)`, `}`,
/// or `]` as `E1004`, not as `E1010`, `E1011`, or `E1012`.
///
/// In the examples of the variants, a statement is in the body of `main`.
///
/// # Examples
///
/// ```rust
/// use descar_core::error::error_code::ErrorCode;
///
/// let code = ErrorCode::E0001;
/// assert_eq!(code.code(), "E0001");
/// assert_eq!(code.numeric_code(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[allow(non_camel_case_types)]
pub enum ErrorCode {
    // =========================================================================
    // LEXICAL ANALYSIS ERRORS (E0001-E0999)
    // =========================================================================
    /// Error E0001: Invalid or unrecognized token
    ///
    /// The lexer found a character sequence that matches no token pattern.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = @invalid
    /// ```
    ///
    /// # Solution
    /// Remove or replace the characters that the lexer does not recognize.
    E0001,

    /// Error E0002: Malformed binary number literal
    ///
    /// A binary literal has the form `#b` followed by at least one binary digit (0 or 1).
    ///
    /// The lexer reports this error if no digit follows `#b` and if a digit other than 0 or 1 follows `#b`.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = #b   // Missing digits
    /// var y: i32 = #b2  // Invalid digit '2' in binary
    /// ```
    ///
    /// # Solution
    /// Use binary digits: `#b1010` is the decimal value 10.
    E0002,

    /// Error E0003: Malformed octal number literal
    ///
    /// An octal literal has the form `#o` followed by at least one octal digit (0-7).
    ///
    /// The lexer reports this error if no digit follows `#o` and if a digit other than 0-7 follows `#o`.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = #o   // Missing digits
    /// var y: i32 = #o8  // Invalid digit '8' in octal
    /// ```
    ///
    /// # Solution
    /// Use octal digits: `#o755` is the decimal value 493.
    E0003,

    /// Error E0004: Malformed hexadecimal number literal
    ///
    /// A hexadecimal literal has the form `#x` followed by at least one hexadecimal digit (0-9, a-f, A-F).
    ///
    /// The lexer reports this error if no digit follows `#x` and if a character that is not a hexadecimal
    /// digit follows `#x`.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = #x     // Missing digits
    /// var y: i32 = #xGH   // Invalid hexadecimal digits
    /// ```
    ///
    /// # Solution
    /// Use hexadecimal digits: `#xDEAD` is the decimal value 57005.
    E0004,

    /// Error E0005: Unterminated string literal
    ///
    /// A string literal must end with a double quote.
    ///
    /// The lexer reports this error only if the literal reaches the end of the file. If more text follows
    /// the literal, the lexer reports E0001.
    ///
    /// # Example
    /// ```text
    /// main {
    ///     var s: string = "hello world
    /// // End of file
    /// ```
    ///
    /// # Solution
    /// Close the string with `"`: `var s: string = "hello world"`
    E0005,

    /// Error E0006: Unterminated character literal
    ///
    /// A character literal must end with a single quote.
    ///
    /// The lexer reports this error only if the literal reaches the end of the file. If more text follows
    /// the literal, the lexer reports E0001.
    ///
    /// # Example
    /// ```text
    /// main {
    ///     var c: char = 'x
    /// // End of file
    /// ```
    ///
    /// # Solution
    /// Close the character with `'`: `var c: char = 'x'`
    E0006,

    /// Error E0007: Invalid escape sequence
    ///
    /// The escape sequence is not valid. The valid escape sequences are `\n`, `\r`, `\t`, `\\`, `\'`, `\"`,
    /// `\0`, and `\u{XXXX}`.
    ///
    /// # Example
    /// ```text
    /// var s: string = "hello\q"  // \q is not valid
    /// ```
    ///
    /// # Solution
    /// Use a valid escape sequence, or write the backslash as `\\`: `"hello\\q"`.
    E0007,

    /// Error E0008: Unterminated multi-line comment
    ///
    /// A multi-line comment that starts with `/*` must end with `*/`.
    ///
    /// # Example
    /// ```text
    /// /* This comment never ends
    /// main { var x: i32 = 1 }
    /// ```
    ///
    /// # Solution
    /// Close the comment with `*/`.
    E0008,

    /// Error E0009: Invalid number suffix
    ///
    /// The suffix of the numeric literal is not valid.
    ///
    /// The valid suffixes are `i8`, `i16`, `i32`, `u8`, `u16`, and `u32`. The compiler also accepts `u`
    /// (u64), `f` (f32), and `d` (f64). The suffixes are not case-sensitive. The compiler does not support
    /// the suffixes `i64`, `u64`, `f32`, and `f64`.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = 42i128  // i128 is not a supported suffix
    /// ```
    ///
    /// # Solution
    /// Use a supported suffix, or remove the suffix.
    E0009,

    /// Error E0010: Number literal overflow
    ///
    /// The value of the numeric literal is outside the range of its type.
    ///
    /// # Example
    /// ```text
    /// var x: i8 = 256i8  // The maximum value of i8 is 127
    /// ```
    ///
    /// # Solution
    /// Use a smaller value, or use a type with a larger range.
    E0010,

    // =========================================================================
    // PARSING ERRORS (E1001-E1999)
    // =========================================================================
    /// Error E1001: Maximum recursion depth exceeded
    ///
    /// The nesting of the parser calls is deeper than the limit of 1000. Deeply nested expressions cause
    /// this error.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = ((((((((((((( ... 1 ... ))))))))))))  // More than 1000 nested parentheses
    /// ```
    ///
    /// # Solution
    /// Reduce the nesting, or split the expression into smaller parts.
    E1001,

    /// Error E1002: Invalid type specification
    ///
    /// The parser expected a type and found another token.
    ///
    /// A type is one of `i8`, `i16`, `i32`, `i64`, `u8`, `u16`, `u32`, `u64`, `f32`, `f64`, `char`,
    /// `string`, `bool`, or a custom identifier. An array type adds a size in square brackets after the
    /// type name. The type `vector<T>` is also accepted.
    ///
    /// # Example
    /// ```text
    /// var x: 123 = 0  // 123 is not a type
    /// ```
    ///
    /// # Solution
    /// Use a valid type name.
    E1002,

    /// Error E1003: Invalid assignment target
    ///
    /// Only a variable or an array element can be the target of an assignment.
    ///
    /// The parser and the type checker report this error.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = 1i32
    /// 5 = x       // A literal is not a valid target
    /// foo() = 1   // A function call is not a valid target
    /// ```
    ///
    /// # Solution
    /// Assign to a variable or to an array element: `x = 5` or `arr[0] = 1`.
    E1003,

    /// Error E1004: Unexpected token
    ///
    /// The parser found a token that is not valid at this position. The parser reports this error also for
    /// a missing closing `)`, `}`, or `]`, and for a `;`.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = (1 + 2  // The closing ')' is missing
    /// ```
    ///
    /// # Solution
    /// Correct the syntax, and add the missing tokens.
    E1004,

    /// Error E1005: Invalid binary operator
    ///
    /// The token is not a binary operator. The parser reports this error also for a token that is not an
    /// assignment operator.
    ///
    /// # Solution
    /// Use a binary operator: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `<=`, `>`, `>=`, `&&`, `||`, `&`,
    /// `|`, `^`, `<<`, or `>>`.
    E1005,

    /// Error E1006: Expected expression
    ///
    /// The parser expected an expression and found no expression.
    ///
    /// # Solution
    /// Provide an expression.
    E1006,

    /// Error E1007: Expected statement
    ///
    /// The parser expected a statement and found no statement.
    E1007,

    /// Error E1008: Expected identifier
    ///
    /// The parser expected an identifier, for example a variable name or a function name.
    ///
    /// # Example
    /// ```text
    /// var 123: i32 = 0  // 123 is not an identifier
    /// ```
    ///
    /// # Solution
    /// Use a valid identifier.
    E1008,

    /// Error E1009: Expected type annotation
    ///
    /// The parser expected a type after `:`.
    ///
    /// # Solution
    /// Provide a type: `var x: i32 = 0`.
    E1009,

    /// Error E1010: Unmatched parenthesis
    ///
    /// An opening `(` has no closing `)`, or a closing `)` has no opening `(`.
    ///
    /// # Solution
    /// Balance the parentheses.
    E1010,

    /// Error E1011: Unmatched brace
    ///
    /// An opening `{` has no closing `}`, or a closing `}` has no opening `{`.
    ///
    /// # Solution
    /// Balance the braces.
    E1011,

    /// Error E1012: Unmatched bracket
    ///
    /// An opening `[` has no closing `]`, or a closing `]` has no opening `[`.
    ///
    /// # Solution
    /// Balance the brackets.
    E1012,

    /// Error E1013: Missing semicolon (informational)
    ///
    /// A semicolon is missing. The severity of this code is [`Severity::Warning`].
    E1013,

    /// Error E1014: Invalid function signature
    ///
    /// The structure of the function declaration is not valid.
    E1014,

    /// Error E1015: Invalid parameter list
    ///
    /// The parameter list of the function is not valid.
    E1015,

    // =========================================================================
    // SEMANTIC / TYPE ERRORS (E2001-E2999)
    // =========================================================================
    /// Error E2001: Initializer count mismatch
    ///
    /// The number of initializers is not equal to the number of variables. The parser reports this error.
    ///
    /// # Example
    /// ```text
    /// var a, b: i32 = 1i32  // Two variables and one initializer
    /// ```
    ///
    /// # Solution
    /// Provide one initializer for each variable: `var a, b: i32 = 1i32, 2i32`.
    E2001,

    /// Error E2002: Type mismatch in assignment
    ///
    /// The type of the value is not compatible with the type of the variable.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = "hello"  // A string is not assignable to i32
    /// ```
    ///
    /// # Solution
    /// Use a value of a compatible type.
    E2002,

    /// Error E2003: Missing return path
    ///
    /// A function with a return type has a code path that does not return a value.
    ///
    /// # Example
    /// ```text
    /// fun foo(): i32 {
    ///     if (true) { return 1i32 }
    ///     // The code path of a false condition has no return
    /// }
    /// ```
    ///
    /// # Solution
    /// Make all code paths return a value.
    E2003,

    /// Error E2004: Non-boolean condition
    ///
    /// The condition of a statement must have the type `bool`.
    ///
    /// # Example
    /// ```text
    /// if (42) { }  // 42 has the type i64
    /// ```
    ///
    /// # Solution
    /// Use a boolean expression: `if (x > 0) { }`.
    E2004,

    /// Error E2005: Return outside function
    ///
    /// A `return` statement is outside of a function body.
    ///
    /// # Solution
    /// Move the `return` statement into a function.
    E2005,

    /// Error E2006: Return value in void function
    ///
    /// A function without a return type cannot return a value.
    ///
    /// # Example
    /// ```text
    /// fun foo() {
    ///     return 42  // The function has no return type
    /// }
    /// ```
    ///
    /// # Solution
    /// Remove the return value, or add a return type to the function.
    E2006,

    /// Error E2007: Return type mismatch
    ///
    /// The type of the returned value is not the declared return type of the function.
    ///
    /// # Example
    /// ```text
    /// fun foo(): i32 {
    ///     return "hello"  // string is not i32
    /// }
    /// ```
    ///
    /// # Solution
    /// Return a value of the declared type.
    E2007,

    /// Error E2008: Missing return value
    ///
    /// A function with a return type uses `return` without a value.
    ///
    /// # Example
    /// ```text
    /// fun foo(): i32 {
    ///     return  // No value
    /// }
    /// ```
    ///
    /// # Solution
    /// Return a value: `return 0i32`.
    E2008,

    /// Error E2009: Break outside loop
    ///
    /// A `break` statement is outside of a loop.
    ///
    /// # Solution
    /// Use `break` only in a `while` loop or in a `for` loop.
    E2009,

    /// Error E2010: Continue outside loop
    ///
    /// A `continue` statement is outside of a loop.
    ///
    /// # Solution
    /// Use `continue` only in a `while` loop or in a `for` loop.
    E2010,

    /// Error E2011: Bitwise operator on non-integer
    ///
    /// The bitwise operators (`&`, `|`, `^`, `<<`, `>>`) need integer operands.
    ///
    /// # Example
    /// ```text
    /// var x: f64 = 1.5 & 2.5  // f64 is not an integer type
    /// ```
    ///
    /// # Solution
    /// Use integer operands.
    E2011,

    /// Error E2012: Logical operator on non-boolean
    ///
    /// The logical operators (`&&`, `||`) need operands of the type `bool`.
    ///
    /// # Example
    /// ```text
    /// var x: bool = true && 1  // 1 is not a bool
    /// ```
    ///
    /// # Solution
    /// Use boolean expressions: `x > 0 && y > 0`.
    E2012,

    /// Error E2013: Arithmetic operator on non-numeric
    ///
    /// The arithmetic operators (`+`, `-`, `*`, `/`, `%`) need numeric operands.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = 1i32 + "a"  // A string is not a number
    /// ```
    ///
    /// # Solution
    /// Use numeric operands.
    E2013,

    /// Error E2014: Incompatible comparison types
    ///
    /// The operands of a comparison operator have types that are not compatible.
    ///
    /// # Example
    /// ```text
    /// var x: bool = 1 < "hello"  // i64 and string
    /// ```
    ///
    /// # Solution
    /// Compare values of compatible types.
    E2014,

    /// Error E2015: Binary operation type mismatch
    ///
    /// The operand types of a binary expression are not compatible.
    ///
    /// # Solution
    /// Use operands with compatible types.
    E2015,

    /// Error E2016: Unsupported arithmetic operation
    ///
    /// The type of the operands does not support the arithmetic operation.
    ///
    /// # Example
    /// ```text
    /// var x: string = "a" + "b"  // The type string does not support +
    /// ```
    ///
    /// # Solution
    /// Use operands of a numeric type.
    E2016,

    /// Error E2017: Logical operation requires bool
    ///
    /// A logical operation needs operands of the type `bool`.
    ///
    /// # Example
    /// ```text
    /// var x: bool = 1 && 2  // 1 and 2 are not bool values
    /// ```
    ///
    /// # Solution
    /// Use boolean operands.
    E2017,

    /// Error E2018: Negation on non-numeric type
    ///
    /// The unary operator `-` needs a numeric operand.
    ///
    /// # Example
    /// ```text
    /// var x: bool = -true  // A bool cannot be negated
    /// ```
    ///
    /// # Solution
    /// Use a numeric operand.
    E2018,

    /// Error E2019: Logical NOT on non-boolean
    ///
    /// The unary operator `!` needs an operand of the type `bool`.
    ///
    /// # Example
    /// ```text
    /// var x: bool = !42  // 42 is not a bool
    /// ```
    ///
    /// # Solution
    /// Use a boolean operand: `!flag`.
    E2019,

    /// Error E2020: Empty array literal
    ///
    /// An array literal needs at least one element. The type checker uses the elements to find the type of
    /// the array.
    ///
    /// # Example
    /// ```text
    /// var arr: i32[1] = {}  // No elements
    /// ```
    ///
    /// # Solution
    /// Provide at least one element: `{0i32}`.
    E2020,

    /// Error E2021: Mixed array element types
    ///
    /// All elements of an array literal must have the same type.
    ///
    /// # Example
    /// ```text
    /// var arr: i32[3] = {1i32, "hello", 3i32}  // i32 and string
    /// ```
    ///
    /// # Solution
    /// Use elements of the same type.
    E2021,

    /// Error E2022: Function used as variable
    ///
    /// A function name is in a place where the program needs a variable.
    ///
    /// # Example
    /// ```text
    /// fun foo() { }
    /// var x: i32 = foo  // foo is a function
    /// ```
    ///
    /// # Solution
    /// Call the function: `foo()`.
    E2022,

    /// Error E2023: Undefined variable
    ///
    /// The variable is not declared in the current scope or in an enclosing scope.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = y  // y is not declared
    /// ```
    ///
    /// # Solution
    /// Declare the variable before its use.
    E2023,

    /// Error E2024: Assignment to immutable variable
    ///
    /// A variable that `const` declares cannot receive a new value.
    ///
    /// # Example
    /// ```text
    /// const x: i32 = 1i32
    /// x = 2i32  // x is a constant
    /// ```
    ///
    /// # Solution
    /// Declare the variable with `var`, or remove the assignment.
    E2024,

    /// Error E2025: Undefined variable in assignment
    ///
    /// The target of an assignment is not a declared variable.
    ///
    /// # Example
    /// ```text
    /// y = 5i32  // y is not declared
    /// ```
    ///
    /// # Solution
    /// Declare the variable before the assignment.
    E2025,

    /// Error E2026: Invalid callee
    ///
    /// The expression of a call is not a function name.
    ///
    /// # Example
    /// ```text
    /// var arr: i32[1] = {1i32}
    /// arr[0]()  // The callee is not a name
    /// ```
    ///
    /// # Solution
    /// Call a function by its name.
    E2026,

    /// Error E2027: Undefined function
    ///
    /// The program calls a name that is not a declared function. A call of a variable also causes this
    /// error.
    ///
    /// # Example
    /// ```text
    /// foo()  // foo is not declared
    /// ```
    ///
    /// # Solution
    /// Declare the function before the call.
    E2027,

    /// Error E2028: Argument count mismatch
    ///
    /// The number of arguments is not equal to the number of parameters of the function.
    ///
    /// # Example
    /// ```text
    /// fun add(a: i32, b: i32): i32 { return a + b }
    /// var x: i32 = add(1i32)  // The function needs 2 arguments
    /// ```
    ///
    /// # Solution
    /// Provide the correct number of arguments.
    E2028,

    /// Error E2029: Argument type mismatch
    ///
    /// The type of an argument is not the type of the parameter.
    ///
    /// # Example
    /// ```text
    /// fun foo(x: i32) { }
    /// foo("hello")  // string is not i32
    /// ```
    ///
    /// # Solution
    /// Provide an argument of the correct type.
    E2029,

    /// Error E2030: Non-integer array index
    ///
    /// The index of an array element must have an integer type.
    ///
    /// # Example
    /// ```text
    /// var arr: i32[5] = {1i32, 2i32, 3i32, 4i32, 5i32}
    /// var x: i32 = arr[1.5]  // f64 is not an integer type
    /// ```
    ///
    /// # Solution
    /// Use an integer index.
    E2030,

    /// Error E2031: Indexing non-array type
    ///
    /// Only an array can have an index.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = 42i32
    /// var y: i32 = x[0]  // i32 is not an array
    /// ```
    ///
    /// # Solution
    /// Use an index only on an array.
    E2031,

    /// Error E2032: Duplicate declaration
    ///
    /// The current scope already has an identifier with this name.
    ///
    /// # Example
    /// ```text
    /// var x: i32 = 1i32
    /// var x: i32 = 2i32  // x is already declared
    /// ```
    ///
    /// # Solution
    /// Use a different name, or remove the duplicate declaration.
    E2032,

    // =========================================================================
    // IR GENERATION ERRORS (E3001-E3999)
    // =========================================================================
    /// Error E3001: Break outside loop in IR generation
    ///
    /// A `break` is outside of a loop during IR generation. This is an internal error.
    E3001,

    /// Error E3002: Continue outside loop in IR generation
    ///
    /// A `continue` is outside of a loop during IR generation. This is an internal error.
    E3002,

    /// Error E3003: Invalid IR instruction
    ///
    /// The IR generator created an invalid instruction.
    E3003,

    /// Error E3004: Undefined IR variable
    ///
    /// A variable is not defined during IR generation.
    E3004,

    /// Error E3005: Invalid basic block
    ///
    /// A basic block in the control flow graph is not valid.
    E3005,

    /// Error E3006: Invalid terminator
    ///
    /// A basic block has an invalid terminator or no terminator.
    E3006,

    /// Error E3007: SSA transformation error
    ///
    /// An error occurred in the transformation to Static Single Assignment form.
    E3007,

    /// Error E3008: CFG construction error
    ///
    /// An error occurred in the construction of the control flow graph.
    E3008,

    // =========================================================================
    // CODE GENERATION ERRORS (E4001-E4999)
    // =========================================================================
    /// Error E4001: Invalid assembly instruction
    ///
    /// The code generator cannot create valid assembly for the operation.
    E4001,

    /// Error E4002: Register allocation failed
    ///
    /// The code generator cannot allocate registers for the operation.
    E4002,

    /// Error E4003: Stack overflow
    ///
    /// The stack frame is larger than the maximum size.
    E4003,

    /// Error E4004: Unsupported platform
    ///
    /// The target platform is not supported.
    E4004,

    /// Error E4005: ABI violation
    ///
    /// The generated code does not follow the ABI of the target.
    E4005,

    // =========================================================================
    // I/O AND SYSTEM ERRORS (E5001-E5999)
    // =========================================================================
    /// Error E5001: File not found
    ///
    /// The compiler cannot find the specified source file.
    E5001,

    /// Error E5002: Permission denied
    ///
    /// The permissions are not sufficient to access the file.
    E5002,

    /// Error E5003: Invalid file extension
    ///
    /// The extension of a source file must be `.dr`.
    E5003,

    /// Error E5004: Write error
    ///
    /// The compiler cannot write an output file.
    E5004,

    /// Error E5005: Read error
    ///
    /// The compiler cannot read an input file.
    E5005,
}

impl ErrorCode {
    /// Returns the code as text, for example `"E0001"`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::E0001.code(), "E0001");
    /// assert_eq!(ErrorCode::E2023.code(), "E2023");
    /// ```
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            // Lexical errors
            Self::E0001 => "E0001",
            Self::E0002 => "E0002",
            Self::E0003 => "E0003",
            Self::E0004 => "E0004",
            Self::E0005 => "E0005",
            Self::E0006 => "E0006",
            Self::E0007 => "E0007",
            Self::E0008 => "E0008",
            Self::E0009 => "E0009",
            Self::E0010 => "E0010",

            // Parser errors
            Self::E1001 => "E1001",
            Self::E1002 => "E1002",
            Self::E1003 => "E1003",
            Self::E1004 => "E1004",
            Self::E1005 => "E1005",
            Self::E1006 => "E1006",
            Self::E1007 => "E1007",
            Self::E1008 => "E1008",
            Self::E1009 => "E1009",
            Self::E1010 => "E1010",
            Self::E1011 => "E1011",
            Self::E1012 => "E1012",
            Self::E1013 => "E1013",
            Self::E1014 => "E1014",
            Self::E1015 => "E1015",

            // Type errors
            Self::E2001 => "E2001",
            Self::E2002 => "E2002",
            Self::E2003 => "E2003",
            Self::E2004 => "E2004",
            Self::E2005 => "E2005",
            Self::E2006 => "E2006",
            Self::E2007 => "E2007",
            Self::E2008 => "E2008",
            Self::E2009 => "E2009",
            Self::E2010 => "E2010",
            Self::E2011 => "E2011",
            Self::E2012 => "E2012",
            Self::E2013 => "E2013",
            Self::E2014 => "E2014",
            Self::E2015 => "E2015",
            Self::E2016 => "E2016",
            Self::E2017 => "E2017",
            Self::E2018 => "E2018",
            Self::E2019 => "E2019",
            Self::E2020 => "E2020",
            Self::E2021 => "E2021",
            Self::E2022 => "E2022",
            Self::E2023 => "E2023",
            Self::E2024 => "E2024",
            Self::E2025 => "E2025",
            Self::E2026 => "E2026",
            Self::E2027 => "E2027",
            Self::E2028 => "E2028",
            Self::E2029 => "E2029",
            Self::E2030 => "E2030",
            Self::E2031 => "E2031",
            Self::E2032 => "E2032",

            // IR errors
            Self::E3001 => "E3001",
            Self::E3002 => "E3002",
            Self::E3003 => "E3003",
            Self::E3004 => "E3004",
            Self::E3005 => "E3005",
            Self::E3006 => "E3006",
            Self::E3007 => "E3007",
            Self::E3008 => "E3008",

            // Code gen errors
            Self::E4001 => "E4001",
            Self::E4002 => "E4002",
            Self::E4003 => "E4003",
            Self::E4004 => "E4004",
            Self::E4005 => "E4005",

            // I/O errors
            Self::E5001 => "E5001",
            Self::E5002 => "E5002",
            Self::E5003 => "E5003",
            Self::E5004 => "E5004",
            Self::E5005 => "E5005",
        }
    }

    /// Returns the number of the code, without the letter `E`.
    ///
    /// For example, the result for `E0001` is 1 and the result for `E2023` is 2023.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::ErrorCode;
    ///
    /// assert_eq!(ErrorCode::E0001.numeric_code(), 1);
    /// assert_eq!(ErrorCode::E2023.numeric_code(), 2023);
    /// ```
    #[must_use]
    pub const fn numeric_code(&self) -> u16 {
        match self {
            // Lexical errors (0001-0999)
            Self::E0001 => 1,
            Self::E0002 => 2,
            Self::E0003 => 3,
            Self::E0004 => 4,
            Self::E0005 => 5,
            Self::E0006 => 6,
            Self::E0007 => 7,
            Self::E0008 => 8,
            Self::E0009 => 9,
            Self::E0010 => 10,

            // Parser errors (1001-1999)
            Self::E1001 => 1001,
            Self::E1002 => 1002,
            Self::E1003 => 1003,
            Self::E1004 => 1004,
            Self::E1005 => 1005,
            Self::E1006 => 1006,
            Self::E1007 => 1007,
            Self::E1008 => 1008,
            Self::E1009 => 1009,
            Self::E1010 => 1010,
            Self::E1011 => 1011,
            Self::E1012 => 1012,
            Self::E1013 => 1013,
            Self::E1014 => 1014,
            Self::E1015 => 1015,

            // Type errors (2001-2999)
            Self::E2001 => 2001,
            Self::E2002 => 2002,
            Self::E2003 => 2003,
            Self::E2004 => 2004,
            Self::E2005 => 2005,
            Self::E2006 => 2006,
            Self::E2007 => 2007,
            Self::E2008 => 2008,
            Self::E2009 => 2009,
            Self::E2010 => 2010,
            Self::E2011 => 2011,
            Self::E2012 => 2012,
            Self::E2013 => 2013,
            Self::E2014 => 2014,
            Self::E2015 => 2015,
            Self::E2016 => 2016,
            Self::E2017 => 2017,
            Self::E2018 => 2018,
            Self::E2019 => 2019,
            Self::E2020 => 2020,
            Self::E2021 => 2021,
            Self::E2022 => 2022,
            Self::E2023 => 2023,
            Self::E2024 => 2024,
            Self::E2025 => 2025,
            Self::E2026 => 2026,
            Self::E2027 => 2027,
            Self::E2028 => 2028,
            Self::E2029 => 2029,
            Self::E2030 => 2030,
            Self::E2031 => 2031,
            Self::E2032 => 2032,

            // IR errors (3001-3999)
            Self::E3001 => 3001,
            Self::E3002 => 3002,
            Self::E3003 => 3003,
            Self::E3004 => 3004,
            Self::E3005 => 3005,
            Self::E3006 => 3006,
            Self::E3007 => 3007,
            Self::E3008 => 3008,

            // Code gen errors (4001-4999)
            Self::E4001 => 4001,
            Self::E4002 => 4002,
            Self::E4003 => 4003,
            Self::E4004 => 4004,
            Self::E4005 => 4005,

            // I/O errors (5001-5999)
            Self::E5001 => 5001,
            Self::E5002 => 5002,
            Self::E5003 => 5003,
            Self::E5004 => 5004,
            Self::E5005 => 5005,
        }
    }

    /// Returns the severity of the code.
    ///
    /// The result is [`Severity::Warning`] for `E1013`. The result for all other codes is
    /// [`Severity::Error`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::{ErrorCode, Severity};
    ///
    /// assert_eq!(ErrorCode::E0001.severity(), Severity::Error);
    /// assert_eq!(ErrorCode::E1013.severity(), Severity::Warning);
    /// ```
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            // Warnings
            Self::E1013 => Severity::Warning,

            // All others are errors
            _ => Severity::Error,
        }
    }

    /// Returns the compiler phase of the code.
    ///
    /// The phase comes from the range of the number of the code. See the module documentation
    /// for the ranges.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::{ErrorCode, CompilerPhase};
    ///
    /// assert_eq!(ErrorCode::E0001.phase(), CompilerPhase::Lexer);
    /// assert_eq!(ErrorCode::E1001.phase(), CompilerPhase::Parser);
    /// assert_eq!(ErrorCode::E2023.phase(), CompilerPhase::Semantic);
    /// ```
    #[must_use]
    pub const fn phase(&self) -> CompilerPhase {
        match self.numeric_code() {
            1..=999 => CompilerPhase::Lexer,
            1001..=1999 => CompilerPhase::Parser,
            2001..=2999 => CompilerPhase::Semantic,
            3001..=3999 => CompilerPhase::IrGeneration,
            4001..=4999 => CompilerPhase::CodeGeneration,
            // 5001..=5999 and any unknown codes default to System
            _ => CompilerPhase::System,
        }
    }

    /// Returns a short message for the code.
    ///
    /// The message starts with a lowercase letter. For a longer text, use
    /// [`explanation()`](Self::explanation).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::ErrorCode;
    ///
    /// assert_eq!(
    ///     ErrorCode::E0001.message(),
    ///     "invalid or unrecognized token"
    /// );
    /// ```
    #[must_use]
    pub const fn message(&self) -> &'static str {
        match self {
            // Lexical errors
            Self::E0001 => "invalid or unrecognized token",
            Self::E0002 => "malformed binary number literal",
            Self::E0003 => "malformed octal number literal",
            Self::E0004 => "malformed hexadecimal number literal",
            Self::E0005 => "unterminated string literal",
            Self::E0006 => "unterminated character literal",
            Self::E0007 => "invalid escape sequence",
            Self::E0008 => "unterminated multi-line comment",
            Self::E0009 => "invalid number suffix",
            Self::E0010 => "number literal overflow",

            // Parser errors
            Self::E1001 => "maximum recursion depth exceeded",
            Self::E1002 => "invalid type specification",
            Self::E1003 => "invalid assignment target",
            Self::E1004 => "unexpected token",
            Self::E1005 => "invalid binary operator",
            Self::E1006 => "expected expression",
            Self::E1007 => "expected statement",
            Self::E1008 => "expected identifier",
            Self::E1009 => "expected type annotation",
            Self::E1010 => "unmatched parenthesis",
            Self::E1011 => "unmatched brace",
            Self::E1012 => "unmatched bracket",
            Self::E1013 => "missing semicolon",
            Self::E1014 => "invalid function signature",
            Self::E1015 => "invalid parameter list",

            // Type errors
            Self::E2001 => "initializer count mismatch",
            Self::E2002 => "type mismatch in assignment",
            Self::E2003 => "missing return in some code paths",
            Self::E2004 => "condition must be boolean",
            Self::E2005 => "return outside function",
            Self::E2006 => "cannot return value from void function",
            Self::E2007 => "return type mismatch",
            Self::E2008 => "missing return value",
            Self::E2009 => "break outside loop",
            Self::E2010 => "continue outside loop",
            Self::E2011 => "bitwise operator requires integer operands",
            Self::E2012 => "logical operator requires boolean operands",
            Self::E2013 => "arithmetic operator requires numeric operands",
            Self::E2014 => "incompatible types in comparison",
            Self::E2015 => "type mismatch in binary operation",
            Self::E2016 => "unsupported arithmetic operation",
            Self::E2017 => "logical operation requires boolean",
            Self::E2018 => "negation requires numeric type",
            Self::E2019 => "logical NOT requires boolean type",
            Self::E2020 => "empty array literal",
            Self::E2021 => "mixed types in array literal",
            Self::E2022 => "function cannot be used as variable",
            Self::E2023 => "undefined variable",
            Self::E2024 => "cannot assign to immutable variable",
            Self::E2025 => "undefined variable in assignment",
            Self::E2026 => "callee must be a function",
            Self::E2027 => "undefined function",
            Self::E2028 => "wrong number of arguments",
            Self::E2029 => "argument type mismatch",
            Self::E2030 => "array index must be integer",
            Self::E2031 => "cannot index non-array type",
            Self::E2032 => "duplicate declaration",

            // IR errors
            Self::E3001 => "break outside loop in IR",
            Self::E3002 => "continue outside loop in IR",
            Self::E3003 => "invalid IR instruction",
            Self::E3004 => "undefined variable in IR",
            Self::E3005 => "invalid basic block",
            Self::E3006 => "invalid block terminator",
            Self::E3007 => "SSA transformation error",
            Self::E3008 => "CFG construction error",

            // Code gen errors
            Self::E4001 => "invalid assembly instruction",
            Self::E4002 => "register allocation failed",
            Self::E4003 => "stack frame overflow",
            Self::E4004 => "unsupported target platform",
            Self::E4005 => "ABI violation",

            // I/O errors
            Self::E5001 => "file not found",
            Self::E5002 => "permission denied",
            Self::E5003 => "invalid file extension",
            Self::E5004 => "write error",
            Self::E5005 => "read error",
        }
    }

    /// Returns an explanation of the code.
    ///
    /// The text can have more than one line. Only some codes have a specific text. For all
    /// other codes, the result is `"See the error message for details."`.
    ///
    /// # Notes
    ///
    /// The text for `E0009` lists the suffixes `i64`, `u64`, `f32`, and `f64`. The lexer does
    /// not accept these suffixes. See the documentation of [`ErrorCode::E0009`] for the
    /// supported suffixes. The text for `E0005` says that the closing quote must be before the
    /// end of the line. The lexer reports `E0005` only if the literal reaches the end of the
    /// file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::ErrorCode;
    ///
    /// let explanation = ErrorCode::E2023.explanation();
    /// assert!(explanation.contains("declare"));
    /// ```
    #[must_use]
    pub const fn explanation(&self) -> &'static str {
        match self {
            Self::E0001 => {
                "The lexer encountered a character sequence that doesn't match any valid token pattern.\n\
                Check for typos or unsupported characters. Valid tokens include identifiers,\n\
                keywords, operators, and literals."
            }
            Self::E0002 => {
                "Binary literals must have at least one binary digit (0 or 1) after `#b`.\n\
                Example: `#b1010` for decimal 10."
            }
            Self::E0003 => {
                "Octal literals must have at least one octal digit (0-7) after `#o`.\n\
                Example: `#o755` for decimal 493."
            }
            Self::E0004 => {
                "Hexadecimal literals must have at least one hex digit (0-9, a-f, A-F) after `#x`.\n\
                Example: `#xDEAD` for decimal 57005."
            }
            Self::E0005 => {
                "String literals must be closed with a matching double quote before the end of the line.\n\
                Example: `\"hello world\"` instead of `\"hello world`."
            }
            Self::E0006 => {
                "Character literals must be closed with a matching single quote.\n\
                Example: `'x'` instead of `'x`."
            }
            Self::E0007 => {
                "The escape sequence is not recognized. Valid escape sequences include:\n\
                \\n (newline), \\r (carriage return), \\t (tab), \\\\ (backslash),\n\
                \\' (single quote), \\\" (double quote), \\0 (null), \\u{XXXX} (unicode)."
            }
            Self::E0008 => {
                "Multi-line comments opened with `/*` must be closed with `*/`.\n\
                Check for missing closing markers or accidental nesting."
            }
            Self::E0009 => {
                "The suffix on the numeric literal is not recognized.\n\
                Valid suffixes: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64."
            }
            Self::E0010 => {
                "The numeric value exceeds the range of the target type.\n\
                Use a larger type or reduce the value."
            }
            Self::E1001 => {
                "The parser has exceeded its recursion limit due to deeply nested expressions.\n\
                Simplify the expression or break it into smaller parts."
            }
            Self::E1002 => {
                "Expected a valid type but found something else.\n\
                Valid types: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, char, string, bool,\n\
                or custom type identifiers."
            }
            Self::E1003 => {
                "Only variables and array elements can be assigned to.\n\
                Examples: `x = 5` or `arr[0] = 1`."
            }
            Self::E2023 => {
                "The variable has not been declared in the current scope or any outer scope.\n\
                Declare the variable with `var` or `const` before using it."
            }
            Self::E2024 => {
                "Variables declared with `const` cannot be reassigned.\n\
                Use `var` for mutable variables or remove the reassignment."
            }
            Self::E2027 => {
                "The function has not been declared.\n\
                Define the function with `fun` before calling it."
            }
            Self::E2028 => {
                "The number of arguments provided doesn't match the function's parameter count.\n\
                Check the function definition and provide the correct number of arguments."
            }
            // Default explanation for other errors
            _ => "See the error message for details.",
        }
    }

    /// Returns suggestions for the correction of the error.
    ///
    /// Only `E0002`, `E0003`, `E0004`, `E0005`, `E2009`, `E2010`, `E2023`, and `E2024` have
    /// suggestions. For all other codes, the result is an empty slice.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use descar_core::error::error_code::ErrorCode;
    ///
    /// assert!(!ErrorCode::E2023.suggestions().is_empty());
    /// assert!(ErrorCode::E0001.suggestions().is_empty());
    /// ```
    #[must_use]
    pub const fn suggestions(&self) -> &'static [&'static str] {
        match self {
            Self::E0002 => &["Add binary digits after #b: #b1010", "Check for invalid digits (only 0 and 1 allowed)"],
            Self::E0003 => &["Add octal digits after #o: #o755", "Check for invalid digits (only 0-7 allowed)"],
            Self::E0004 => {
                &["Add hexadecimal digits after #x: #xDEAD", "Check for invalid digits (only 0-9, a-f, A-F allowed)"]
            }
            Self::E0005 => &[
                "Add a closing double quote: \"hello\"",
                "Use escape sequence for embedded quotes: \"say \\\"hello\\\"\"",
            ],
            Self::E2023 => &[
                "Declare the variable: var x: i32 = 0",
                "Check for typos in the variable name",
                "Ensure the variable is in scope",
            ],
            Self::E2024 => &["Use 'var' instead of 'const' for mutable variables", "Remove the reassignment"],
            Self::E2009 | Self::E2010 => {
                &["Move the statement inside a while or for loop", "Use return to exit a function instead"]
            }
            _ => &[],
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code(), self.message())
    }
}

impl std::error::Error for ErrorCode {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_format() {
        assert_eq!(ErrorCode::E0001.code(), "E0001");
        assert_eq!(ErrorCode::E1001.code(), "E1001");
        assert_eq!(ErrorCode::E2023.code(), "E2023");
        assert_eq!(ErrorCode::E3001.code(), "E3001");
        assert_eq!(ErrorCode::E4001.code(), "E4001");
        assert_eq!(ErrorCode::E5001.code(), "E5001");
    }

    #[test]
    fn test_numeric_code() {
        assert_eq!(ErrorCode::E0001.numeric_code(), 1);
        assert_eq!(ErrorCode::E2023.numeric_code(), 2023);
    }

    #[test]
    fn test_phase_detection() {
        assert_eq!(ErrorCode::E0001.phase(), CompilerPhase::Lexer);
        assert_eq!(ErrorCode::E1001.phase(), CompilerPhase::Parser);
        assert_eq!(ErrorCode::E2023.phase(), CompilerPhase::Semantic);
        assert_eq!(ErrorCode::E3001.phase(), CompilerPhase::IrGeneration);
        assert_eq!(ErrorCode::E4001.phase(), CompilerPhase::CodeGeneration);
        assert_eq!(ErrorCode::E5001.phase(), CompilerPhase::System);
    }

    #[test]
    fn test_severity() {
        assert_eq!(ErrorCode::E0001.severity(), Severity::Error);
        assert_eq!(ErrorCode::E1013.severity(), Severity::Warning);
    }

    #[test]
    fn test_display() {
        let code = ErrorCode::E2023;
        let display = format!("{code}");
        assert!(display.contains("E2023"));
        assert!(display.contains("undefined variable"));
    }

    #[test]
    fn test_suggestions_not_empty() {
        let suggestions: &[&str] = ErrorCode::E2023.suggestions();
        assert_ne!(suggestions, [] as [&str; 0]);
    }

    #[test]
    fn test_explanation_not_empty() {
        let explanation = ErrorCode::E2023.explanation();
        assert_ne!(explanation, "");
        assert!(explanation.contains("declare"));
    }
}
