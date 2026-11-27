// crates/descar-core/src/syntax/parser.rs
use crate::error::compile_error::CompileError;
use crate::error::error_code::ErrorCode;
use crate::location::source_span::SourceSpan;
use crate::syntax::ast::assignment_op::AssignmentOp;
use crate::syntax::ast::binary_op::BinaryOp;
use crate::syntax::ast::compilation_unit::CompilationUnit;
use crate::syntax::ast::else_branch::ElseBranch;
use crate::syntax::ast::expr::Expr;
use crate::syntax::ast::parameter::Parameter;
use crate::syntax::ast::stmt::{Stmt, VarBinding};
use crate::syntax::ast::type_syntax::TypeSyntax;
use crate::syntax::ast::unary_op::UnaryOp;
use crate::syntax::ast::unary_op_side::UnaryOpSide;
use crate::syntax::precendence::Precedence;
use crate::tokens::token::Token;
use crate::tokens::token_kind::TokenKind;
use std::sync::Arc;
use tracing::warn;

/// Parser that converts a slice of tokens into a compilation unit.
///
/// The parser is a recursive descent parser. It parses expressions with binding powers (see
/// [`Precedence`]). The parser does not stop at the first error. It stores each error in a list
/// and continues, so one run can report more than one error.
pub struct JsavParser<'a> {
    /// The tokens to parse.
    tokens: &'a [Token],
    /// The index of the next token to read.
    current: usize,
    /// The errors that the parser has found.
    errors: Vec<CompileError>,
    /// The number of active calls of `parse_expr`.
    recursion_depth: usize,
    /// Whether the parser has accepted a `main` declaration.
    main_declared: bool,
}

impl<'a> JsavParser<'a> {
    /// The parser reports error `E1001` when the recursion depth is greater than this value.
    const MAX_RECURSION_DEPTH: usize = 1000;

    /// Checks the recursion depth of the expression parser.
    ///
    /// If the depth is greater than [`Self::MAX_RECURSION_DEPTH`], the function logs a warning. If a
    /// token is available, it also stores a syntax error with code `E1001` at the current token.
    ///
    /// # Returns
    ///
    /// `true` if the depth is greater than the limit. Otherwise, `false`.
    fn check_recursion_limit(&mut self) -> bool {
        if self.recursion_depth > Self::MAX_RECURSION_DEPTH {
            if let Some(token) = self.peek() {
                warn!(depth = self.recursion_depth, "parser recursion limit exceeded");
                self.errors.push(CompileError::SyntaxError {
                    code: Some(ErrorCode::E1001),
                    message: "Maximum recursion depth exceeded".into(),
                    span: token.span.clone(),
                    help: Some("Simplify the expression or break it into smaller parts".to_string()),
                });
            }
            true
        } else {
            false
        }
    }

    /// Increases the recursion depth by one.
    const fn enter_recursion(&mut self) {
        self.recursion_depth += 1;
    }

    /// Decreases the recursion depth by one. The depth does not go below zero.
    const fn exit_recursion(&mut self) {
        if self.recursion_depth > 0 {
            self.recursion_depth -= 1;
        }
    }

