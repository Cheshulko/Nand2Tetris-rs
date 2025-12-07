use itertools::{Itertools, MultiPeek};
use std::convert::TryFrom;

use crate::tokenizer::{Constant, Identifier, Keyword, Symbol, Token, TokenType};

macro_rules! consume {
    ($tokens:expr) => {
        $tokens.next().ok_or(anyhow::anyhow!(
            "Could not consume a token. Token list is empty"
        ))
    };
}

macro_rules! peek {
    ($tokens:expr) => {
        if let Some(res) = $tokens.peek().clone() {
            Ok(res)
        } else {
            Err(anyhow::anyhow!(
                "Could not peek a token. Token list is empty"
            ))
        }
    };
}

macro_rules! peek_matches {
    ($tokens:expr, $( $pattern:pat ),* $(,)?) => {
        if let Some(value) = $tokens.peek() {
            let res = match value {
                $(Token {
                    token_type: $pattern,
                    ..
                } => true, )*
                _ => false,
            };

            $tokens.reset_peek();
            res
        } else {
            $tokens.reset_peek();
            false
        }
    };
}

macro_rules! consume_and_ensure_matches {
    ($tokens:expr, $( $pattern:pat ),* $(,)?) => {
        match $tokens.next() {
            $(Some(token @ Token {
                token_type: $pattern,
                ..
            }) => anyhow::Result::<Token>::Ok(token), )*
            token => {
                let expected_patterns = vec![$(stringify!($pattern)),*];
                Err(anyhow::anyhow!(
                    "Unexpected token. Expected one of: {} but got {:?}",
                    expected_patterns.join(", "),
                    token
                ))
            },
        }
    };
}

#[derive(Debug)]
pub struct ClassVarDec<'de> {
    pub(super) class_var_dec_kind: ClassVarDecKind,
    pub(super) class_var_dec_type: Type<'de>,
    pub(super) var_names: Vec<Identifier<'de>>,
}

#[derive(Debug)]
pub enum ClassVarDecKind {
    Static,
    Field,
}

#[derive(Debug)]
pub struct SubroutineDec<'de> {
    pub(super) subroutine_dec_type: SubroutineDecType,
    pub(super) subroutine_dec_return_type: SubroutineDecReturn<'de>,
    pub(super) subroutine_name: Identifier<'de>,
    pub(super) parameter_list: ParameterList<'de>,
    pub(super) subroutine_body: SubroutineBody<'de>,
}

#[derive(Debug)]
pub enum SubroutineDecType {
    Constructor,
    Function,
    Method,
}

#[derive(Debug)]
pub enum SubroutineDecReturn<'de> {
    Void,
    Type(Type<'de>),
}

#[derive(Debug)]
pub struct VarDec<'de> {
    pub(super) var_type: Type<'de>,
    pub(super) var_names: Vec<Identifier<'de>>,
}

#[derive(Debug)]
pub struct Class<'de> {
    pub(super) class_name: Identifier<'de>,
    pub(super) class_var_decs: Vec<ClassVarDec<'de>>,
    pub(super) subroutine_decs: Vec<SubroutineDec<'de>>,
}

#[derive(Debug)]
pub enum Type<'de> {
    Int,
    Char,
    Boolean,
    Class { name: Identifier<'de> },
}

#[derive(Debug)]
pub struct ParameterList<'de> {
    pub(super) parameters: Vec<(Type<'de>, Identifier<'de>)>,
}

#[derive(Debug)]
pub struct SubroutineBody<'de> {
    pub(super) var_decs: Vec<VarDec<'de>>,
    pub(super) statements: Statements<'de>,
}

#[derive(Debug)]
pub struct Statements<'de> {
    pub(super) statements: Vec<Statement<'de>>,
}

