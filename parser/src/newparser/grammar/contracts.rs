use super::functions::parse_fn_def;
use super::types::{
    parse_event_def,
    parse_field_def,
    parse_opt_qualifier,
};

use crate::ast::{
    ConstQualifier,
    ContractStmt,
    ModuleStmt,
    PubQualifier,
};
use crate::lexer::{
    Token,
    TokenKind,
};
use crate::newparser::{
    Label,
    ParseResult,
    Parser,
};
use crate::node::{
    Node,
    Span,
};

// Rule: all "statement" level parse functions consume their trailing
// newline(s), either directly or via a function they call.
// This is required to parse an `if` block, because we need to peek past the
// trailing newlines to check whether it's followed by an `else` block, and is
// done for all statements for consistency.

pub fn parse_contract_def<'a>(par: &mut Parser<'a>) -> ParseResult<Node<ModuleStmt>> {
    use TokenKind::*;
    let contract_tok = par.assert(Contract);

    // contract Foo:
    //   x: map<address, u256>
    //   pub y: u8
    //   const z: u256 = 10
    //
    //   event Sent:
    //     idx sender: address
    //     val: u256
    //
    //   pub def foo() -> address:
    //     return abc
    //

    let contract_name = par.expect_with_notes(
        Name,
        "failed to parse contract definition",
        || vec!["Note: `contract` must be followed by a name, which must start with a letter and contain only letters, numbers, or underscores".into()],
    )?;

    let header_span = contract_tok.span + contract_name.span;
    par.enter_block(header_span, "contract definition")?;

    let mut fields = vec![];
    let mut defs = vec![];

    loop {
        let pub_qual = parse_opt_qualifier(par, TokenKind::Pub, PubQualifier {});
        let const_qual = parse_opt_qualifier(par, TokenKind::Const, ConstQualifier {});
        if pub_qual.is_none() && const_qual.is_some() && par.peek() == Some(Pub) {
            let tok = par.next()?;
            par.error(
                const_qual.as_ref().unwrap().span + tok.span,
                "`const pub` should be written `pub const`",
            );
        }

        match par.peek() {
            Some(TokenKind::Name) => {
                let field = parse_field_def(par, pub_qual, const_qual)?;
                if !defs.is_empty() {
                    par.error(field.span, "contract field definitions must come before any function or event definitions");
                }
                fields.push(field);
            }
            Some(TokenKind::Def) => {
                if const_qual.is_some() {
                    par.error(
                        const_qual.unwrap().span,
                        "`const` qualifier can't be used with function definitions",
                    );
                }
                defs.push(parse_fn_def(par, pub_qual)?);
            }
            Some(TokenKind::Event) => {
                if pub_qual.is_some() {
                    par.error(
                        pub_qual.unwrap().span,
                        "`pub` qualifier can't be used with event definitions",
                    );
                }
                if const_qual.is_some() {
                    par.error(
                        const_qual.unwrap().span,
                        "`const` qualifier can't be used with event definitions",
                    );
                }
                defs.push(parse_event_def(par)?);
            }
            Some(TokenKind::Dedent) => {
                par.next()?;
                break;
            }
            None => break,
            Some(_) => {
                let tok = par.next()?;
                par.unexpected_token_error(
                    tok.span,
                    "failed to parse contract definition body",
                    vec![],
                );
                return Err(());
            }
        };
    }

    if fields.is_empty() && defs.is_empty() {
        par.error(header_span, "empty contract definition");
    }
    let span = header_span + fields.last() + defs.last();
    Ok(Node::new(
        ModuleStmt::ContractDef {
            name: Node::new(contract_name.text.to_string(), contract_name.span),
            fields,
            body: defs,
        },
        span,
    ))
}