    /// Creates a parser for a slice of tokens.
    ///
    /// The parser starts at the first token of the slice.
    ///
    /// # Arguments
    ///
    /// * `tokens` - The tokens to parse.
    #[must_use]
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0, errors: Vec::with_capacity(8), recursion_depth: 0, main_declared: false }
    }

    /// Parses all tokens into a compilation unit.
    ///
    /// The parser accepts only these declarations at the top level: global variables (`var` and
    /// `const`), functions (`fun`), and one `main` block. For any other token, the parser stores
    /// error `E1004`. It still parses the token as a statement and keeps the statement in the
    /// unit, so that later analysis can report more errors. A second `main` declaration causes
    /// error `E1004`, and the parser does not keep it.
    ///
    /// If the parser makes no progress for a declaration, it skips one token.
    ///
    /// # Returns
    ///
    /// A tuple of the compilation unit and the list of errors. The function always returns a
    /// unit, also when the list has errors. The span of the unit goes from the first to the last
    /// declaration. The span is the default span if the unit has no declaration.
    #[must_use]
    pub fn parse(mut self) -> (CompilationUnit, Vec<CompileError>) {
        let mut declarations = Vec::with_capacity(self.tokens.len() / 4);
        while !self.is_at_end() {
            let Some(token) = self.peek().cloned() else { break };
            let before = self.current;
            let declaration = match token.kind {
                TokenKind::KeywordFun => self.parse_function(),
                TokenKind::KeywordMain => {
                    let parsed = self.parse_main_function();
                    if self.main_declared {
                        self.syntax_error(
                            "A compilation unit can contain only one main declaration",
                            &token,
                            Some("Remove the duplicate main declaration"),
                            Some(ErrorCode::E1004),
                        );
                        None
                    } else {
                        self.main_declared = parsed.is_some();
                        parsed
                    }
                }
                TokenKind::KeywordVar | TokenKind::KeywordConst => self.parse_var_declaration(),
                _ => {
                    self.syntax_error(
                        "Expected a top-level declaration",
                        &token,
                        Some("Only global variables, functions, and main are allowed at compilation-unit level"),
                        Some(ErrorCode::E1004),
                    );
                    // Keep the statement node for downstream semantic analysis.
                    // The syntax error still makes the compilation unit invalid, but
                    // retaining the node allows later diagnostics to be reported.
                    self.parse_stmt()
                }
            };
            if let Some(declaration) = declaration {
                declarations.push(declaration);
            } else if !self.is_at_end() && self.current == before {
                self.advance();
            }
        }
        let unit_span = declarations
            .first()
            .zip(declarations.last())
            .map_or_else(SourceSpan::default, |(first, last)| first.span().merge(last.span()));
        declarations.shrink_to_fit();
        self.errors.shrink_to_fit();
        (CompilationUnit::new(declarations, unit_span), self.errors)
    }

    /// Parses one statement.
    ///
    /// The first token selects the kind of statement. A token that starts no other statement
    /// starts an expression statement.
    ///
    /// A nested `fun` or `main` declaration causes error `E1004`. The parser parses the nested
    /// declaration, discards it, and returns `None`.
    #[inline]
    fn parse_stmt(&mut self) -> Option<Stmt> {
        let token = self.peek()?;
        match token.kind {
            TokenKind::KeywordFun | TokenKind::KeywordMain => {
                let token = self.peek()?.clone();
                self.syntax_error(
                    "Nested function and main declarations are not allowed",
                    &token,
                    Some("Move this declaration to compilation-unit level"),
                    Some(ErrorCode::E1004),
                );
                if matches!(token.kind, TokenKind::KeywordFun) {
                    let _ = self.parse_function();
                } else {
                    let _ = self.parse_main_function();
                }
                None
            }
            TokenKind::KeywordIf => self.parse_if(),
            TokenKind::KeywordVar | TokenKind::KeywordConst => self.parse_var_declaration(),
            TokenKind::KeywordReturn => self.parse_return(),
            TokenKind::KeywordWhile => self.parse_while(),
            TokenKind::KeywordFor => self.parse_for(),
            TokenKind::KeywordBreak => self.parse_break(),
            TokenKind::KeywordContinue => self.parse_continue(),
            TokenKind::OpenBrace => self.parse_block_stmt(),
            _ => self.parse_expression_stmt(),
        }
    }
    /// Parses the `main` declaration: the keyword `main` and a block.
    ///
    /// If `{` does not follow the keyword, the function stores an error, skips tokens up to the next
    /// top-level declaration, and returns `None`.
    fn parse_main_function(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'main'
        if !self.check(&TokenKind::OpenBrace) {
            self.expect(&TokenKind::OpenBrace, "after 'main'");
            self.synchronize_to_top_level_declaration();
            return None;
        }

        let body = self.parse_block_stmt()?;
        let end_span = body.span();

        let function_span = start_token.span.merge(end_span);

        Some(Stmt::MainFunction { body: Box::new(body), span: function_span })
    }

    /// Skips tokens until the current token is `fun`, `main`, `var`, or `const`, or until the end of
    /// the tokens.
    fn synchronize_to_top_level_declaration(&mut self) {
        while !self.is_at_end()
            && !matches!(
                self.peek().map(|token| &token.kind),
                Some(TokenKind::KeywordFun | TokenKind::KeywordMain | TokenKind::KeywordVar | TokenKind::KeywordConst)
            )
        {
            self.advance();
        }
    }
    /// Parses a break statement.
    #[inline]
    fn parse_break(&mut self) -> Option<Stmt> {
        let span = self.advance_and_get_span()?;
        Some(Stmt::Break { span })
    }

    /// Parses a continue statement.
    #[inline]
    fn parse_continue(&mut self) -> Option<Stmt> {
        let span = self.advance_and_get_span()?;
        Some(Stmt::Continue { span })
    }

    /// Consumes the current token and returns its span.
    ///
    /// At the end of the tokens, the function returns the span of the last token. The result is
    /// `None` only if the token slice is empty.
    fn advance_and_get_span(&mut self) -> Option<SourceSpan> {
        let token = self.advance()?; // Use reference
        Some(token.span.clone())
    }

    /// Parses a block.
    ///
    /// The function consumes the current token without a check. It assumes that this token is
    /// `{`. Then it parses statements until `}` or the end of the tokens. If a statement consumes no
    /// token, the function skips one token. If `}` is missing, the function stores error `E1004`.
    /// The function returns a block also in this case.
    fn parse_block_stmt(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // '{'
        let mut statements = Vec::new();

        while !self.check(&TokenKind::CloseBrace) && !self.is_at_end() {
            let before = self.current;
            if let Some(stmt) = self.parse_stmt() {
                statements.push(stmt);
            }
            if self.current == before {
                self.advance();
            }
        }

        self.expect(&TokenKind::CloseBrace, "end of block");
        Some(Stmt::Block { statements, span: self.merge_span(&start_token) })
    }

    /// Parses a `return` statement.
    ///
    /// The statement has no value if the next token is `}`, `;`, or the end of the tokens.
    /// Otherwise, the function parses an expression as the value. If this expression fails, the
    /// function returns `None`.
    fn parse_return(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'return'
        let return_value = if self.is_end_of_statement() { None } else { Some(self.parse_expr(0)?) };

        Some(Stmt::Return {
            value: return_value.clone(),
            span: self.calculate_return_span(&start_token, &return_value),
        })
    }

    /// Checks if the current token is `}`, `;`, or the end of the tokens.
    #[inline]
    fn is_end_of_statement(&self) -> bool {
        matches!(self.peek().map(|t| &t.kind), Some(&TokenKind::CloseBrace | &TokenKind::Eof | &TokenKind::Semicolon))
    }

    /// Calculates the span of a return statement.
    ///
    /// The span goes from the `return` token to the end of the value. Without a value, the span is the
    /// span of the `return` token.
    #[inline]
    #[allow(clippy::ref_option, clippy::unused_self)]
    fn calculate_return_span(&self, start: &Token, value: &Option<Expr>) -> SourceSpan {
        value.as_ref().map_or_else(|| start.span.clone(), |v| start.span.merge(v.span()))
    }

    /// Parses a function declaration.
    ///
    /// The syntax is `fun name(param: type, ...)`, an optional `: return_type`, and a block. The
    /// return type is [`TypeSyntax::Void`] if the declaration has no return type. The span of a
    /// parameter goes from its name to its type.
    fn parse_function(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'fun'
        let name = self.consume_identifier()?;
        let _name_span = self.previous()?.span.clone();

        self.expect(&TokenKind::OpenParen, "after function name");
        let mut params = Vec::new();
        while !self.check(&TokenKind::CloseParen) && !self.is_at_end() {
            let name = self.consume_identifier()?;
            let name_span = self.previous()?.span.clone();
            self.expect(&TokenKind::Colon, "after parameter name");
            let type_ann = self.parse_type()?;
            let type_span = self.previous()?.span.clone();
            let param_span = name_span.merge(&type_span);
            params.push(Parameter { name: name.to_string(), type_annotation: type_ann, span: param_span });
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        params.shrink_to_fit();
        self.expect(&TokenKind::CloseParen, "after parameter list");

        let return_type = if self.match_token(&TokenKind::Colon) {
            let type_ann = self.parse_type()?;
            Some(type_ann)
        } else {
            None
        };

        let body = self.parse_block_stmt()?;
        let end_span = body.span();
        let function_span = start_token.span.merge(end_span);

        Some(Stmt::Function {
            name: name.to_string(),
            parameters: params,
            return_type: return_type.unwrap_or(TypeSyntax::Void),
            body: Box::new(body),
            span: function_span,
        })
    }

    /// Parses the condition of an `if` or `while` statement.
    ///
    /// The syntax is `(` expression `)`. The function stores error `E1004` for a missing `(` or
    /// `)`. If the expression fails, the function returns `None`.
    ///
    /// # Arguments
    ///
    /// * `keword` - The keyword of the statement. The error message uses it.
    fn parse_condition(&mut self, keword: &str) -> Option<Expr> {
        self.expect(&TokenKind::OpenParen, format!("after '{keword}'").as_str());
        let condition = self.parse_expr(0)?;
        self.expect(&TokenKind::CloseParen, "after the condition");
        Some(condition)
    }

    /// Parses an `if` statement.
    ///
    /// The statement has a condition, a block, and an optional `else` part. After `else`, the
    /// function parses a statement. A block becomes [`ElseBranch::Block`] and an `if` statement
    /// becomes [`ElseBranch::ElseIf`]. For any other statement, the function stores error `E1004`
    /// and uses [`ElseBranch::None`].
    fn parse_if(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'if'
        let condition = self.parse_condition("if")?;
        let then_branch = self.parse_block_stmt()?;

        let else_branch = if self.match_token(&TokenKind::KeywordElse) {
            match self.parse_stmt() {
                Some(block @ Stmt::Block { .. }) => ElseBranch::Block(Box::new(block)),
                Some(if_stmt @ Stmt::If { .. }) => ElseBranch::ElseIf(Box::new(if_stmt)),
                Some(invalid_stmt) => {
                    self.errors.push(CompileError::SyntaxError {
                        code: Some(ErrorCode::E1004),
                        message: "Invalid else branch".into(),
                        span: invalid_stmt.span().clone(),
                        help: Some("The else branch must be a block or else-if".to_string()),
                    });
                    ElseBranch::None
                }
                None => ElseBranch::None,
            }
        } else {
            ElseBranch::None
        };

        Some(Stmt::If {
            condition: Box::new(condition),
            then_branch: Box::new(then_branch),
            else_branch,
            span: self.merge_span(&start_token),
        })
    }

    /// Parses a `while` statement: a condition and a block.
    fn parse_while(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'while'
        let condition = self.parse_condition("while")?;
        let body = self.parse_block_stmt()?;
        let span = start_token.span.merge(body.span());
        Some(Stmt::While { condition: Box::new(condition), body: Box::new(body), span })
    }

    /// Parses the initializer of a `for` statement, including the `;` that follows it.
    ///
    /// The initializer is a `var` or `const` declaration or an expression statement. If the next
    /// token is `;`, the initializer is empty. The function consumes this `;` and returns `None`.
    fn parse_for_initializer(&mut self) -> Option<Box<Stmt>> {
        if self.match_token(&TokenKind::Semicolon) {
            // Empty initializer
            return None;
        }

        let stmt = if self.check(&TokenKind::KeywordVar) || self.check(&TokenKind::KeywordConst) {
            self.parse_var_declaration()
        } else {
            self.parse_expression_stmt()
        };

        self.expect(&TokenKind::Semicolon, "after for loop initializer");
        stmt.map(Box::new)
    }

    /// Parses a `for` statement.
    ///
    /// The syntax is `for (initializer; condition; increment) body`. The initializer, the
    /// condition, and the increment are optional. The body is one statement.
    fn parse_for(&mut self) -> Option<Stmt> {
        let start_token = self.advance()?.clone(); // 'for'
        self.expect(&TokenKind::OpenParen, "after 'for'");

        // 1. Parse initializer (can be var/const, an expression, or empty)
        let initializer = self.parse_for_initializer();

        // 2. Parse condition (optional)
        let condition = if self.check(&TokenKind::Semicolon) {
            self.advance(); // Consume the semicolon
            None
        } else {
            let expr = self.parse_expr(0);
            self.expect(&TokenKind::Semicolon, "after for loop condition");
            expr
        };

        // 3. Parse increment (optional)
        let increment = if self.check(&TokenKind::CloseParen) {
            None
        } else {
            // There is no semicolon after the increment
            self.parse_expr(0)
        };

        // 4. Close the parenthesis
        self.expect(&TokenKind::CloseParen, "after for loop clauses");

        // 5. Parse body
        let body_stmt = self.parse_stmt()?;

        // Compute the total span (from the 'for' token to the end of the body)
        let span = start_token.span.merge(body_stmt.span());
        Some(Stmt::For { initializer, condition, increment, body: Box::new(body_stmt), span })
    }

    /// Parses a type.
    ///
    /// The type starts with a primitive type name or an identifier. For any other token, the
    /// function stores error `E1002` and returns `None`. Square brackets with a size expression can
    /// follow the name. The first pair of brackets is the outermost array. The identifier `vector`
    /// followed by `<` starts a vector type: `vector<type>`.
    fn parse_type(&mut self) -> Option<TypeSyntax> {
        let token = self.advance()?.clone();
        let mut base_type = match &token.kind {
            TokenKind::TypeI8 => TypeSyntax::I8,
            TokenKind::TypeI16 => TypeSyntax::I16,
            TokenKind::TypeI32 => TypeSyntax::I32,
            TokenKind::TypeI64 => TypeSyntax::I64,
            TokenKind::TypeU8 => TypeSyntax::U8,
            TokenKind::TypeU16 => TypeSyntax::U16,
            TokenKind::TypeU32 => TypeSyntax::U32,
            TokenKind::TypeU64 => TypeSyntax::U64,
            TokenKind::TypeF32 => TypeSyntax::F32,
            TokenKind::TypeF64 => TypeSyntax::F64,
            TokenKind::TypeChar => TypeSyntax::Char,
            TokenKind::TypeString => TypeSyntax::String,
            TokenKind::TypeBool => TypeSyntax::Bool,
            TokenKind::IdentifierAscii(name) | TokenKind::IdentifierUnicode(name) => {
                TypeSyntax::Custom { name: name.clone() }
            }
            _ => {
                self.syntax_error(
                    "Invalid type specification, expected primitive type or custom identifier",
                    &token,
                    Some("Try using a primitive type (like i32, f64) or a custom type identifier"),
                    Some(ErrorCode::E1002),
                );
                return None;
            }
        };

        // Collect dimensions in a vector instead of nesting immediately
        let mut dimensions = Vec::new();
        while self.match_token(&TokenKind::OpenBracket) {
            let size_expr = self.parse_expr(0)?;
            self.expect(&TokenKind::CloseBracket, "after array size");
            dimensions.push(size_expr);
        }
        dimensions.shrink_to_fit();

        // Apply dimensions in reverse order
        for size in dimensions.into_iter().rev() {
            base_type = TypeSyntax::Array { element_type: Box::new(base_type), size: Box::new(size) };
        }

        #[allow(clippy::collapsible_if)]
        if let TypeSyntax::Custom { name } = &base_type {
            if **name == *"vector" && self.match_token(&TokenKind::Less) {
                let inner_type = self.parse_type()?;
                self.expect(&TokenKind::Greater, "after vector inner type");
                base_type = TypeSyntax::Vector { element_type: Box::new(inner_type) };
            }
        }

        Some(base_type)
    }

    /// Parses a variable declaration.
    ///
    /// The syntax is `var` or `const`, one or more names, `:`, a type, `=`, and one initializer for each
    /// name. The names and the initializers are in lists with commas. The declaration is mutable for
    /// `var` and not mutable for `const`.
    ///
    /// The function stores these errors:
    ///
    /// * `E1008` if no name follows the keyword. The function returns `None`.
    /// * `E1004` for a missing `:` or `=`, an invalid type, or a missing initializer.
    /// * `E2001` if the number of initializers is not equal to the number of names.
    ///
    /// For an invalid type, the declaration uses [`TypeSyntax::Void`]. A name without an initializer
    /// has `None` as the initializer.
    #[allow(clippy::if_same_then_else)]
    fn parse_var_declaration(&mut self) -> Option<Stmt> {
        let (start_token, is_mutable) = self
            .match_token(&TokenKind::KeywordConst)
            .then(|| self.previous().map(|t| (t.clone(), false)))
            .flatten()
            .or_else(|| {
                self.match_token(&TokenKind::KeywordVar).then(|| self.previous().map(|t| (t.clone(), true))).flatten()
            })?;

        let mut variables = Vec::new();
        while let Some(name) = self.consume_identifier() {
            let name_span = self.previous().map_or_else(|| start_token.span.clone(), |token| token.span.clone());
            variables.push((name, name_span));
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        if variables.is_empty() {
            self.syntax_error(
                "Expected at least one variable name",
                &start_token,
                Some("Provide at least one variable name after 'var' or 'const'"),
                Some(ErrorCode::E1008),
            );
            return None;
        }

        self.expect(&TokenKind::Colon, "after variable name(s)");
        let type_ann = self.parse_type().unwrap_or_else(|| {
            self.report_peek_error(
                "Invalid type specification",
                Some("Try using a primitive type or a custom type identifier"),
            );
            TypeSyntax::Void
        });

        self.expect(&TokenKind::Equal, "after type annotation");
        let mut initializers = Vec::with_capacity(variables.len());
        loop {
            if let Some(expr) = self.parse_expr(0) {
                initializers.push(expr);
            } else {
                self.report_peek_error(
                    "Expected initializer expression",
                    Some("Provide an expression to initialize the variable (e.g., 42, \"text\", variable_name)"),
                );
                break;
            }
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }

        if variables.len() != initializers.len() {
            self.syntax_error(
                format!("Declaration mismatch: {} variables but {} initializers", variables.len(), initializers.len()),
                &start_token,
                Some("Each variable must have exactly one initializer expression"),
                Some(ErrorCode::E2001),
            );
        }

        let bindings = variables
            .into_iter()
            .zip(initializers.into_iter().map(Some).chain(std::iter::repeat(None)))
            .map(|((name, name_span), initializer)| {
                let span = initializer.as_ref().map_or_else(|| name_span.clone(), |expr| name_span.merge(expr.span()));
                VarBinding { name: name.to_string(), initializer, span }
            })
            .collect();

        Some(Stmt::VarDeclaration {
            bindings,
            type_annotation: type_ann,
            is_mutable,
            span: self.merge_span(&start_token),
        })
    }

    /// Stores a syntax error with code `E1004` at the current token. The function does nothing at the end
    /// of the tokens.
    fn report_peek_error(&mut self, message: &str, help: Option<&str>) {
        if let Some(token) = &self.peek().cloned() {
            self.syntax_error(message, token, help, Some(ErrorCode::E1004));
        }
    }

    /// Consumes an identifier and returns its name.
    ///
    /// If the current token is not an identifier, the function stores error `E1008` and returns
    /// `None`. It does not consume the token in this case.
    fn consume_identifier(&mut self) -> Option<Arc<str>> {
        // Capture token first to avoid overlapping borrows
        let token = self.peek()?.clone();

        match &token.kind {
            TokenKind::IdentifierAscii(s) | TokenKind::IdentifierUnicode(s) => {
                self.advance();
                Some(s.clone())
            }
            _ => {
                self.syntax_error(
                    "Expected identifier",
                    &token,
                    Some("An identifier must start with a letter/underscore and contain only alphanumeric characters"),
                    Some(ErrorCode::E1008),
                );
                None
            }
        }
    }

    /// Parses an expression and wraps it in an expression statement.
    fn parse_expression_stmt(&mut self) -> Option<Stmt> {
        let expr = self.parse_expr(0)?;
        Some(Stmt::Expression { expr: Box::new(expr) })
    }

    /// Parses an expression.
    ///
    /// The function keeps track of the recursion depth. If the depth is greater than the limit, it
    /// returns `None` (see [`Self::MAX_RECURSION_DEPTH`]).
    ///
    /// # Arguments
    ///
    /// * `min_bp` - The minimum binding power. The function stops before an operator with a left
    ///   binding power that is not greater than this value.
    fn parse_expr(&mut self, min_bp: u8) -> Option<Expr> {
        // Check the recursion limit to prevent a stack overflow
        if self.check_recursion_limit() {
            return None;
        }

        self.enter_recursion();

        let result = self.parse_expr_inner(min_bp);

        self.exit_recursion();
        result
    }

    /// Parses an expression with the binding power method.
    ///
    /// The function parses a first expression with `nud`. Then, while the next token has a left
    /// binding power that is greater than `min_bp`, it extends the expression with `led`.
    fn parse_expr_inner(&mut self, min_bp: u8) -> Option<Expr> {
        let mut left = self.nud()?;

        while let Some(token) = self.peek() {
            // A line break makes ++/-- a prefix for the next statement, not postfix on `left`.
            if matches!(&token.kind, TokenKind::PlusPlus | TokenKind::MinusMinus)
                && token.span.start().line() > left.span().end().line()
            {
                break;
            }

            let lbp = Precedence::binding_power(token).left;
            if lbp <= min_bp {
                break;
            }
            left = self.led(left)?;
        }

        Some(left)
    }

    /// Parses a token that starts an expression.
    ///
    /// The function consumes one token. The result is a literal, a prefix operation, an array literal,
    /// a grouping, or a variable. For any other token, the function stores error `E1004` and returns
    /// `None`.
    fn nud(&mut self) -> Option<Expr> {
        let token = self.advance()?.clone();
        match token.kind {
            // Literals
            TokenKind::Numeric(n) => Some(Expr::new_number_literal(n, token.span)),
            TokenKind::KeywordBool(b) => Some(Expr::new_bool_literal(b, token.span)),
            TokenKind::KeywordNullptr => Some(Expr::new_nullptr_literal(token.span)),
            TokenKind::StringLiteral(s) => Some(Expr::new_string_literal(s.to_string(), token.span)),
            TokenKind::CharLiteral(c) => Some(Expr::new_char_literal(c.to_string(), token.span)),
            // Unary operators
            TokenKind::Minus => Some(self.parse_unary(UnaryOp::Negate, &token)),
            TokenKind::Not => Some(self.parse_unary(UnaryOp::Not, &token)),
            TokenKind::BitwiseNot => Some(self.parse_unary(UnaryOp::BitwiseNot, &token)),
            TokenKind::PlusPlus => Some(self.parse_unary(UnaryOp::Increment, &token)),
            TokenKind::MinusMinus => Some(self.parse_unary(UnaryOp::Decrement, &token)),
            // Grouping
            TokenKind::OpenBrace => self.parse_array_literal(&token),
            TokenKind::OpenParen => self.parse_grouping(&token),
            // Variables
            TokenKind::IdentifierAscii(name) | TokenKind::IdentifierUnicode(name) => {
                Some(Expr::Variable { name: name.to_string(), span: token.span })
            }
            _ => {
                self.syntax_error(
                    "Unexpected token",
                    &token,
                    Some("Expected an expression (number, string, variable, or operator)"),
                    Some(ErrorCode::E1004),
                );
                None
            }
        }
    }

    /// Parses an infix or postfix token with the expression on its left.
    ///
    /// The function consumes one token. The token can be a binary operator, `++`, `--`, an
    /// assignment operator, `as`, `(` (a call), or `[` (an array access). For any other token, the
    /// function stores error `E1004` and returns `None`.
    fn led(&mut self, left: Expr) -> Option<Expr> {
        let token = self.advance()?.clone();

        match token.kind {
            // Binary operators
            TokenKind::Plus
            | TokenKind::Minus
            | TokenKind::Star
            | TokenKind::Slash
            | TokenKind::Percent
            | TokenKind::EqualEqual
            | TokenKind::NotEqual
            | TokenKind::Less
            | TokenKind::LessEqual
            | TokenKind::Greater
            | TokenKind::GreaterEqual
            | TokenKind::AndAnd
            | TokenKind::OrOr
            | TokenKind::And
            | TokenKind::Or
            | TokenKind::Xor
            | TokenKind::ShiftLeft
            | TokenKind::ShiftRight => self.parse_binary(left, &token),

            TokenKind::PlusPlus => {
                let span = left.span().merge(&token.span);
                Some(Expr::Unary { op: UnaryOp::Increment, side: UnaryOpSide::Postfix, expr: Box::new(left), span })
            }
            TokenKind::MinusMinus => {
                let span = left.span().merge(&token.span);
                Some(Expr::Unary { op: UnaryOp::Decrement, side: UnaryOpSide::Postfix, expr: Box::new(left), span })
            }

            // Simple and compound assignment
            TokenKind::Equal
            | TokenKind::PlusEqual
            | TokenKind::MinusEqual
            | TokenKind::StarEqual
            | TokenKind::SlashEqual
            | TokenKind::PercentEqual
            | TokenKind::AndEqual
            | TokenKind::OrEqual
            | TokenKind::XorEqual
            | TokenKind::ShiftLeftEqual
            | TokenKind::ShiftRightEqual => self.parse_assignment(left, &token),
            // Explicit cast
            TokenKind::KeywordAs => self.parse_cast(left),
            // Function call
            TokenKind::OpenParen => self.parse_call(left, &token),
            // Array access
            TokenKind::OpenBracket => self.parse_array_access(left, &token),
            _ => {
                self.syntax_error(
                    "Unexpected operator",
                    &token,
                    Some("This operator is not supported in this context"),
                    Some(ErrorCode::E1004),
                );
                None
            }
        }
    }

    /// Parses `expression as type`, accepting the same type syntax as declarations.
    fn parse_cast(&mut self, expr: Expr) -> Option<Expr> {
        let Some(target_start) = self.peek().cloned() else {
            self.errors.push(CompileError::SyntaxError {
                code: Some(ErrorCode::E1002),
                message: "Missing cast target type".into(),
                span: expr.span().clone(),
                help: Some("Add a supported type after 'as'".into()),
            });
            return None;
        };
        if self.is_at_end() {
            self.syntax_error(
                "Missing cast target type",
                &target_start,
                Some("Add a supported type after 'as'"),
                Some(ErrorCode::E1002),
            );
            return None;
        }

        let target_type = self.parse_type()?;
        let target_end = self.previous().map_or_else(|| target_start.span.clone(), |token| token.span.clone());
        let span = expr.span().merge(&target_start.span.merge(&target_end));
        Some(Expr::Cast { expr: Box::new(expr), target_type, span })
    }

    #[inline]
    /// Returns whether an expression can be used as an assignment target.
    const fn is_valid_assignment_target(expr: &Expr) -> bool {
        matches!(expr, Expr::Variable { .. } | Expr::ArrayAccess { .. })
    }

    /// Reports the standard parser error for an invalid assignment target.
    fn report_invalid_assignment_target(&mut self, left: &Expr) {
        let help_msg = "Only variables and array elements can be assigned to. Consider using a variable name or an array access expression.";

        self.errors.push(CompileError::SyntaxError {
            code: Some(ErrorCode::E1003),
            message: "Invalid left-hand side in assignment".into(),
            span: left.span().clone(),
            help: Some(help_msg.to_string()),
        });
    }

    /// Parses a prefix operation.
    ///
    /// The function parses the operand with the right binding power of the operator. If the operand
    /// fails, the operand is a null pointer literal at the span of the operator. The error of the
    /// operand is already stored.
    fn parse_unary(&mut self, op: UnaryOp, token: &Token) -> Expr {
        let rbp = Precedence::unary_binding_power(token).right;
        let expr = self.parse_expr(rbp).unwrap_or_else(|| Expr::null_expr(token.span.clone()));
        Expr::Unary { op, side: UnaryOpSide::Prefix, span: token.span.merge(expr.span()), expr: Box::new(expr) }
    }

    /// Parses the elements of an array literal and the closing `}`.
    ///
    /// The function expects that the caller has consumed the opening `{`. If `}` is missing, the
    /// function stores error `E1004` and returns `None`.
    fn parse_array_literal(&mut self, start_token: &Token) -> Option<Expr> {
        let mut elements = Vec::new();
        let cbr = &TokenKind::CloseBrace;
        self.extract_elements(cbr, &mut elements);
        if !self.expect(cbr, "end of array literal") {
            return None;
        }
        elements.shrink_to_fit();
        Some(Expr::ArrayLiteral { elements, span: self.merge_span(start_token) })
    }

    /// Parses expressions with commas between them until the token `kind` or the end of the tokens.
    ///
    /// The function adds each parsed expression to `elements`. A comma after the last expression is
    /// allowed. If no comma follows an expression, the function stops.
    fn extract_elements(&mut self, kind: &TokenKind, elements: &mut Vec<Expr>) {
        while !self.check(&kind.clone()) && !self.is_at_end() {
            if let Some(expr) = self.parse_expr(0) {
                elements.push(expr);
            }
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
    }

    /// Parses the right operand of a binary operator.
    ///
    /// The function uses the right binding power of the operator. If the operand fails, the operand is
    /// a null pointer literal at the span of the operator. If the token is not a binary operator, the
    /// function stores the error of [`BinaryOp::get_op`] and returns `None`.
    fn parse_binary(&mut self, left: Expr, token: &Token) -> Option<Expr> {
        let op = match BinaryOp::get_op(token) {
            Ok(op) => op,
            Err(e) => {
                self.errors.push(e);
                return None;
            }
        };

        let right = self
            .parse_expr(Precedence::binding_power(token).right)
            .unwrap_or_else(|| Expr::null_expr(token.span.clone()));
        let span = left.span().merge(right.span());
        Some(Expr::Binary { left: Box::new(left), op, right: Box::new(right), span })
    }

    /// Parses the expression of a grouping and the closing `)`.
    ///
    /// The function expects that the caller has consumed the opening `(`. If `)` is missing or the
    /// expression fails, the function returns `None`.
    fn parse_grouping(&mut self, start_token: &Token) -> Option<Expr> {
        let expr = self.parse_expr(0);
        if !self.expect(&TokenKind::CloseParen, "end of grouping") {
            return None;
        }
        Some(Expr::Grouping { expr: Box::new(expr?), span: self.merge_span(start_token) })
    }

    /// Parses simple or compound assignment after validating its left-hand side.
    ///
    /// The function parses the value first. Then it checks the target. A target that is not a
    /// variable or an array element causes error `E1003`, and the function returns `None`. The
    /// assignment operators associate to the right.
    fn parse_assignment(&mut self, left: Expr, token: &Token) -> Option<Expr> {
        let op = match AssignmentOp::get_op(token) {
            Ok(op) => op,
            Err(error) => {
                self.errors.push(error);
                return None;
            }
        };
        let value = self
            .parse_expr(Precedence::binding_power(token).right)
            .unwrap_or_else(|| Expr::null_expr(token.span.clone()));

        let span = left.span().merge(value.span());

        if !Self::is_valid_assignment_target(&left) {
            self.report_invalid_assignment_target(&left);
            return None;
        }

        Some(Expr::Assign { op, target: Box::new(left), value: Box::new(value), span })
    }

    /// Parses the arguments of a call and the closing `)`.
    ///
    /// The function expects that the caller has consumed the opening `(`. If `)` is missing, the
    /// function stores error `E1004` and returns `None`.
    fn parse_call(&mut self, callee: Expr, start_token: &Token) -> Option<Expr> {
        let mut arguments = Vec::new();
        let cpr = &TokenKind::CloseParen;
        self.extract_elements(cpr, &mut arguments);

        // Check if we successfully found the closing parenthesis
        if !self.expect(cpr, "end of function call arguments") {
            return None;
        }
        arguments.shrink_to_fit();

        let span = callee.span().merge(&self.merge_span(start_token));
        Some(Expr::Call { callee: Box::new(callee), arguments, span })
    }

    /// Parses the index of an array access and the closing `]`.
    ///
    /// The function expects that the caller has consumed the opening `[`. If the index fails, the
    /// index is a null pointer literal. If `]` is missing, the function stores error `E1004` and
    /// returns `None`.
    fn parse_array_access(&mut self, array: Expr, start_token: &Token) -> Option<Expr> {
        let index = self.parse_expr(0).unwrap_or_else(|| Expr::null_expr(start_token.span.clone()));
        if !self.expect(&TokenKind::CloseBracket, "end of array access") {
            return None;
        }
        let span = array.span().merge(&self.merge_span(start_token));
        Some(Expr::ArrayAccess { array: Box::new(array), index: Box::new(index), span })
    }

    /// Returns the span from `start_token` to the previous token. If no previous token exists, the result is the
    /// span of `start_token`.
    #[inline]
    fn merge_span(&self, start_token: &Token) -> SourceSpan {
        self.previous().map_or_else(|| start_token.span.clone(), |end| start_token.span.merge(&end.span))
    }

    /// Stores a syntax error at a token.
    ///
    /// The message of the error is the given message, a colon, and the kind of the token.
    ///
    /// # Arguments
    ///
    /// * `message` - The message of the error.
    /// * `token` - The token that gives the span and the text at the end of the message.
    /// * `help` - The optional help text.
    /// * `error_code` - The optional error code.
    fn syntax_error(
        &mut self, message: impl Into<String>, token: &Token, help: Option<&str>, error_code: Option<ErrorCode>,
    ) {
        let message_str = message.into();
        self.errors.push(CompileError::SyntaxError {
            code: error_code,
            message: Arc::from(format!("{}: {}", message_str, token.kind)),
            span: token.span.clone(),
            help: help.map(std::string::ToString::to_string),
        });
    }

    /// Consumes the current token if it has the given kind.
    ///
    /// If the kind is different, the function does not consume the token. It stores error `E1004`
    /// with the message `Expected <kind> in <context>, found <token>.` The help text is `Try adding a
    /// <kind>`. At the end of the tokens, the found text is `end of input`.
    ///
    /// # Arguments
    ///
    /// * `kind` - The expected token kind.
    /// * `context` - The text that describes the place of the token in the message.
    ///
    /// # Returns
    ///
    /// `true` if the token has the given kind. Otherwise, `false`.
    fn expect(&mut self, kind: &TokenKind, context: &str) -> bool {
        if self.match_token(&kind.clone()) {
            true
        } else {
            let current_token = self.peek().cloned();
            let expected = &kind.clone();
            let found_str = current_token.as_ref().map_or_else(|| "end of input".to_string(), |t| t.kind.to_string());

            let span = current_token.as_ref().map_or_default(|t| t.span.clone());

            let error_message = Arc::from(format!("Expected {expected} in {context}, found {found_str}."));
            let help_message = format!("Try adding a {expected}");
            self.errors.push(CompileError::SyntaxError {
                code: Some(ErrorCode::E1004),
                message: error_message,
                span,
                help: Some(help_message),
            });
            false
        }
    }

    /// Consumes the current token if it has the given kind. Returns `true` if the function consumed it.
    #[inline]
    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Moves to the next token, unless the parser is at the end, and returns the previous token.
    ///
    /// The previous token is the token that the function consumed. At the end of the tokens, the
    /// function does not move and returns the last token.
    #[inline]
    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Returns the token before the current token. At the start, the result is the first token.
    #[inline]
    fn previous(&self) -> Option<&Token> {
        self.tokens.get(self.current.saturating_sub(1))
    }

    /// Returns the current token without consuming it. The result is `None` after the last token.
    #[inline]
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    /// Checks if the current token has the given kind.
    #[inline]
    fn check(&self, kind: &TokenKind) -> bool {
        self.peek().is_some_and(|t| &t.kind == kind)
    }

    /// Checks if the parser is at the end of the tokens. The parser is at the end if no token remains or
    /// the current token is `Eof`.
    #[inline]
    fn is_at_end(&self) -> bool {
        self.peek().is_none_or(|t| t.kind == TokenKind::Eof)
    }
}