#[derive(Debug)]
pub enum Statement<'de> {
    LetStatement(LetStatement<'de>),
    IfStatement(IfStatement<'de>),
    WhileStatement(WhileStatement<'de>),
    DoStatement(DoStatement<'de>),
    ReturnStatement(ReturnStatement<'de>),
}

#[derive(Debug)]
pub struct LetStatement<'de> {
    pub(super) var_name: Identifier<'de>,
    pub(super) expression_1: Option<Expression<'de>>,
    pub(super) expression_2: Expression<'de>,
}

#[derive(Debug)]
pub struct IfStatement<'de> {
    pub(super) condition: Expression<'de>,
    pub(super) then_branch: Statements<'de>,
    pub(super) else_branch: Option<Statements<'de>>,
}

#[derive(Debug)]
pub struct WhileStatement<'de> {
    pub(super) condition: Expression<'de>,
    pub(super) body: Statements<'de>,
}

#[derive(Debug)]
pub struct DoStatement<'de> {
    pub(super) subroutine_call: SubroutineCall<'de>,
}

#[derive(Debug)]
pub struct ReturnStatement<'de> {
    pub(super) expression: Option<Expression<'de>>,
}

#[derive(Debug)]
pub struct Expression<'de> {
    pub(super) term: Term<'de>,
    pub(super) terms: Vec<(Op, Term<'de>)>,
}

