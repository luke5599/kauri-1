extern crate serde_json;
extern crate xml;
extern crate zip;

use crate::document::node::{Element, Node, Text};
use crate::document::units::DistanceUnit;
use crate::document::{Document, PaperSize};
use std::collections::HashMap;
use xml::reader::{EventReader, XmlEvent};

pub struct ODTParser {
    body_begin: bool,
    styles_begin: bool,
    auto_styles: HashMap<String, HashMap<String, String>>,
    set_children_underline: Vec<bool>,
    ensure_children_no_underline: Vec<bool>,
    document_root: Document,
    document_hierarchy: Vec<Element>,
}

impl ODTParser {
    /// Initialises a new ODTParser instance
    pub fn new() -> ODTParser {
        let document_root = Document::new(
            "Kauri (Working Title)".to_string(),
            PaperSize::new(297, 210, DistanceUnit::Millimetres),
        );
        let document_hierarchy: Vec<Element> = Vec::new();
        ODTParser {
            body_begin: false,
            styles_begin: false,
            auto_styles: HashMap::new(),
            set_children_underline: Vec::new(),
            ensure_children_no_underline: Vec::new(),
            document_root,
            document_hierarchy,
        }
    }

    /// Parse the ODT file referenced by the file path
    pub fn parse(&mut self, filepath: &str) -> Result<String, String> {
        let archive = super::util::get_archive(filepath);
        if let Err(e) = archive {
            return Err(e.to_string());
        }
        let archive = archive.unwrap();
        self.parse_private(archive)
    }

    /// Actually parse the file, this is a separate function so we actually own the archive here
    fn parse_private(
        &mut self,
        mut archive: zip::ZipArchive<std::fs::File>,
    ) -> Result<String, String> {
        // returns a ZipFile struct which implements Read if the file is in the archive
        let content_xml = archive.by_name("content.xml");
        if let Err(e) = content_xml {
            // Handle case where there is no content.xml (so probably not actually an ODT file)
            return Err(e.to_string());
        }

        // These are here instead of the struct because we may need to move the contents of these somewhere else
        let mut current_style_name = String::new();
        let mut current_style_value: HashMap<String, String> = HashMap::new();

        let parser = EventReader::new(content_xml.unwrap());
        for e in parser {
            // Iterate through the XML
            match e {
                Ok(XmlEvent::StartElement {
                    name, attributes, ..
                }) => {
                    let (current_style_name_new, current_style_value_new) =
                        self.handle_element_start(name, attributes);
                    if let Some(x) = current_style_name_new {
                        current_style_name = x;
                    }
                    if let Some(x) = current_style_value_new {
                        current_style_value = x;
                    }
                }
                Ok(XmlEvent::Characters(contents)) => self.handle_characters(contents),
                Ok(XmlEvent::EndElement { name }) => {
                    let result =
                        self.handle_element_end(name, current_style_name, current_style_value);
                    if let Some(x) = result {
                        // If they were not used inside handle_element_end() then put them back
                        let (current_style_name_new, current_style_value_new) = x;
                        current_style_name = current_style_name_new;
                        current_style_value = current_style_value_new;
                    } else {
                        // Otherwise reinitialise them
                        current_style_name = String::new();
                        current_style_value = HashMap::new();
                    }
                }
                Err(e) => {
                    println!("Error: {}", e);
                    return Err(e.to_string());
                }
                _ => {}
            }
        }

        Ok(self.document_root.to_json().unwrap())
    }

