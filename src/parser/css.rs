#![allow(len_zero)]

use std::str;
use std::io::Read;

use url::{Url, UrlParser};

use parser::{rewrite_url, ParserError};
use tasks::Resource;


pub fn explore_inline_css(mut data: &[u8], base: &Url) -> Result<Vec<Url>, ParserError> {
    let mut input = String::new();
    try!(data.read_to_string(&mut input));

    trace!("Processing inline CSS with base {}", base);

    let (_, urls) = try!(process_css(&input, base));

    Ok(urls)
}


pub fn explore_css(resource: &mut Resource) -> Result<(Vec<u8>, Vec<Url>), ParserError> {
    let base = resource.get_response_url().clone();
    let input = try!(str::from_utf8(resource.read_response()));
    trace!("Processing CSS with base {}", base);

    let (css, urls) = try!(process_css(input, &base));

    Ok((css.into_bytes(), urls))
}


peg! lexer(r#"
    use super::Token;

    #[pub]
    css -> Vec<Token<'input>>
        = t:(uri / import / ignored)+ {
            t.into_iter()
                .filter_map(|x| x)
                .collect()
        }

    import -> Option<Token<'input>>
        = "@import" ws+ s:(string / uri) {
            if let Some(url) = s {
                Some(Token::Import(Box::new(url)))
            } else {
                None
            }
        }

    uri -> Option<Token<'input>>
        = "url" ws* "(" ws* s:string ws* ")" {
            if let Some(url) = s {
                Some(Token::Uri(Box::new(url)))
            } else {
                None
            }
        }

    string -> Option<Token<'input>>
        = ("\"" [^"]* "\"" { Some(Token::String(match_str)) })
          / ("'" [^"]* "'" { Some(Token::String(match_str)) })

    ignored -> Option<Token<'input>>
        = . { None }

    ws = [ \t\n]
"#);


#[derive(Debug)]
pub enum Token<'a> {
    Uri(Box<Token<'a>>),
    Import(Box<Token<'a>>),

    // Only used in Token::Uri/Token::Import
    String(&'a str),
}


fn process_css(input: &str, base: &Url) -> Result<(String, Vec<Url>), ParserError> {
    let mut url_parser = UrlParser::new();
    url_parser.base_url(base);

    let mut urls = Vec::new();
    let mut css = input.to_owned();

    if let Ok(tokens) = lexer::css(input) {
        trace!("tokens: {:?}", tokens);

        // FIXME: Calling String::replace over and over again is probably very
        // inefficient, especially as the offset of the replacement is known!
        for token in tokens {
            match token {
                Token::Uri(box Token::String(s)) |
                Token::Import(box Token::Uri(box Token::String(s))) |
                Token::Import(box Token::String(s)) => {
                    let s = &s[1..s.len() - 1];  // Remove quotes

                    if let Ok(url) = url_parser.parse(s) {
                        css = css.replace(s, &rewrite_url(&url));
                        urls.push(url);
                    } else {
                        warn!("Failed to resolve <base href=\"{}\"> with respect to {}",
                              s,
                              base);
                    }
                }
                _ => return Err(ParserError::MalformedCSS),
            }
        }
    } else {
        // Malformed CSS
        return Err(ParserError::MalformedCSS);
    }

    Ok((css, urls))
}


// FIXME: Tests for @import
// FIXME: Tests for url('...')

#[cfg(test)]
mod test {
    use url::Url;

    use parser::rewrite_url;

    use super::process_css;

    #[test]
    fn test_simple_parsing() {
        let source = "body { display: none; background: url(\"http://example.com/bg.png\"); }";
        let (serialized, urls) = process_css(source, &Url::parse("http://example.com").unwrap())
                                     .unwrap();

        assert_eq!(serialized,
                   format!("body {{ display: none; background: url(\"{}\"); }}",
                           rewrite_url(&Url::parse("http://example.com/bg.png").unwrap())));
        assert_eq!(urls, vec![Url::parse("http://example.com/bg.png").unwrap()]);
    }
}