#[derive(Debug)]
pub enum Term<'de> {
    Constant(Constant<'de>),
    KeywordConstant(KeywordConstant),
    VarName(Identifier<'de>),
    VarNameExpression {
        var_name: Identifier<'de>,
        expression: Box<Expression<'de>>,
    },
    Expression(Box<Expression<'de>>),
    UnaryOpTerm {
        unary_op: UnaryOp,
        term: Box<Term<'de>>,
    },
    SubroutineCall(SubroutineCall<'de>),
}

#[derive(Debug)]
pub enum SubroutineCall<'de> {
    Call {
        subroutine_name: Identifier<'de>,
        expression_list: ExpressionList<'de>,
    },
    ClassCall {
        class_or_var_name: Identifier<'de>,
        subroutine_name: Identifier<'de>,
        expression_list: ExpressionList<'de>,
    },
}

#[derive(Debug)]
pub struct ExpressionList<'de> {
    pub(super) expressions: Vec<Expression<'de>>,
}

#[derive(Debug)]
pub enum Op {
    Plus,
    Minus,
    Asterisk,
    Slash,
    Ampersand,
    Pipe,
    LessThan,
    GreaterThan,
    Equal,
}

#[derive(Debug)]
pub enum UnaryOp {
    Minus,
    Tilde,
}

#[derive(Debug)]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}

pub struct Parser<'de, I: Iterator<Item = Token<'de>>> {
    tokens: MultiPeek<I>,
}

impl<'de, I> Parser<'de, I>
where
    I: Iterator<Item = Token<'de>>,
{
    pub fn new(tokens: I) -> Parser<'de, I> {
        Parser {
            tokens: tokens.multipeek(),
        }
    }

    pub fn parse(&mut self) -> Option<anyhow::Result<Class<'de>>> {
        while let Some(token) = self.tokens.peek() {
            if matches!(token.token_type, TokenType::Eof) {
                return None;
            }

            if let Ok(class) = self.parse_class() {
                return Some(Ok(class));
            }

            return None;
        }

        return None;
    }

    fn parse_class(&mut self) -> anyhow::Result<Class<'de>> {
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::Class));

        let class_name = consume!(self.tokens)?.try_into()?;

        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftCurlyBrace));
        let mut class_var_decs = vec![];
        while let Some(class_var_dec) = self.parse_class_var_dec() {
            class_var_decs.push(class_var_dec);
        }

        let mut subroutine_decs = vec![];
        while let Some(subroutine_dec) = self.parse_subroutine_dec() {
            subroutine_decs.push(subroutine_dec);
        }

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightCurlyBrace));

        Ok(Class {
            class_name,
            class_var_decs,
            subroutine_decs,
        })
    }

    fn parse_let_statement(&mut self) -> Option<LetStatement<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::Let)) {
            return None;
        }
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::Let)).ok()?;

        let var_name = self.parse_identifier()?.try_into().ok()?;

        let expression_1 =
            if peek_matches!(self.tokens, TokenType::Symbol(Symbol::LeftSquareBracket)) {
                let _ = consume!(self.tokens).ok()?;

                let expression_1 = self.parse_expression().ok()?;
                let _ = consume_and_ensure_matches!(
                    self.tokens,
                    TokenType::Symbol(Symbol::RightSquareBracket)
                )
                .ok()?;

                Some(expression_1)
            } else {
                None
            };

        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Equal)).ok()?;
        let expression_2 = self.parse_expression().ok()?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon)).ok()?;

        Some(LetStatement {
            var_name,
            expression_1,
            expression_2,
        })
    }

    fn parse_if_statement(&mut self) -> Option<IfStatement<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::If)) {
            return None;
        }
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::If)).ok()?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftParenthesis))
                .ok()?;
        let condition = self.parse_expression().ok()?;
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightParenthesis))
                .ok()?;

        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftCurlyBrace))
            .ok()?;
        let then_branch = self.parse_statements()?;
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightCurlyBrace))
                .ok()?;

        let else_branch = if peek_matches!(self.tokens, TokenType::Keyword(Keyword::Else)) {
            let _ = consume!(self.tokens).ok()?;

            let _ =
                consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftCurlyBrace))
                    .ok()?;
            let else_branch = self.parse_statements()?;
            let _ = consume_and_ensure_matches!(
                self.tokens,
                TokenType::Symbol(Symbol::RightCurlyBrace)
            )
            .ok()?;

            Some(else_branch)
        } else {
            None
        };

        Some(IfStatement {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Option<WhileStatement<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::While)) {
            return None;
        }
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::While)).ok()?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftParenthesis))
                .ok()?;
        let condition = self.parse_expression().ok()?;
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightParenthesis))
                .ok()?;

        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftCurlyBrace))
            .ok()?;
        let body = self.parse_statements()?;
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightCurlyBrace))
                .ok()?;

        Some(WhileStatement { condition, body })
    }

    fn parse_do_statement(&mut self) -> Option<DoStatement<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::Do)) {
            return None;
        }
        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::Do)).ok()?;

        let subroutine_call = self.parse_subroutine_call().ok()?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon)).ok()?;

        Some(DoStatement { subroutine_call })
    }

    fn parse_return_statement(&mut self) -> Option<ReturnStatement<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::Return)) {
            return None;
        }
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::Return)).ok()?;

        let expression = if peek_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon)) {
            let _ = consume!(self.tokens).ok()?;

            None
        } else {
            let expression = self.parse_expression().ok()?;

            let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon))
                .ok()?;

            Some(expression)
        };

        Some(ReturnStatement { expression })
    }

    fn parse_statement(&mut self) -> Option<Statement<'de>> {
        if let Some(let_statement) = self.parse_let_statement() {
            Some(Statement::LetStatement(let_statement))
        } else if let Some(if_statement) = self.parse_if_statement() {
            Some(Statement::IfStatement(if_statement))
        } else if let Some(while_statement) = self.parse_while_statement() {
            Some(Statement::WhileStatement(while_statement))
        } else if let Some(do_statement) = self.parse_do_statement() {
            Some(Statement::DoStatement(do_statement))
        } else if let Some(return_statement) = self.parse_return_statement() {
            Some(Statement::ReturnStatement(return_statement))
        } else {
            None
        }
    }

    fn parse_statements(&mut self) -> Option<Statements<'de>> {
        let mut statements = vec![];

        while let Some(statement) = self.parse_statement() {
            statements.push(statement);
        }

        Some(Statements { statements })
    }

    fn parse_parameeter_list(&mut self) -> Option<ParameterList<'de>> {
        let mut parameters = vec![];

        while let Some(r#type) = self.parse_type() {
            let var_name = self.parse_identifier()?.try_into().ok()?;

            parameters.push((r#type, var_name));

            if peek_matches!(self.tokens, TokenType::Symbol(Symbol::Comma)) {
                let _ = consume!(self.tokens);
            } else {
                break;
            }
        }

        Some(ParameterList { parameters })
    }

    fn parse_var_dec(&mut self) -> Option<VarDec<'de>> {
        if !peek_matches!(self.tokens, TokenType::Keyword(Keyword::Var)) {
            return None;
        }

        let _ = consume_and_ensure_matches!(self.tokens, TokenType::Keyword(Keyword::Var)).ok()?;

        let var_type = self.parse_type()?;

        let mut var_names = vec![];
        loop {
            let var_name = self.parse_identifier()?.try_into().ok()?;
            var_names.push(var_name);

            if peek_matches!(self.tokens, TokenType::Symbol(Symbol::Comma)) {
                let _ = consume!(self.tokens);
            } else {
                break;
            }
        }

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon)).ok()?;

        Some(VarDec {
            var_type,
            var_names,
        })
    }

    fn parse_subroutine_body(&mut self) -> anyhow::Result<SubroutineBody<'de>> {
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftCurlyBrace))?;

        let mut var_decs = vec![];
        while let Some(var_dec) = self.parse_var_dec() {
            var_decs.push(var_dec);
        }

        let statements = self.parse_statements().ok_or(anyhow::anyhow!(
            "Could not `parse_statements` at `parse_subroutine_body`"
        ))?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightCurlyBrace))?;

        Ok(SubroutineBody {
            var_decs,
            statements,
        })
    }

    fn parse_subroutine_dec(&mut self) -> Option<SubroutineDec<'de>> {
        if !matches!(
            peek!(self.tokens).ok()?.token_type,
            TokenType::Keyword(Keyword::Constructor)
                | TokenType::Keyword(Keyword::Function)
                | TokenType::Keyword(Keyword::Method)
        ) {
            self.tokens.reset_peek();

            return None;
        }

        let subroutine_dec_type = match consume!(self.tokens).ok()?.token_type {
            TokenType::Keyword(Keyword::Constructor) => SubroutineDecType::Constructor,
            TokenType::Keyword(Keyword::Function) => SubroutineDecType::Function,
            TokenType::Keyword(Keyword::Method) => SubroutineDecType::Method,
            _ => unreachable!(),
        };

        let subroutine_dec_return_type = match peek!(self.tokens).ok()?.token_type {
            TokenType::Keyword(Keyword::Void) => {
                let _ = consume!(self.tokens);

                SubroutineDecReturn::Void
            }
            _ => {
                let r#type = self.parse_type()?;

                SubroutineDecReturn::Type(r#type)
            }
        };

        let subroutine_name = consume!(self.tokens).ok()?.try_into().ok()?;

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::LeftParenthesis))
                .ok()?;
        let parameter_list = self.parse_parameeter_list()?;
        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::RightParenthesis))
                .ok()?;

        let subroutine_body = self.parse_subroutine_body().ok()?;

        Some(SubroutineDec {
            subroutine_dec_type,
            subroutine_dec_return_type,
            subroutine_name,
            parameter_list,
            subroutine_body,
        })
    }

    fn parse_type(&mut self) -> Option<Type<'de>> {
        if !matches!(
            peek!(self.tokens).ok()?.token_type,
            TokenType::Keyword(Keyword::Int)
                | TokenType::Keyword(Keyword::Char)
                | TokenType::Keyword(Keyword::Boolean)
                | TokenType::Identifier(_)
        ) {
            self.tokens.reset_peek();

            return None;
        }

        match consume!(self.tokens).ok()?.token_type {
            TokenType::Keyword(Keyword::Int) => Some(Type::Int),
            TokenType::Keyword(Keyword::Char) => Some(Type::Char),
            TokenType::Keyword(Keyword::Boolean) => Some(Type::Boolean),
            TokenType::Identifier(identifier) => Some(Type::Class { name: identifier }),
            _ => unreachable!(),
        }
    }

    fn parse_class_var_dec(&mut self) -> Option<ClassVarDec<'de>> {
        let class_var_dec_kind = match peek!(self.tokens).ok()?.token_type {
            TokenType::Keyword(Keyword::Static) => {
                let _ = consume!(self.tokens);

                ClassVarDecKind::Static
            }
            TokenType::Keyword(Keyword::Field) => {
                let _ = consume!(self.tokens);

                ClassVarDecKind::Field
            }
            _ => {
                self.tokens.reset_peek();

                return None;
            }
        };

        let class_var_dec_type = self.parse_type()?;

        let mut var_names = vec![];

        loop {
            let var_name = self.parse_identifier()?;
            var_names.push(var_name);

            if matches!(
                peek!(self.tokens).ok()?.token_type,
                TokenType::Symbol(Symbol::Comma)
            ) {
                let _ = consume!(self.tokens).ok()?;
            } else {
                self.tokens.reset_peek();

                break;
            }
        }

        let _ =
            consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Semicolon)).ok()?;

        Some(ClassVarDec {
            class_var_dec_kind,
            class_var_dec_type,
            var_names,
        })
    }

    fn parse_expression(&mut self) -> anyhow::Result<Expression<'de>> {
        let mut terms = vec![];

        let term = self.parse_term()?;
        if let Some(op) = self.parse_op() {
            let term = self.parse_term()?;

            terms.push((op, term));
        }

        Ok(Expression { term, terms })
    }

    fn parse_term(&mut self) -> anyhow::Result<Term<'de>> {
        if let Some(keyword_constant) = self.parse_keyword_constant() {
            return Ok(Term::KeywordConstant(keyword_constant));
        }

        if let Some(unary_op) = self.parse_unary_op() {
            let term = self.parse_term()?;

            return Ok(Term::UnaryOpTerm {
                unary_op,
                term: Box::new(term),
            });
        }

        let next_1 = self
            .tokens
            .peek()
            .ok_or(anyhow::anyhow!("Could not peek a token at `parse_term`"))?;
        match &next_1.token_type {
            // integerConstant | stringConstant
            TokenType::Constant(_) => {
                let constant = consume!(self.tokens)?.try_into()?;

                return Ok(Term::Constant(constant));
            }
            // '(' expression ')'
            TokenType::Symbol(Symbol::LeftParenthesis) => {
                let _ = consume_and_ensure_matches!(
                    self.tokens,
                    TokenType::Symbol(Symbol::LeftParenthesis)
                );
                let expression = self.parse_expression()?;
                let _ = consume_and_ensure_matches!(
                    self.tokens,
                    TokenType::Symbol(Symbol::RightParenthesis)
                );

                return Ok(Term::Expression(Box::new(expression)));
            }
            TokenType::Identifier(_) => {
                if let Some(next_2) = self.tokens.peek() {
                    // varName '[' expression ']'
                    match &next_2.token_type {
                        TokenType::Symbol(Symbol::LeftSquareBracket) => {
                            let var_name = consume!(self.tokens)?.try_into()?;
                            let _ = consume_and_ensure_matches!(
                                self.tokens,
                                TokenType::Symbol(Symbol::LeftSquareBracket)
                            )?;
                            let expression = self.parse_expression()?;
                            let _ = consume_and_ensure_matches!(
                                self.tokens,
                                TokenType::Symbol(Symbol::RightSquareBracket)
                            )?;

                            return Ok(Term::VarNameExpression {
                                var_name,
                                expression: Box::new(expression),
                            });
                        }
                        _ => {}
                    }
                }

                self.tokens.reset_peek();

                // subroutineCall
                if let Ok(subroutine_call) = self.parse_subroutine_call() {
                    return Ok(Term::SubroutineCall(subroutine_call));
                }

                // varName
                {
                    let var_name = consume!(self.tokens)?.try_into()?;

                    return Ok(Term::VarName(var_name));
                }
            }
            _ => {
                // keywordConstant
                if let Some(keyword_constant) = self.parse_keyword_constant() {
                    return Ok(Term::KeywordConstant(keyword_constant));
                }
                // (unaryOp term)
                if let Some(unary_op) = self.parse_unary_op() {
                    let term = self.parse_term()?;

                    return Ok(Term::UnaryOpTerm {
                        unary_op,
                        term: Box::new(term),
                    });
                }

                panic!()
            }
        };
    }

    fn parse_subroutine_call(&mut self) -> anyhow::Result<SubroutineCall<'de>> {
        let next_1 = self.tokens.peek().ok_or(anyhow::anyhow!(
            "Could not peek a token at `parse_subroutine_call`"
        ))?;
        if !matches!(next_1.token_type, TokenType::Identifier(_)) {
            self.tokens.reset_peek();

            anyhow::bail!(
                "Got a wrong token type at `parse_subroutine_call`. Not an `Identifier` <thinking>"
            );
        }

        let next_2 = self.tokens.peek().ok_or(anyhow::anyhow!(
            "Could not peek a token at `parse_subroutine_call`"
        ))?;
        match next_2.token_type {
            TokenType::Symbol(Symbol::LeftParenthesis) => {
                let subroutine_name = consume!(self.tokens)?.try_into()?;
                let _ = consume_and_ensure_matches!(
                    self.tokens,
                    TokenType::Symbol(Symbol::LeftParenthesis)
                )?;
                let expression_list =
                    if peek_matches!(self.tokens, TokenType::Symbol(Symbol::RightParenthesis)) {
                        let _ = consume_and_ensure_matches!(
                            self.tokens,
                            TokenType::Symbol(Symbol::RightParenthesis)
                        )?;

                        ExpressionList {
                            expressions: vec![],
                        }
                    } else {
                        let expression_list = self.parse_expression_list()?;

                        let _ = consume_and_ensure_matches!(
                            self.tokens,
                            TokenType::Symbol(Symbol::RightParenthesis)
                        )?;

                        expression_list
                    };

                Ok(SubroutineCall::Call {
                    subroutine_name,
                    expression_list,
                })
            }
            TokenType::Symbol(Symbol::Dot) => {
                let class_or_var_name = consume!(self.tokens)?.try_into()?;
                let _ = consume_and_ensure_matches!(self.tokens, TokenType::Symbol(Symbol::Dot))?;

                let subroutine_name = consume!(self.tokens)?.try_into()?;
                let _ = consume_and_ensure_matches!(
                    self.tokens,
                    TokenType::Symbol(Symbol::LeftParenthesis)
                )?;
                let expression_list =
                    if peek_matches!(self.tokens, TokenType::Symbol(Symbol::RightParenthesis)) {
                        let _ = consume_and_ensure_matches!(
                            self.tokens,
                            TokenType::Symbol(Symbol::RightParenthesis)
                        )?;

                        ExpressionList {
                            expressions: vec![],
                        }
                    } else {
                        let expression_list = self.parse_expression_list()?;

                        let _ = consume_and_ensure_matches!(
                            self.tokens,
                            TokenType::Symbol(Symbol::RightParenthesis)
                        )?;

                        expression_list
                    };

                Ok(SubroutineCall::ClassCall {
                    class_or_var_name,
                    subroutine_name,
                    expression_list,
                })
            }
            _ => {
                self.tokens.reset_peek();

                anyhow::bail!(
                    "Got a wrong token type at `parse_subroutine_call`. Neither `Symbol::LeftParenthesis` nor `Symbol::Dot` <thinking>"
                );
            }
        }
    }

    fn parse_expression_list(&mut self) -> anyhow::Result<ExpressionList<'de>> {
        let mut expressions = vec![];

        while let Ok(expression) = self.parse_expression() {
            expressions.push(expression);

            if peek_matches!(self.tokens, TokenType::Symbol(Symbol::Comma)) {
                let _ = consume!(self.tokens);
            } else {
                break;
            }
        }

        Ok(ExpressionList { expressions })
    }

    fn parse_identifier(&mut self) -> Option<Identifier<'de>> {
        match &peek!(self.tokens).ok()?.token_type {
            TokenType::Identifier(_) => {
                let token = consume!(self.tokens).ok()?;
                let identifier = token.try_into().ok()?;

                Some(identifier)
            }
            _ => {
                self.tokens.reset_peek();

                None
            }
        }
    }

    fn parse_op(&mut self) -> Option<Op> {
        let result = match &peek!(self.tokens).ok()?.token_type {
            TokenType::Symbol(symbol) => match symbol {
                Symbol::Plus => Some(Op::Plus),
                Symbol::Minus => Some(Op::Minus),
                Symbol::Asterisk => Some(Op::Asterisk),
                Symbol::Slash => Some(Op::Slash),
                Symbol::Ampersand => Some(Op::Ampersand),
                Symbol::Pipe => Some(Op::Pipe),
                Symbol::LessThan => Some(Op::LessThan),
                Symbol::GreaterThan => Some(Op::GreaterThan),
                Symbol::Equal => Some(Op::Equal),
                _ => {
                    self.tokens.reset_peek();

                    None
                }
            },
            _ => {
                self.tokens.reset_peek();

                None
            }
        };
        if result.is_some() {
            let _ = consume!(self.tokens);
        }

        result
    }

    fn parse_unary_op(&mut self) -> Option<UnaryOp> {
        let result = match &peek!(self.tokens).ok()?.token_type {
            TokenType::Symbol(symbol) => match symbol {
                Symbol::Minus => Some(UnaryOp::Minus),
                Symbol::Tilde => Some(UnaryOp::Tilde),
                _ => {
                    self.tokens.reset_peek();
                    None
                }
            },
            _ => {
                self.tokens.reset_peek();
                None
            }
        };
        if result.is_some() {
            let _ = consume!(self.tokens);
        }

        result
    }

    fn parse_keyword_constant(&mut self) -> Option<KeywordConstant> {
        let result = match &peek!(self.tokens).ok()?.token_type {
            TokenType::Keyword(keyword) => match keyword {
                Keyword::True => Some(KeywordConstant::True),
                Keyword::False => Some(KeywordConstant::False),
                Keyword::Null => Some(KeywordConstant::Null),
                Keyword::This => Some(KeywordConstant::This),
                _ => {
                    self.tokens.reset_peek();
                    None
                }
            },
            _ => {
                self.tokens.reset_peek();

                None
            }
        };
        if result.is_some() {
            let _ = consume!(self.tokens);
        }

        result
    }
}

impl<'de, I> Iterator for Parser<'de, I>
where
    I: Iterator<Item = Token<'de>>,
{
    type Item = anyhow::Result<Class<'de>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse()
    }
}

impl<'de> TryFrom<Token<'de>> for Identifier<'de> {
    type Error = anyhow::Error;

    fn try_from(token: Token<'de>) -> Result<Self, Self::Error> {
        let Token {
            token_type: TokenType::Identifier(identifier),
            ..
        } = token
        else {
            anyhow::bail!("Error: Could not conver token `{token:?}` into an identifier")
        };

        Ok(identifier)
    }
}

impl<'de> TryFrom<Token<'de>> for Constant<'de> {
    type Error = anyhow::Error;

    fn try_from(token: Token<'de>) -> Result<Self, Self::Error> {
        let Token {
            token_type: TokenType::Constant(constant),
            ..
        } = token
        else {
            anyhow::bail!("Error: Could not conver token `{token:?}` into a constant")
        };

        Ok(constant)
    }
}