    /// Handles a StartElement event from the XML parser by taking its contents (only name and attributes needed)
    /// and returns the new values of current_style_name and current_style_value if either were set as a result
    /// as well as mutating internal state accordingly
    fn handle_element_start(
        &mut self,
        name: xml::name::OwnedName,
        attributes: Vec<xml::attribute::OwnedAttribute>,
    ) -> (Option<String>, Option<HashMap<String, String>>) {
        let mut current_style_name: Option<String> = None;
        let mut current_style_value: Option<HashMap<String, String>> = None;
        if let Some(prefix) = name.prefix {
            if prefix == "office" && name.local_name == "body" {
                self.body_begin = true;
            } else if self.body_begin {
                if prefix != "text" {
                    return (current_style_name, current_style_value);
                }
                match name.local_name.as_str() {
                    "h" => {
                        let (element, set_children_underline_new, ensure_children_no_underline_new) =
                            check_underline(
                                heading_begin(attributes),
                                &self.auto_styles,
                                !self.set_children_underline.is_empty()
                                    && *self.set_children_underline.last().unwrap(),
                                !self.ensure_children_no_underline.is_empty()
                                    && *self.ensure_children_no_underline.last().unwrap(),
                            );
                        self.ensure_children_no_underline
                            .push(ensure_children_no_underline_new);
                        self.set_children_underline.push(set_children_underline_new);
                        self.document_hierarchy.push(element);
                    }
                    "p" => {
                        let (element, set_children_underline_new, ensure_children_no_underline_new) =
                            check_underline(
                                paragraph_begin(attributes),
                                &self.auto_styles,
                                !self.set_children_underline.is_empty()
                                    && *self.set_children_underline.last().unwrap(),
                                !self.ensure_children_no_underline.is_empty()
                                    && *self.ensure_children_no_underline.last().unwrap(),
                            );
                        self.ensure_children_no_underline
                            .push(ensure_children_no_underline_new);
                        self.set_children_underline.push(set_children_underline_new);
                        self.document_hierarchy.push(element);
                    }
                    "span" => {
                        let (element, set_children_underline_new, ensure_children_no_underline_new) =
                            check_underline(
                                span_begin(attributes),
                                &self.auto_styles,
                                !self.set_children_underline.is_empty()
                                    && *self.set_children_underline.last().unwrap(),
                                !self.ensure_children_no_underline.is_empty()
                                    && *self.ensure_children_no_underline.last().unwrap(),
                            );
                        self.ensure_children_no_underline
                            .push(ensure_children_no_underline_new);
                        self.set_children_underline.push(set_children_underline_new);
                        self.document_hierarchy.push(element);
                    }
                    _ => (),
                }
            } else if prefix == "office" && name.local_name == "automatic-styles" {
                self.styles_begin = true;
            } else if self.styles_begin {
                if prefix == "style" && name.local_name == "style" {
                    current_style_name = Some(style_begin(attributes));
                } else if prefix == "style" && name.local_name == "text-properties" {
                    current_style_value = Some(text_properties_begin(attributes));
                }
            }
        }
        (current_style_name, current_style_value)
    }

    /// Handles a Characters event from the XML parser by taking its contents
    /// and mutating internal state accordingly
    fn handle_characters(&mut self, contents: String) {
        if self.document_hierarchy.is_empty() {
            return;
        }
        // Currently the only type of tag expected to emit this event is the ones in the body,
        // in which case they will contain the document text
        let text = Text::new(contents);
        self.document_hierarchy
            .last_mut()
            .unwrap()
            .children
            .push(Node::Text(text));
    }

    /// Handles an EndElement event from the XML parser by taking its contents (the name of the element),
    /// the style name and value of the current element and mutating internal state accordingly,
    /// then it will return the current_style_name and current_style_value back if they were not used
    fn handle_element_end(
        &mut self,
        name: xml::name::OwnedName,
        current_style_name: String,
        current_style_value: HashMap<String, String>,
    ) -> Option<(String, HashMap<String, String>)> {
        if self.body_begin {
            if let Some(prefix) = name.prefix {
                if prefix == "office" && name.local_name == "body" {
                    return Some((current_style_name, current_style_value));
                } else if prefix == "text"
                    && (name.local_name == "h"
                        || name.local_name == "p"
                        || name.local_name == "span")
                {
                    if self.document_hierarchy.is_empty() {
                        // It shouldn't be empty now, if it is then this is an unmatched end tag
                        return Some((current_style_name, current_style_value));
                    }
                    // The top of set_children_underline and ensure_children_no_underline is for this node's children,
                    // so pop them here before we finish up with this node
                    self.set_children_underline.pop();
                    self.ensure_children_no_underline.pop();
                    let mut child = self.document_hierarchy.pop().unwrap();
                    if name.local_name == "span" {
                        handle_underline(
                            &mut child.styles,
                            !self.set_children_underline.is_empty()
                                && *self.set_children_underline.last().unwrap(),
                            !self.ensure_children_no_underline.is_empty()
                                && *self.ensure_children_no_underline.last().unwrap(),
                        );
                    }
                    if self.document_hierarchy.is_empty() {
                        self.document_root.children.push(Node::Element(child));
                    } else {
                        self.document_hierarchy
                            .last_mut()
                            .unwrap()
                            .children
                            .push(Node::Element(child));
                    }
                }
            }
        } else if self.styles_begin {
            if let Some(prefix) = name.prefix {
                if prefix == "office" && name.local_name == "automatic-styles" {
                    self.styles_begin = false;
                } else if prefix == "style" && name.local_name == "style" {
                    self.auto_styles
                        .insert(current_style_name, current_style_value);
                    return None;
                }
            }
        }
        Some((current_style_name, current_style_value))
    }
}

