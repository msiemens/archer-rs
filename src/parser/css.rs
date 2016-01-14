use std::io::Read;

use cssparser::{Parser, Token};
use url::Url;

use Queues;
use parser::resolve_rel_url;


// struct RuleParser;
//
// impl QualifiedRuleParser for RuleParser {
//    type Prelude = ();
//    type QualifiedRule = ();
// }
//
// impl AtRuleParser for RuleParser {
//    type Prelude = ();
//    type AtRule = ();
// }


pub fn explore_css<R: Read>(data: &mut R, base: &Url, _: Queues) -> Vec<Url> {
    let mut input = String::new();
    data.read_to_string(&mut input).unwrap();  // FIXME: Error handling

    let mut parser = Parser::new(&input);

    // TODO:
    // - Skip until AtKeyword or UnquotedUrl, expect_url_or_string
    // - Add URL to download list
    // - Store rewritten CSS

    parse_css(&mut parser, base).unwrap()
}


// fn parse_css(parser: Parser) -> Result<Vec<Url>, ()> {
fn parse_css<'i, 't>(parser: &mut Parser<'i, 't>, base: &Url) -> Result<Vec<Url>, ()> {
    let mut urls = Vec::new();

    while !parser.is_exhausted() {
        let token = match parser.next() {
            Ok(token) => token,
            Err(..) => continue,
        };

        match token {
            Token::Function(..) |
            Token::ParenthesisBlock |
            Token::CurlyBracketBlock |
            Token::SquareBracketBlock => {
                parser.parse_nested_block(|parser| {
                          parse_css(parser, base).map(|new_urls| urls.extend(new_urls.into_iter()))
                      })
                      .unwrap();
            }
            _ => {}
        }

        trace!("Processing CSS token {:?}", token);

        // FIXME: This parser skips over all blocks!

        let url = match token {
            Token::UnquotedUrl(url) => {
                debug!("Found URL in CSS: {}", url);

                url
            }
            Token::AtKeyword(ref kw) if kw == "import" => {
                match parser.expect_url_or_string() {
                    Ok(url) => {
                        debug!("Found @import in CSS: {}", url);
                        url
                    }
                    Err(..) => continue,
                }
            }
            _ => continue,
        };

        match resolve_rel_url(base, &url) {
            Some(url) => urls.push(url),
            None => {
                warn!("Failed to resolve <base href=\"{}\"> with respect to {}",
                      url,
                      base)
            }
        }
    }

    Ok(urls)
}


// FIXME: Tests for @import
// FIXME: Tests for url('...')
