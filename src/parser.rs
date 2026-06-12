use crate::ast::{Expression, Statement};
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    fn peek(&self) -> &Token {
        if self.position < self.tokens.len() {
            &self.tokens[self.position]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek().clone();
        if self.position < self.tokens.len() {
            self.position += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek().kind == *kind
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, kind: TokenKind, msg: &str) -> Result<Token, String> {
        let tok = self.peek().clone();
        if tok.kind == kind {
            self.advance();
            Ok(tok)
        } else {
            Err(format!(
                "Parser Error: {} at line {}, col {}. Got {:?}",
                msg, tok.line, tok.column, tok.kind
            ))
        }
    }

    fn consume_identifier(&mut self, msg: &str) -> Result<String, String> {
        let tok = self.peek().clone();
        if let TokenKind::Identifier(name) = tok.kind {
            self.advance();
            Ok(name)
        } else {
            Err(format!(
                "Parser Error: {} at line {}, col {}. Got {:?}",
                msg, tok.line, tok.column, tok.kind
            ))
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while !self.check(&TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        if self.check(&TokenKind::Let) {
            self.parse_variable_decl()
        } else if self.check(&TokenKind::Fn) {
            self.parse_function_decl()
        } else if self.check(&TokenKind::While) {
            self.parse_while_loop()
        } else {
            self.parse_expression_stmt()
        }
    }

    fn parse_variable_decl(&mut self) -> Result<Statement, String> {
        self.consume(TokenKind::Let, "Expected 'let'")?;
        let name = self.consume_identifier("Expected identifier")?;
        self.consume(TokenKind::Assign, "Expected '='")?;
        let expr = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Statement::Let(name, expr))
    }

    fn parse_function_decl(&mut self) -> Result<Statement, String> {
        self.consume(TokenKind::Fn, "Expected 'fn'")?;
        let name = self.consume_identifier("Expected function name")?;
        self.consume(TokenKind::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&TokenKind::RParen) {
            loop {
                let p = self.consume_identifier("Expected parameter name")?;
                params.push(p);
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        self.consume(TokenKind::RParen, "Expected ')' after parameters")?;
        let body = self.parse_block_expression()?;
        Ok(Statement::FnDecl(name, params, body))
    }

    fn parse_while_loop(&mut self) -> Result<Statement, String> {
        self.consume(TokenKind::While, "Expected 'while'")?;
        let cond = self.parse_expression()?;
        let body = self.parse_block_expression()?;
        Ok(Statement::While(cond, body))
    }

    fn parse_expression_stmt(&mut self) -> Result<Statement, String> {
        let expr = self.parse_expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after expression statement")?;
        Ok(Statement::Expr(expr))
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_pipeline()
    }

    fn parse_pipeline(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;

        while self.match_token(TokenKind::Pipeline) {
            let func_name = self.consume_identifier("Expected function identifier after '|>'")?;
            self.consume(TokenKind::LParen, "Expected '(' after pipelined function name")?;
            let mut args = vec![expr]; // Pipelinend expression is the first argument
            if !self.check(&TokenKind::RParen) {
                loop {
                    args.push(self.parse_expression()?);
                    if !self.match_token(TokenKind::Comma) {
                        break;
                    }
                }
            }
            self.consume(TokenKind::RParen, "Expected ')' after pipelined function arguments")?;
            expr = Expression::Call(func_name, args);
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_additive()?;

        let tok_kind = self.peek().kind.clone();
        if matches!(
            tok_kind,
            TokenKind::Eq | TokenKind::Ne | TokenKind::Lt | TokenKind::Gt | TokenKind::Le | TokenKind::Ge
        ) {
            self.advance();
            let op = match tok_kind {
                TokenKind::Eq => "==".to_string(),
                TokenKind::Ne => "!=".to_string(),
                TokenKind::Lt => "<".to_string(),
                TokenKind::Gt => ">".to_string(),
                TokenKind::Le => "<=".to_string(),
                TokenKind::Ge => ">=".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_additive()?;
            expr = Expression::BinaryOp(op, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn parse_additive(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_multiplicative()?;

        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let tok = self.advance().clone();
            let op = match tok.kind {
                TokenKind::Plus => "+".to_string(),
                TokenKind::Minus => "-".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_multiplicative()?;
            expr = Expression::BinaryOp(op, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn parse_multiplicative(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        while self.check(&TokenKind::Asterisk) || self.check(&TokenKind::Slash) || self.check(&TokenKind::Percent) {
            let tok = self.advance().clone();
            let op = match tok.kind {
                TokenKind::Asterisk => "*".to_string(),
                TokenKind::Slash => "/".to_string(),
                TokenKind::Percent => "%".to_string(),
                _ => unreachable!(),
            };
            let right = self.parse_primary()?;
            expr = Expression::BinaryOp(op, Box::new(expr), Box::new(right));
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let tok = self.peek().clone();
        match tok.kind {
            TokenKind::Number(val) => {
                self.advance();
                Ok(Expression::Number(val))
            }
            TokenKind::String(val) => {
                self.advance();
                Ok(Expression::String(val))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Boolean(true))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Boolean(false))
            }
            TokenKind::Identifier(ref name) => {
                self.advance();
                if self.check(&TokenKind::LParen) {
                    self.consume(TokenKind::LParen, "Expected '('")?;
                    let mut args = Vec::new();
                    if !self.check(&TokenKind::RParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if !self.match_token(TokenKind::Comma) {
                                break;
                            }
                        }
                    }
                    self.consume(TokenKind::RParen, "Expected ')'")?;
                    Ok(Expression::Call(name.clone(), args))
                } else {
                    Ok(Expression::Identifier(name.clone()))
                }
            }
            TokenKind::LBrace => self.parse_block_expression(),
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.consume(TokenKind::RParen, "Expected ')'")?;
                Ok(expr)
            }
            TokenKind::If => self.parse_if_expression(),
            _ => Err(format!(
                "Parser Error: Unexpected token {:?} at line {}, col {}",
                tok.kind, tok.line, tok.column
            )),
        }
    }

    fn parse_block_expression(&mut self) -> Result<Expression, String> {
        self.consume(TokenKind::LBrace, "Expected '{'")?;
        let mut statements = Vec::new();
        let mut trailing_expr = None;

        while !self.check(&TokenKind::RBrace) {
            // Check if the current expression is statement or trailing expression
            if self.check(&TokenKind::Let) || self.check(&TokenKind::Fn) || self.check(&TokenKind::While) {
                statements.push(self.parse_statement()?);
            } else {
                let expr = self.parse_expression()?;
                if self.match_token(TokenKind::Semicolon) {
                    statements.push(Statement::Expr(expr));
                } else {
                    // Check if it's the last element before }
                    if self.check(&TokenKind::RBrace) {
                        trailing_expr = Some(Box::new(expr));
                    } else {
                        return Err(format!(
                            "Parser Error: Expected ';' after expression at line {}, col {}",
                            self.peek().line, self.peek().column
                        ));
                    }
                }
            }
        }
        self.consume(TokenKind::RBrace, "Expected '}'")?;
        Ok(Expression::Block(statements, trailing_expr))
    }

    fn parse_if_expression(&mut self) -> Result<Expression, String> {
        self.consume(TokenKind::If, "Expected 'if'")?;
        let cond = self.parse_expression()?;
        let then_branch = self.parse_block_expression()?;
        self.consume(TokenKind::Else, "Expected 'else'")?;
        let else_branch = if self.check(&TokenKind::If) {
            self.parse_if_expression()?
        } else {
            self.parse_block_expression()?
        };
        Ok(Expression::If(Box::new(cond), Box::new(then_branch), Box::new(else_branch)))
    }
}