/// Takes the results of either heading_begin() or paragraph_begin() (called params)
/// and a reference to the map of automatic style names to the map of CSS properties,
/// and returns the element from params together with the values for set_children_underline
/// and ensure_children_no_underline in ODTParser
fn check_underline(
    params: (Element, String),
    auto_styles: &HashMap<String, HashMap<String, String>>,
    set_children_underline_old: bool,
    ensure_children_no_underline_old: bool,
) -> (Element, bool, bool) {
    let mut ensure_children_no_underline = ensure_children_no_underline_old;
    let mut set_children_underline = set_children_underline_old;
    let (mut element, style_name) = params;
    let style = auto_styles
        .get(&style_name)
        .unwrap_or(&HashMap::new())
        .clone();
    let underline = style.get("textDecorationLine");
    let underline_color = style.get("textDecorationColor");
    if let Some(x) = underline {
        if x == "underline" {
            ensure_children_no_underline = true;
            if let Some(x) = underline_color {
                set_children_underline = x == "currentcolor";
            }
        } else if x == "none" {
            ensure_children_no_underline = false;
        }
    }
    element.styles = style;
    (
        element,
        set_children_underline,
        ensure_children_no_underline,
    )
}

/// Takes a mutable reference to a map of CSS style properties to values and 2 booleans
/// (the boolean results of check_underline()), and adds an extra CSS property to
/// handle some special cases related to underlines if needed depending on the booleans
fn handle_underline(
    style_map: &mut HashMap<String, String>,
    set_children_underline: bool,
    ensure_children_no_underline: bool,
) {
    if set_children_underline {
        if let Some(x) = style_map.get("textDecorationLine") {
            if x != "none" {
                style_map.insert("textDecorationLine".to_string(), "underline".to_string());
            } else if ensure_children_no_underline {
                // Need this to make sure the underline is actually not there, because CSS things
                style_map.insert("display".to_string(), "inline-block".to_string());
            }
        } else {
            style_map.insert("textDecoration".to_string(), "underline".to_string());
        }
    } else if ensure_children_no_underline {
        if let Some(x) = style_map.get("textDecorationLine") {
            if x == "none" {
                // Need this to make sure the underline is actually not there, because CSS things
                style_map.insert("display".to_string(), "inline-block".to_string());
            }
        }
    }
}

/// Takes the set of attributes of a text:h tag in the ODT's content.xml,
/// and returns a heading element based on the attributes,
/// together with the value of the text:style-name attribute of the tag
fn heading_begin(attributes: Vec<xml::attribute::OwnedAttribute>) -> (Element, String) {
    // Because JS numbers are always floats apparently
    let mut level = 0.0;
    let mut style_name = String::new();
    for i in attributes {
        let prefix = i.name.prefix.unwrap_or_else(|| "".to_string());
        if prefix == "text" && i.name.local_name == "outline-level" {
            level = i.value.parse::<f64>().unwrap_or(1.0);
        } else if prefix == "text" && i.name.local_name == "style-name" {
            style_name = i.value;
        }
    }
    let mut element = Element::new("heading".to_string());
    element
        .attributes
        .insert("level".to_string(), level.to_string());
    (element, style_name)
}

