use std::mem;

use html5ever::{self, one_input};
use html5ever::rcdom::{Handle, RcDom};
use hyper::client::Response;
use phf;
use tendril::{ByteTendril, ReadExt};
use url::Url;

use Queues;
use parser::css;
use parser::resolve_rel_url;


macro_rules! html_rule {
    (src=[$($src:ident),*]) => {
        HTMLRule {
            required: &[],
            sources: &[$(stringify!($src)),*],
        }
    };
    (req=[$($name:ident = $value:expr),*], src=[$($src:ident),*]) => {
        HTMLRule {
            required: &[$((stringify!($name), $value)),*],
            sources: &[$(stringify!($src)),*],
        }
    };
    (@assign ($name:ident = $value:expr)) => {
        (stringify!($name), $value)
    };
}



#[derive(Debug)]
struct HTMLRule {
    /// Which attributes are required for the rule to match (logical OR)
    required: &'static [(&'static str, &'static str)],

    /// The source of the explored resource
    sources: &'static [&'static str],
}


static HTML_RULES: phf::Map<&'static str, HTMLRule> = phf_map! {
    "link" => html_rule!(req=[rel="stylesheet", rel="shortcut icon"],
                         src=[href]),
    "applet" => html_rule!(src=[code]),
    "bgsound" => html_rule!(src=[src]),
    "body" => html_rule!(src=[background]),
    "img" => html_rule!(src=[href, lowsrc, src]),
    "input" => html_rule!(src=[src]),
    "layer" => html_rule!(src=[src]),
    "object" => html_rule!(src=[data]),
    "overlay" => html_rule!(src=[src]),
    "script" => html_rule!(src=[src]),
    "table" => html_rule!(src=[background]),
    "th" => html_rule!(src=[background]),
    "td" => html_rule!(src=[background]),
    "embed" => html_rule!(src=[src, href]),
    "frame" => html_rule!(src=[src]),
    "iframe" => html_rule!(src=[src]),
    "audio" => html_rule!(src=[src]),
    "video" => html_rule!(src=[src]),
    "source" => html_rule!(src=[src]),
};


pub fn explore_html(data: &mut Response, queues: Queues) -> Vec<Url> {
    // FIXME: Figure out error handling. What does Servo do?
    let mut input = ByteTendril::new();
    data.read_to_tendril(&mut input).unwrap();
    let input = input.try_reinterpret().unwrap();
    let dom: RcDom = html5ever::parse(one_input(input), Default::default());

    let explorer = HTMLExplorer::new(dom, &data.url, queues);
    explorer.explore()
}


struct HTMLExplorer<'a> {
    dom: RcDom,
    request_url: &'a Url,
    html_base: Option<Url>,
    explored_resources: Vec<Url>,
    queues: Queues,
}

impl<'a> HTMLExplorer<'a> {
    fn new(dom: RcDom, request_url: &Url, queues: Queues) -> HTMLExplorer {
        HTMLExplorer {
            dom: dom,
            request_url: request_url,
            html_base: None,
            explored_resources: Vec::new(),
            queues: queues,
        }
    }

    fn explore(mut self) -> Vec<Url> {
        let root = self.dom.document.clone();
        self.walk_dom(root);

        // FIXME: Store rewritten HTML

        self.explored_resources
    }

    // FIXME: This is way too complex...
    fn walk_dom(&mut self, handle: Handle) {
        let mut node = handle.borrow_mut();

        if let html5ever::rcdom::Element(ref name, _, ref mut attrs) = node.node {
            trace!("Processing HTML element name={:?} attrs={:?}", name, attrs);

            // Handle <base> tags
            // See HTML spec 4.2.3
            // - A base element, if it has an href attribute, must come before any other elements
            //   in the tree that have attributes defined as taking URLs
            // - If there are multiple base elements with href attributes, all but the first are
            //   ignored.
            if self.html_base.is_none() && &*name.local == "base" {
                // Search for "href" attribute and set html_base if found
                if let Some(attr) = attrs.iter()
                                         .find(|attr| &*(attr.name.local) == "href") {
                    let base = self.resolve_url(&*attr.value);
                    if base.is_none() {
                        warn!("Failed to resolve <base href=\"{}\"> with respect to {}",
                              &*attr.value,
                              self.get_base())
                    }

                    mem::replace(&mut self.html_base, base);
                }
            }

            // Handle inline styles
            for attr in attrs.iter()
                             .filter(|attr| &*attr.name.local == "style") {
                let new_urls = css::explore_css(&mut (*attr.value).as_bytes(),
                                                self.get_base(),
                                                self.queues.clone());
                self.explored_resources.extend(new_urls.into_iter());
            }

            // Handle HTML exploration rules
            trace!("HTML rule: {:?}", HTML_RULES.get(&*name.local));

            if let Some(rule) = HTML_RULES.get(&*name.local) {
                // If there are no required attrs, we're done:
                let mut required_attr_found = rule.required.len() == 0;
                let mut source: Option<String> = None;

                // Process attributes
                for attr in attrs.iter_mut() {
                    if rule.sources.contains(&&*attr.name.local) {
                        source = Some((&*attr.value).into());
                        // FIXME: Rewrite URL
                        // mem::replace(&mut attr.value, StrTendril::from_slice(""));
                    }

                    // Make sure one of the required attributes is there
                    required_attr_found |= rule.required
                                               .iter()
                                               .any(|&(required_attr, required_value)| {
                                                   &*attr.name.local == required_attr &&
                                                   &*attr.value == required_value
                                               });

                    if source.is_some() && required_attr_found {
                        break;  // Skip other attributes
                    }
                }

                if let Some(source) = source {
                    if required_attr_found {
                        debug!("New source from <{}>: {:?}", name.local, source);

                        match self.resolve_url(&source) {
                            Some(url) => self.explored_resources.push(url),
                            None => {
                                warn!("Failed to resolve URL {} with respect to {}",
                                      source,
                                      self.get_base())
                            }
                        }
                    } else {
                        trace!("Skipping element since required attribtes are not found")
                    }
                }
            }
        }

        for child in &node.children {
            self.walk_dom(child.clone());
        }
    }

    fn resolve_url(&self, url: &str) -> Option<Url> {
        resolve_rel_url(self.get_base(), url)
    }

    fn get_base(&self) -> &Url {
        self.html_base.as_ref().unwrap_or(self.request_url)
    }
}


// FIXME: Test for <base> handling
// FIXME: Test for lowercase/uppercase/mixed case tag and attr handling
