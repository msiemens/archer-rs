use std::mem;

use html5ever::{parse_document, serialize, Attribute};
use html5ever::rcdom::{Element, Handle, RcDom};
use phf;
use tendril::TendrilSink;
use url::{Url, UrlParser};

use parser::{css, rewrite_url, ParserError};
use tasks::Resource;


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
    /// Which attributes+values are required for the rule to match (logical OR)
    /// E.g. `rel="stylesheet" OR rel="shortcut icon"`
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


pub fn explore_html(resource: &mut Resource) -> Result<(Vec<u8>, Vec<Url>), ParserError> {
    let dom = parse_document(RcDom::default(), Default::default())
                  .from_utf8()
                  .read_from(&mut resource.read_response())
                  .unwrap();

    let explorer = HTMLExplorer::new(dom, resource);
    explorer.explore()
}


struct HTMLExplorer<'a> {
    resource: &'a mut Resource,
    dom: RcDom,
    html_base: Option<Url>,
    explored_resources: Vec<Url>,
}

impl<'a> HTMLExplorer<'a> {
    fn new(dom: RcDom, resource: &'a mut Resource) -> Self {
        HTMLExplorer {
            resource: resource,
            dom: dom,
            html_base: None,
            explored_resources: Vec::new(),
        }
    }

    fn explore(mut self) -> Result<(Vec<u8>, Vec<Url>), ParserError> {
        let root = self.dom.document.clone();
        self.walk_dom(root);

        // FIXME: Store rewritten HTML
        let mut contents = Vec::new();
        try!(serialize(&mut contents, &self.dom.document, Default::default()));

        Ok((contents, self.explored_resources))
    }

    fn walk_dom(&mut self, handle: Handle) {
        let mut node = handle.borrow_mut();

        if let Element(ref name, _, ref mut attrs) = node.node {
            trace!("Processing HTML element name={:?} attrs={:?}", name, attrs);

            if self.html_base.is_none() && &*name.local == "base" {
                self.handle_base_tag(attrs);
            }

            // Handle inline styles
            for attr in attrs.iter()
                             .filter(|attr| &*attr.name.local == "style") {
                self.handle_inline_styles(attr);
            }

            // Handle HTML exploration rules
            trace!("HTML rule: {:?}", HTML_RULES.get(&*name.local));

            if let Some(rule) = HTML_RULES.get(&*name.local) {
                self.handle_html_rule(rule, &*name.local, attrs);
            }
        }

        for child in &node.children {
            self.walk_dom(child.clone());
        }
    }

    /// Handle <base> tags
    ///
    /// See HTML spec 4.2.3
    /// - A base element, if it has an href attribute, must come before any other elements
    ///   in the tree that have attributes defined as taking URLs
    /// - If there are multiple base elements with href attributes, all but the first are
    ///   ignored.
    fn handle_base_tag(&mut self, attrs: &[Attribute]) {
        // Search for "href" attribute and set html_base if found

        // FIXME: What happens if a fully qualified base has already been set?
        if let Some(attr) = attrs.iter()
                                 .find(|attr| &*(attr.name.local) == "href") {
            let base = match UrlParser::new().base_url(self.get_base()).parse(&*attr.value) {
                Ok(base) => base,
                Err(_) => {
                    warn!("Failed to resolve <base href=\"{}\"> with respect to {}",
                          &*attr.value,
                          self.get_base());
                    return;
                }
            };

            // FIXME: Rewrite URL. What directory to use?

            mem::replace(&mut self.html_base, Some(base));
        }
    }

    fn handle_inline_styles(&mut self, attr: &Attribute) {
        let new_urls = match css::explore_inline_css(&(*attr.value).as_bytes(), self.get_base()) {
            Ok(urls) => urls,
            Err(e) => {
                warn!("Error while parsing inline CSS in {}: {}",
                      self.resource.get_url(),
                      e);
                return;
            }
        };
        self.explored_resources.extend(new_urls.into_iter());
    }

    fn handle_html_rule(&mut self, rule: &HTMLRule, name: &str, attrs: &mut [Attribute]) {
        // If there are no required attrs, we're done:
        let mut required_attr_found = rule.required.len() == 0;
        let mut source_idx: Option<usize> = None;

        // Process attributes
        for (idx, attr) in attrs.iter().enumerate() {
            let attr_name = &*attr.name.local;
            let attr_value = &*attr.value;

            if rule.sources.contains(&attr_name) {
                source_idx = Some(idx);
            }

            // Make sure one of the required attributes is there
            required_attr_found |= rule.required
                                       .iter()
                                       .any(|&(required_attr, required_value)| {
                                           attr_name == required_attr &&
                                           attr_value == required_value
                                       });

            if source_idx.is_some() && required_attr_found {
                break;  // Skip other attributes
            }
        }

        if let Some(source_idx) = source_idx {
            if required_attr_found {
                let attr = &mut attrs[source_idx];
                let source: String = (&*attr.value).into();

                debug!("New source from <{}>: {:?}", name, source);

                match UrlParser::new().base_url(self.get_base()).parse(&source) {
                    Ok(url) => {
                        // Rewrite URL
                        mem::replace(&mut attr.value, rewrite_url(&url).into());

                        // Update explored resources
                        self.explored_resources.push(url);
                    }
                    Err(_) => {
                        warn!("Failed to resolve URL {} with respect to {}",
                              source,
                              self.get_base())
                    }
                }
            } else {
                trace!("Skipping element since required attribtes are not found")
            }
        } else {
            trace!("Skipping element since source attribte cannot be found")
        }
    }

    fn get_base(&self) -> &Url {
        self.html_base.as_ref().unwrap_or(&self.resource.get_response_url())
    }
}


// FIXME: Test for <base> handling
// FIXME: Test for lowercase/uppercase/mixed case tag and attr handling
