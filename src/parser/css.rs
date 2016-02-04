use std::borrow::Cow;
use std::io::Read;

use cssparser::{Parser, Token, ToCss};
use url::Url;

use Queues;
use parser::{resolve_rel_url, rewrite_url};
use model::WebsiteID;


pub fn explore_css<R: Read>(wid: WebsiteID, data: &mut R, base: &Url, _: Queues) -> Vec<Url> {
    trace!("Processing CSS with base {}", base);

    let mut input = String::new();
    data.read_to_string(&mut input).unwrap();  // FIXME: Error handling

    let mut parser = Parser::new(&input);

    let (urls, tokens) = parse_css(&mut parser, base).unwrap();

    // FIXME: Emit open and closing braces
    // FIXME: Get information if it's a resouce or if it's inline
    // FIXME: Emit store event
    trace!("CSS tokens: {}",
           tokens.iter().map(|tok| tok.to_css_string()).collect::<Vec<String>>().join(" "));

    urls
}


fn parse_css<'i, 't>(parser: &mut Parser<'i, 't>,
                     base: &Url)
                     -> Result<(Vec<Url>, Vec<Token<'i>>), ()> {
    let mut urls = Vec::new();
    let mut tokens = Vec::new();

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
                          parse_css(parser, base).map(|(new_urls, new_tokens)| {
                              urls.extend(new_urls.into_iter());
                              tokens.extend(new_tokens.into_iter())
                          })
                      })
                      .unwrap();
            }
            _ => {}
        }

        let mut add_tokens_cb: Option<Box<FnMut(String)>> = None;

        let url = match token {
            Token::UnquotedUrl(url) => {
                debug!("Found URL in CSS: {}", url);

                add_tokens_cb = Some(Box::new(|rewritten| {
                    tokens.push(Token::UnquotedUrl(Cow::Owned(rewritten)));
                }));

                url
            }
            Token::AtKeyword(ref kw) if kw == "import" => {
                match parser.expect_url_or_string() {
                    Ok(url) => {
                        debug!("Found @import in CSS: {}", url);

                        add_tokens_cb = Some(Box::new(|rewritten| {
                            tokens.push(Token::AtKeyword(Cow::Borrowed("import")));
                            tokens.push(Token::UnquotedUrl(Cow::Owned(rewritten)))
                        }));

                        url
                    }
                    Err(..) => {
                        tokens.push(Token::AtKeyword(kw.clone()));
                        continue;
                    }
                }
            }
            _ => {
                tokens.push(token);
                continue;
            }
        };

        match resolve_rel_url(base, &url) {
            Some(url) => {
                if let Some(mut cb) = add_tokens_cb {
                    cb(rewrite_url(&url))
                };

                urls.push(url);
            }
            None => {
                warn!("Failed to resolve <base href=\"{}\"> with respect to {}",
                      url,
                      base);
            }
        }
    }

    Ok((urls, tokens))
}


// FIXME: Tests for @import
// FIXME: Tests for url('...')
