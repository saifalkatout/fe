use fxhash::FxHashSet;

use crate::{ParseError, SyntaxKind, TextRange};

use self::token_stream::{BackTrackableTokenStream, SyntaxToken, TokenStream};

pub mod token_stream;

mod attr;
mod func;
mod item;

/// Parser to build a rowan syntax tree.
pub struct Parser<S: TokenStream> {
    /// Token stream to parse.
    stream: BackTrackableTokenStream<S>,

    builder: rowan::GreenNodeBuilder<'static>,
    scopes: Vec<Box<dyn ParsingScope>>,
    errors: Vec<ParseError>,

    current_pos: rowan::TextSize,
}

impl<S: TokenStream> Parser<S> {
    /// Returns the current token of the parser.
    pub fn current_token(&mut self) -> Option<&S::Token> {
        self.stream.peek()
    }

    /// Returns the current token kind of the parser.
    pub fn current_kind(&mut self) -> Option<SyntaxKind> {
        self.current_token().map(|token| token.syntax_kind())
    }

    pub fn parse<T>(&mut self, mut scope: T, checkpoint: Option<rowan::Checkpoint>)
    where
        T: Parse + 'static,
    {
        let checkpoint = self.enter(scope.clone(), checkpoint);
        scope.parse(self);
        self.leave(checkpoint);
    }

    /// Mark the current branch as a checkpoint.
    /// The checked branch is wrapped up later when [`parse]` is
    /// called with the `checkpoint`.
    pub fn checkpoint(&mut self) -> rowan::Checkpoint {
        self.builder.checkpoint()
    }

    pub fn error_and_recover(&mut self, msg: &str, checkpoint: Option<rowan::Checkpoint>) {
        let err_scope = self.error(msg);
        let checkpoint = self.enter(err_scope, checkpoint);
        self.recover();
        self.leave(checkpoint);
    }

    /// Bumps the current token and trailing trivias and adds them to the
    /// current branch.
    pub fn bump(&mut self) {
        self.bump_raw();
        self.bump_trivias(false);
    }

    /// Bumps the current token adds it to the current branch.
    pub fn bump_raw(&mut self) {
        let tok = self.stream.next().unwrap();
        self.current_pos += rowan::TextSize::of(tok.text());
        self.builder.token(tok.syntax_kind().into(), tok.text());
    }

    /// Bumps consecutive trivia tokens.
    /// If `bump_newlines` is true, newlines are also bumped.
    pub fn bump_trivias(&mut self, skip_newlines: bool) {
        while let Some(tok) = self.current_token() {
            let kind = tok.syntax_kind();
            if kind.is_trivia() || (skip_newlines && kind == SyntaxKind::Newline) {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Bump consecutive newlines.
    pub fn bump_newlines(&mut self) {
        while let Some(tok) = self.current_token() {
            if tok.syntax_kind() == SyntaxKind::Newline {
                self.bump();
            } else {
                break;
            }
        }
    }

    /// Proceeds the parser to the recovery token of the current scope.
    pub fn recover(&mut self) {
        let mut scope_index = self.scopes.len() - 1;
        // Finds the nearest scope that has its own recovery set.
        loop {
            if self.scopes[scope_index].recovery_method() != &RecoveryMethod::Inheritance
                || scope_index == 0
            {
                break;
            } else {
                scope_index -= 1;
            }
        }

        while let Some(tok) = self.stream.peek() {
            let syntax_kind = tok.syntax_kind();
            if self.scopes[scope_index]
                .recovery_method()
                .contains(syntax_kind)
            {
                break;
            } else {
                self.bump();
            }
        }
    }

    /// Add the `msg` to the error list.
    fn error(&mut self, msg: &str) -> ErrorScope {
        let start = self.current_pos;
        let end = if let Some(current_token) = self.current_token() {
            start + current_token.text_size()
        } else {
            start
        };
        let range = TextRange::new(start, end);

        self.errors.push(ParseError {
            range,
            msg: msg.to_string(),
        });
        ErrorScope::default()
    }

    fn enter<T>(&mut self, scope: T, checkpoint: Option<rowan::Checkpoint>) -> rowan::Checkpoint
    where
        T: ParsingScope + 'static,
    {
        self.scopes.push(Box::new(scope));
        checkpoint.unwrap_or_else(|| self.checkpoint())
    }

    fn leave(&mut self, checkpoint: rowan::Checkpoint) {
        let scope = self.scopes.pop().unwrap();
        self.builder
            .start_node_at(checkpoint, scope.syntax_kind().into());
    }
}

/// The current scope of parsing.
pub trait ParsingScope {
    /// Returns the recovery method of the current scope.
    fn recovery_method(&self) -> &RecoveryMethod;

    fn syntax_kind(&self) -> SyntaxKind;
}

pub trait Parse: ParsingScope + Clone {
    fn parse<S: TokenStream>(&mut self, parser: &mut Parser<S>);
}

define_scope! {
    ErrorScope,
    Error,
    Inheritance
}

/// Represents the recovery method of the current scope.
#[derive(PartialEq, Eq)]
pub enum RecoveryMethod {
    /// Uses the recovery method of the parent scope.
    Inheritance,

    /// The scope has its own recovery set.
    RecoverySet(FxHashSet<SyntaxKind>),
}

impl RecoveryMethod {
    /// Returns `true` if the recovery set contains the given syntax kind.
    fn contains(&self, syntax_kind: SyntaxKind) -> bool {
        match self {
            RecoveryMethod::Inheritance => false,
            RecoveryMethod::RecoverySet(set) => set.contains(&syntax_kind),
        }
    }
}

trait TextSize {
    fn text_size(&self) -> rowan::TextSize;
}

impl<T> TextSize for T
where
    T: SyntaxToken,
{
    fn text_size(&self) -> rowan::TextSize {
        rowan::TextSize::of(self.text())
    }
}

macro_rules! define_scope {
    ($scope_name: ident, $kind: path ,Inheritance) => {
        #[derive(Default,Debug, Clone, Copy)]
        pub(crate) struct $scope_name {}

        impl crate::parser::ParsingScope for $scope_name {
            fn recovery_method(&self) -> &crate::parser::RecoveryMethod {
                lazy_static::lazy_static! {
                    pub(super) static ref RECOVERY_METHOD: crate::parser::RecoveryMethod = {
                        crate::parser::RecoveryMethod::Inheritance
                    };
                }

                &RECOVERY_METHOD
            }

            fn syntax_kind(&self) -> crate::SyntaxKind {
                use crate::SyntaxKind::*;
                $kind
            }
        }
    };

    ($scope_name: ident, $kind: path, RecoverySet($($recoveries: path), *)) => {
        #[derive(Default, Debug, Clone, Copy)]
        pub(crate) struct $scope_name {}

        impl crate::parser::ParsingScope for $scope_name {
            fn recovery_method(&self) -> &crate::parser::RecoveryMethod {
                lazy_static::lazy_static! {
                    pub(super) static ref RECOVERY_METHOD: crate::parser::RecoveryMethod = {
                        use crate::SyntaxKind::*;
                        let set: fxhash::FxHashSet<crate::SyntaxKind> = vec![
                            $($recoveries), *
                        ].into_iter().map(|kind| kind.into()).collect();

                        crate::parser::RecoveryMethod::RecoverySet(set)
                    };
                }

                &RECOVERY_METHOD
            }

            fn syntax_kind(&self) -> crate::SyntaxKind {
                use crate::SyntaxKind::*;
                $kind
            }
        }
    };
}

use define_scope;