/// Takes the set of attributes of a text:p tag in the ODT's content.xml,
/// and returns a paragraph element together with the value of the text:style-name attribute of the tag
fn paragraph_begin(attributes: Vec<xml::attribute::OwnedAttribute>) -> (Element, String) {
    let mut style_name = String::new();
    for i in attributes {
        if i.name.prefix.unwrap_or_else(|| "".to_string()) == "text"
            && i.name.local_name == "style-name"
        {
            style_name = i.value;
        }
    }
    (Element::new("paragraph".to_string()), style_name)
}

/// Takes the set of attributes of a text:span tag in the ODT's content.xml
/// and returns a span element together with the value of the text:style-name attribute of the tag
fn span_begin(attributes: Vec<xml::attribute::OwnedAttribute>) -> (Element, String) {
    let mut style_name = String::new();
    for i in attributes {
        if i.name.prefix.unwrap_or_else(|| "".to_string()) == "text"
            && i.name.local_name == "style-name"
        {
            style_name = i.value;
        }
    }
    (Element::new("span".to_string()), style_name)
}

/// Takes the set of attributes of a style:style tag in the ODT's content.xml,
/// and returns the name of the style
fn style_begin(attributes: Vec<xml::attribute::OwnedAttribute>) -> String {
    for i in attributes {
        if i.name.prefix.unwrap_or_else(|| "".to_string()) == "style" && i.name.local_name == "name"
        {
            return i.value;
        }
    }
    String::new()
}

/// Takes the set of attributes of a style:text-properties tag in the ODT's content.xml,
/// and creates a map of CSS properties based on the attributes
fn text_properties_begin(
    attributes: Vec<xml::attribute::OwnedAttribute>,
) -> HashMap<String, String> {
    let mut map: HashMap<String, String> = HashMap::new();
    let mut is_double_underline = false;
    for i in attributes {
        let prefix = i.name.prefix.unwrap_or_else(|| "".to_string());
        if prefix == "fo" {
            if i.name.local_name == "font-weight" {
                // All valid values for this attribute is also valid in the CSS equivalent, so just use it as is
                map.insert("fontWeight".to_string(), i.value);
            } else if i.name.local_name == "font-style" && i.value != "backslant" {
                // `backslant` is not valid in CSS, but all the other ones are
                map.insert("fontStyle".to_string(), i.value);
            } else if i.name.local_name == "color" {
                map.insert("color".to_string(), i.value);
            } else if i.name.local_name == "font-size" {
                map.insert("fontSize".to_string(), i.value);
            }
        } else if prefix == "style" {
            if i.name.local_name == "text-underline-style" {
                if i.value == "none" {
                    map.insert("textDecorationLine".to_string(), "none".to_string());
                } else {
                    map.insert("textDecorationLine".to_string(), "underline".to_string());
                    match i.value.as_ref() {
                        "dash" => {
                            map.insert("textDecorationStyle".to_string(), "dashed".to_string())
                        }
                        "dotted" => {
                            map.insert("textDecorationStyle".to_string(), "dotted".to_string())
                        }
                        "wave" => map.insert("textDecorationStyle".to_string(), "wavy".to_string()),
                        // There are a few possible styles in ODF that aren't present in CSS
                        // (dot-dash, dot-dot-dash, long-dash), so just put in a basic underline?
                        "solid" | _ => {
                            map.insert("textDecorationStyle".to_string(), "solid".to_string())
                        }
                    };
                }
            } else if i.name.local_name == "text-underline-type" && i.value == "double" {
                is_double_underline = true;
            } else if i.name.local_name == "text-underline-color" {
                if i.value == "font-color" {
                    map.insert(
                        "textDecorationColor".to_string(),
                        "currentcolor".to_string(),
                    );
                } else {
                    // The other valid values are all in hex format
                    map.insert("textDecorationColor".to_string(), i.value);
                }
            } else if i.name.local_name == "font-name" {
                map.insert("fontFamily".to_string(), i.value);
            }
        }
    }
    if is_double_underline {
        // The ODT standard supports double underlines of any kind (solid, dotted, etc), while CSS
        // only supports double solid underlines, so prioritize the double over the line style?
        map.insert("textDecorationStyle".to_string(), "double".to_string());
    }
    map
}