use std::io;
use std::str::Utf8Error;

use crypto::digest::Digest;
use crypto::md5::Md5;
use hyper::header::ContentType;
use mime::{self, Mime};
use tendril::ByteTendril;
use url::Url;

use tasks::Resource;


mod css;
mod html;




quick_error! {
    #[derive(Debug)]
    pub enum ParserError {
        Io(err: io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
        Encoding {
            from(ByteTendril)
            from(Utf8Error)
            description("encoding error")
            display("Encoding error")
        }
        MalformedCSS {
            description("malformed CSS")
            display("Malformed CSS found")
        }
    }
}


fn rewrite_url(url: &Url) -> String {
    let filename = url.path().and_then(|parts| parts.last());
    let extension = filename.and_then(|f| {
        if f.contains('.') {
            f.rsplitn(2, '.').next()
        } else {
            None
        }
    });

    let mut md5 = Md5::new();
    md5.input_str(&url.serialize());
    let mut rewritten = md5.result_str();

    if let Some(ref ext) = extension {
        rewritten.push('.');
        rewritten.push_str(ext);
    }

    debug!("Rewriting {} to {}", url, rewritten);

    rewritten
}


pub fn task_parse_resource(resource: &mut Resource) -> Vec<Url> {
    debug!("Parse Resource");

    let content_type = resource.get_content_type();

    let res = match content_type {
        Some(ContentType(Mime(mime::TopLevel::Text, mime::SubLevel::Html, _))) => {
            html::explore_html(resource)
        }
        Some(ContentType(Mime(mime::TopLevel::Text, mime::SubLevel::Css, _))) => {
            css::explore_css(resource)
        }
        _ => {
            // No more exploring, just store in DB
            Ok((resource.read_response().iter().cloned().collect(), Vec::new()))
        }
    };

    match res {
        Ok((contents, urls)) => {
            resource.parsed(contents, content_type.map(|t: ContentType| t.0));

            urls
        }
        Err(e) => {
            warn!("Error while parsing {}: {}", resource.get_url(), e);
            resource.failed();

            Vec::new()
        }
    }
}
