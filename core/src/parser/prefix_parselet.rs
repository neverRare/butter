use crate::lexer::Keyword;
use crate::lexer::Token;
use crate::parser::Node;
use crate::parser::NodeType;
use crate::parser::SpanToken;
use std::iter::Peekable;
use util::tree_vec::Tree;
use util::parser::Parser;

pub(super) fn keyword_literal<'a>(
    prefix: SpanToken<'a>,
    _: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    let SpanToken {span, token: prefix} = prefix;
    if let Token::Keyword(keyword) = prefix {
        let node = match keyword {
            Keyword::Abort => NodeType::Abort,
            Keyword::True => NodeType::True,
            Keyword::False => NodeType::False,
            Keyword::Null => NodeType::Null,
            _ => return None,
        };
        Some(Tree::new(Node {
            span,
            node,
            unpack: false,
        }))
    } else {
        None
    }
}
pub(super) fn clone<'a>(
    prefix: SpanToken<'a>,
    tokens: &mut Peekable<impl Iterator<Item = SpanToken<'a>>>,
) -> Option<Tree<Node<'a>>> {
    let SpanToken {span: src, token: prefix} = prefix;
    if let Token::Keyword(Keyword::Clone) = prefix {
        Some(Tree {
            content: todo!(),
            children: Node::partial_parse(tokens, 90).into_tree_vec(),
        })
    } else {
        None
    }
}
