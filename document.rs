#![crate_name = "document"]
#![feature(macro_rules)]

use std::fmt;
use std::rc::{Rc,Weak};
use std::cell::RefCell;
use std::collections::hashmap::HashMap;

// FIXME: Parents need to be weakref!
// TODO: See about removing duplication of child / parent implementations.
// TODO: remove clone from inner?

// children
// root nodes -> 1x element, comment, pi
// element nodes -> element, comment, text, pi (attribute, namespace)
// text nodes ->
// attribute nodes ->
// namespace nodes ->
// processing instruction nodes ->
// comment nodes ->
//
// parents
// root nodes ->
// element nodes -> element, root
// text nodes -> element
// attribute nodes -> element
// namespace nodes -> element
// processing instruction nodes -> element
// comment nodes -> element

struct DocumentInner {
    // We will always have a root, but during construction we have to
    // pick one first
    root: Option<Root>,
}

#[deriving(Clone)]
pub struct Document {
    inner: Rc<RefCell<DocumentInner>>,
}

impl Document {
    pub fn new() -> Document {
        let inner = DocumentInner { root: None };
        let doc = Document { inner: Rc::new(RefCell::new(inner)) };
        let root = Root::new(doc.clone());
        doc.inner.borrow_mut().root = Some(root);
        doc
    }

    pub fn new_element(&self, name: String) -> Element {
        Element::new(self.clone(), name)
    }

    pub fn new_text(&self, text: String) -> Text {
        Text::new(self.clone(), text)
    }

    pub fn root(&self) -> Root {
        let inner = self.inner.borrow();
        inner.root.clone().unwrap()
    }
}

impl PartialEq for Document {
    fn eq(&self, other: &Document) -> bool {
        &*self.inner as *const RefCell<DocumentInner> == &*other.inner as *const RefCell<DocumentInner>
    }
}

impl fmt::Show for Document {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Document")
    }
}

/// Items that may be children of the root node
#[deriving(Clone,PartialEq)]
pub enum RootChild {
    ElementRootChild(Element),
}

impl RootChild {
    fn is_element(&self) -> bool {
        match self {
            &ElementRootChild(_) => true,
        }
    }

    pub fn element(&self) -> Option<Element> {
        match self {
            &ElementRootChild(ref e) => Some(e.clone()),
        }
    }

    pub fn remove_from_parent(&self) {
        match self {
            &ElementRootChild(ref e) => e.remove_from_parent(),
        }
    }

    pub fn set_parent(&self, parent: Root) {
        match self {
            &ElementRootChild(ref e) => e.set_parent(parent),
        }
    }
}

pub trait ToRootChild {
    fn to_root_child(&self) -> RootChild;
}

impl ToRootChild for RootChild {
    fn to_root_child(&self) -> RootChild { self.clone() }
}

impl ToRootChild for Element {
    fn to_root_child(&self) -> RootChild { ElementRootChild(self.clone()) }
}

#[deriving(Clone)]
struct RootInner {
    document: Document,
    children: Vec<RootChild>,
}

#[deriving(Clone)]
pub struct Root {
    inner: Rc<RefCell<RootInner>>,
}

impl Root {
    fn new(document: Document) -> Root {
        let inner = RootInner { document: document, children: Vec::new() };
        Root { inner: Rc::new(RefCell::new(inner)) }
    }

    pub fn document(&self) -> Document {
        self.inner.borrow().document.clone()
    }

    fn remove_element_children(&self) {
        let mut inner = self.inner.borrow_mut();

        inner.children.retain(|c| ! c.is_element());
    }

    pub fn remove_child<C : ToRootChild>(&self, child: C) {
        let child = child.to_root_child();
        let mut inner = self.inner.borrow_mut();
        inner.children.retain(|c| c != &child);
    }

    /// This removes any existing element children before appending a new element
    pub fn append_child<C : ToRootChild>(&self, child: C) {
        let child = child.to_root_child();

        if child.is_element() {
            self.remove_element_children();
        }

        child.remove_from_parent();
        child.set_parent(self.clone());

        let mut inner = self.inner.borrow_mut();
        inner.children.push(child);
    }

    pub fn children(&self) -> Vec<RootChild> {
        let inner = self.inner.borrow();
        inner.children.clone()
    }
}

impl PartialEq for Root {
    fn eq(&self, other: &Root) -> bool {
        &*self.inner as *const RefCell<RootInner> == &*other.inner as *const RefCell<RootInner>
    }
}

impl fmt::Show for Root {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Root element")
    }
}

#[deriving(Clone)]
struct TextInner {
    document: Document,
    text: String,
    parent: Option<Element>,
}

#[deriving(Clone)]
pub struct Text {
    inner: Rc<RefCell<TextInner>>,
}

impl Text {
    fn new(document: Document, text: String) -> Text {
        let inner = TextInner {document: document, text: text, parent: None};
        Text {inner: Rc::new(RefCell::new(inner))}
    }

    pub fn document(&self) -> Document {
        self.inner.borrow().document.clone()
    }

    pub fn text(&self) -> String {
        let inner = self.inner.borrow();
        inner.text.clone()
    }

    pub fn set_text(&self, text: String) {
        let mut inner = self.inner.borrow_mut();
        inner.text = text;
    }

    pub fn remove_from_parent(&self) {
        let mut inner = self.inner.borrow_mut();
        match inner.parent {
            Some(ref e) => e.remove_child(self.clone()),
            None => {}
        };
        inner.parent = None;
    }

    fn set_parent(&self, parent: Element) {
        let mut inner = self.inner.borrow_mut();
        inner.parent = Some(parent);
    }

    pub fn parent(&self) -> Option<Element> {
        let inner = self.inner.borrow();
        inner.parent.clone()
    }
}

impl PartialEq for Text {
    fn eq(&self, other: &Text) -> bool {
        &*self.inner as *const RefCell<TextInner> == &*other.inner as *const RefCell<TextInner>
    }
}

impl fmt::Show for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Text: {}", self.inner.borrow().text)
    }
}

#[deriving(Clone)]
struct AttributeInner {
    document: Document,
    name: String,
    value: String,
    element: Option<Weak<RefCell<ElementInner>>>,
}

#[deriving(Clone)]
pub struct Attribute {
    inner: Rc<RefCell<AttributeInner>>,
}

impl Attribute {
    fn new(document: Document, name: String, value: String) -> Attribute {
        let inner = AttributeInner {document: document,
                                    name: name,
                                    value: value,
                                    element: None};
        Attribute {inner: Rc::new(RefCell::new(inner))}
    }

    pub fn document(&self) -> Document {
        self.inner.borrow().document.clone()
    }

    pub fn name(&self) -> String {
        self.inner.borrow().name.clone()
    }

    pub fn value(&self) -> String {
        self.inner.borrow().value.clone()
    }

    pub fn parent(&self) -> Option<Element> {
        let a = self.inner.borrow();
        let b = &a.element;
        let c = b.as_ref().and_then(|x| x.upgrade());
        let d = c.map(|x| Element {inner: x});
        d
    }
}

impl PartialEq for Attribute {
    fn eq(&self, other: &Attribute) -> bool {
        &*self.inner as *const RefCell<AttributeInner> == &*other.inner as *const RefCell<AttributeInner>
    }
}

impl fmt::Show for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let inner = self.inner.borrow();
        write!(f, "@{}='{}'", inner.name, inner.value)
    }
}

/// Items that may be children of element nodes
#[deriving(Clone,PartialEq,Show)]
pub enum ElementChild {
    ElementElementChild(Element),
    TextElementChild(Text),
}

impl ElementChild {
    pub fn element(&self) -> Option<Element> {
        match self {
            &ElementElementChild(ref e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<Text> {
        match self {
            &TextElementChild(ref t) => Some(t.clone()),
            _ => None,
        }
    }

    pub fn remove_from_parent(&self) {
        match self {
            &ElementElementChild(ref e) => e.remove_from_parent(),
            &TextElementChild(ref t) => t.remove_from_parent(),
        }
    }

    fn set_parent(&self, parent: Element) {
        match self {
            &ElementElementChild(ref e) => e.set_parent(parent),
            &TextElementChild(ref t) => t.set_parent(parent),
        }
    }

    pub fn parent(&self) -> Option<Element> {
        match self {
            &ElementElementChild(ref e) =>
                match e.parent() {
                    None => None,
                    Some(ElementElementParent(ref e)) => Some(e.clone()),
                    _ => fail!("An element's child's parent is not an element")
                },
            &TextElementChild(ref t) => t.parent(),
        }
    }
}

pub trait ToElementChild {
    fn to_element_child(&self) -> ElementChild;
}

impl ToElementChild for ElementChild {
    fn to_element_child(&self) -> ElementChild { self.clone() }
}

impl ToElementChild for Element {
    fn to_element_child(&self) -> ElementChild { ElementElementChild(self.clone()) }
}

impl ToElementChild for Text {
    fn to_element_child(&self) -> ElementChild { TextElementChild(self.clone()) }
}

impl ToElementChild for RootChild {
    fn to_element_child(&self) -> ElementChild {
        match self {
            &ElementRootChild(ref e) => ElementElementChild(e.clone()),
        }
    }
}

/// Items that may be parents of element nodes
#[deriving(PartialEq,Clone)]
pub enum ElementParent {
    ElementElementParent(Element),
    RootElementParent(Root),
}

impl ElementParent {
    pub fn element(&self) -> Option<Element> {
        match self {
            &ElementElementParent(ref e) => Some(e.clone()),
            _ => None
        }
    }

    pub fn root(&self) -> Option<Root> {
        match self {
            &RootElementParent(ref r) => Some(r.clone()),
            _ => None
        }
    }

    pub fn remove_child(&self, child: Element) {
        match self {
            &ElementElementParent(ref e) => e.remove_child(child),
            &RootElementParent(ref r) => r.remove_child(child),
        }
    }

    pub fn children(&self) -> Vec<ElementChild> {
        match self {
            &ElementElementParent(ref e) => e.children(),
            &RootElementParent(ref e) => e.children().iter().map(|x| x.to_element_child()).collect(),
        }
    }
}

pub trait ToElementParent {
    fn to_element_parent(&self) -> ElementParent;
}

impl ToElementParent for Element {
    fn to_element_parent(&self) -> ElementParent {
        ElementElementParent(self.clone())
    }
}

impl ToElementParent for Root {
    fn to_element_parent(&self) -> ElementParent {
        RootElementParent(self.clone())
    }
}

#[deriving(Clone)]
struct ElementInner {
    document: Document,
    name: String,
    parent: Option<ElementParent>,
    children: Vec<ElementChild>,
    attributes: HashMap<String, Attribute>,
}

#[deriving(Clone)]
pub struct Element {
    inner: Rc<RefCell<ElementInner>>,
}

// TODO: See about using the attribute value reference as the key to the hash
impl Element {
    fn new(document: Document, name: String) -> Element {
        let inner = ElementInner {document: document,
                                  name: name,
                                  parent: None,
                                  children: Vec::new(),
                                  attributes: HashMap::new()};
        Element {inner: Rc::new(RefCell::new(inner))}
    }

    pub fn document(&self) -> Document {
        let inner = self.inner.borrow();
        inner.document.clone()
    }

    pub fn name(&self) -> String {
        let inner = self.inner.borrow();
        inner.name.clone()

    }

    pub fn set_name(&self, name: String) {
        let mut inner = self.inner.borrow_mut();
        inner.name = name;
    }

    pub fn parent(&self) -> Option<ElementParent> {
        let inner = self.inner.borrow();
        inner.parent.clone()
    }

    // Does not change child at all
    fn remove_child<C : ToElementChild>(&self, child: C) {
        let child = child.to_element_child();
        let mut inner = self.inner.borrow_mut();
        inner.children.retain(|c| c != &child);
    }

    fn remove_from_parent(&self) {
        let mut inner = self.inner.borrow_mut();
        match inner.parent {
            Some(ref e) => e.remove_child(self.clone()),
            None => {}
        };
        inner.parent = None;
    }

    fn set_parent<P : ToElementParent>(&self, parent: P) {
        let parent = parent.to_element_parent();
        let mut inner = self.inner.borrow_mut();
        inner.parent = Some(parent);
    }

    pub fn append_child<C : ToElementChild>(&self, child: C) {
        let child = child.to_element_child();

        child.remove_from_parent();
        child.set_parent(self.clone());

        let mut inner = self.inner.borrow_mut();
        inner.children.push(child.clone());
    }

    pub fn children(&self) -> Vec<ElementChild> {
        let inner = self.inner.borrow();
        inner.children.clone()
    }

    pub fn set_attribute(&self, name: String, value: String) -> Attribute {
        let attr = {
            let inner = self.inner.borrow();
            Attribute::new(inner.document.clone(), name.clone(), value)
        };

        attr.inner.borrow_mut().element = Some(self.inner.downgrade());
        self.inner.borrow_mut().attributes.insert(name, attr.clone());
        attr
    }

    pub fn attributes(&self) -> Vec<Attribute> {
        let inner = self.inner.borrow();
        inner.attributes.values().map(|a| a.clone()).collect()
    }

    pub fn each_attribute(&self, f: |&Attribute|) {
        let inner = self.inner.borrow();
        for attr in inner.attributes.values() {
            f(attr);
        }
    }

    pub fn attribute(&self, name: &str) -> Option<Attribute> {
        let inner = self.inner.borrow();
        inner.attributes.find(&name.to_string()).map(|x| x.clone())
    }

    // TODO: look into equiv
    pub fn get_attribute(&self, name: &str) -> Option<String> {
        let inner = self.inner.borrow();
        let attr = inner.attributes.find(&name.to_string());
        attr.map(|x| x.inner.borrow().value.clone())
    }
}

impl PartialEq for Element {
    fn eq(&self, other: &Element) -> bool {
        // Nodes have reference equality, so we just check to see if
        // we are pointing at the same thing.
        &*self.inner as *const RefCell<ElementInner> == &*other.inner as *const RefCell<ElementInner>
    }
}

impl fmt::Show for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<{}>", self.inner.borrow().name)
    }
}

#[deriving(Clone,Show,PartialEq)]
pub enum Any {
    ElementAny(Element),
    AttributeAny(Attribute),
    TextAny(Text),
    RootAny(Root),
}

impl Any {
    pub fn element(&self) -> Option<Element> {
        match self {
            &ElementAny(ref e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn attribute(&self) -> Option<Attribute> {
        match self {
            &AttributeAny(ref e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn text(&self) -> Option<Text> {
        match self {
            &TextAny(ref e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn root(&self) -> Option<Root> {
        match self {
            &RootAny(ref e) => Some(e.clone()),
            _ => None,
        }
    }

    pub fn document(&self) -> Document {
        match self {
            &AttributeAny(ref a) => a.document(),
            &ElementAny(ref e)   => e.document(),
            &RootAny(ref r)      => r.document(),
            &TextAny(ref t)      => t.document(),
        }
    }

    pub fn parent(&self) -> Option<Any> {
        match self {
            &AttributeAny(ref a) => a.parent().map(|x| x.to_any()),
            &ElementAny(ref e)   => e.parent().map(|x| x.to_any()),
            &TextAny(ref t)      => t.parent().map(|x| x.to_any()),
            &RootAny(_) => None,
        }
    }

    pub fn children(&self) -> Vec<Any> {
        match self {
            &ElementAny(ref e) => e.children().iter().map(|x| x.to_any()).collect(),
            &RootAny(ref r)    => r.children().iter().map(|x| x.to_any()).collect(),
            &TextAny(_)        |
            &AttributeAny(_)   => Vec::new(),
        }
    }
}

pub trait ToAny {
    fn to_any(&self) -> Any;
}

impl ToAny for Any {
    fn to_any(&self) -> Any { self.clone() }
}

impl ToAny for Element {
    fn to_any(&self) -> Any { ElementAny(self.clone()) }
}

impl ToAny for Attribute {
    fn to_any(&self) -> Any { AttributeAny(self.clone()) }
}

impl ToAny for Text {
    fn to_any(&self) -> Any { TextAny(self.clone()) }
}

impl ToAny for Root {
    fn to_any(&self) -> Any { RootAny(self.clone()) }
}

impl ToAny for ElementChild {
    fn to_any(&self) -> Any {
        match self {
            &ElementElementChild(ref e) => ElementAny(e.clone()),
            &TextElementChild(ref t)    => TextAny(t.clone()),
        }
    }
}

impl ToAny for ElementParent {
    fn to_any(&self) -> Any {
        match self {
            &ElementElementParent(ref e) => ElementAny(e.clone()),
            &RootElementParent(ref r)    => RootAny(r.clone()),
        }
    }
}

impl ToAny for RootChild {
    fn to_any(&self) -> Any {
        match self {
            &ElementRootChild(ref r) => ElementAny(r.clone()),
        }
    }
}

#[macro_export]
macro_rules! nodeset(
    ($($e:expr),*) => ({
        // leading _ to allow empty construction without a warning.
        let mut _temp = ::document::Nodeset::new();
        $(_temp.add($e);)*
        _temp
    });
    ($($e:expr),+,) => (nodeset!($($e),+))
)

#[deriving(Clone,PartialEq,Show)]
pub struct Nodeset {
    nodes: Vec<Any>,
}

impl Nodeset {
    pub fn new() -> Nodeset {
        Nodeset { nodes: Vec::new() }
    }

    pub fn add<A : ToAny>(&mut self, node: A) {
        self.nodes.push(node.to_any());
    }

    pub fn iter(&self) -> std::slice::Items<Any> {
        self.nodes.iter()
    }

    pub fn add_nodeset(& mut self, other: &Nodeset) {
        self.nodes.push_all(other.nodes.as_slice());
    }

    pub fn size(&self) -> uint {
        self.nodes.len()
    }
}

#[test]
fn elements_belong_to_a_document() {
    let doc = Document::new();
    let element = doc.new_element("alpha".to_string());

    assert_eq!(doc, element.document());
}

#[test]
fn elements_can_have_element_children() {
    let doc = Document::new();
    let alpha = doc.new_element("alpha".to_string());
    let beta  = doc.new_element("beta".to_string());

    alpha.append_child(beta.clone());

    let children = alpha.children();
    let ref child = children[0].element().unwrap();

    assert_eq!(beta, *child);
}

#[test]
fn element_children_are_ordered() {
    let doc = Document::new();
    let greek = doc.new_element("greek".to_string());
    let alpha = doc.new_element("alpha".to_string());
    let omega = doc.new_element("omega".to_string());

    greek.append_child(alpha.clone());
    greek.append_child(omega.clone());

    let children = greek.children();

    assert_eq!(children[0].element().unwrap(), alpha);
    assert_eq!(children[1].element().unwrap(), omega);
}

#[test]
fn element_children_know_their_parent() {
    let doc = Document::new();
    let alpha = doc.new_element("alpha".to_string());
    let beta  = doc.new_element("beta".to_string());

    alpha.append_child(beta);

    let ref child = alpha.children()[0];
    let parent = child.parent().unwrap();

    assert_eq!(alpha, parent);
}

#[test]
fn replacing_parent_updates_original_parent() {
    let doc = Document::new();
    let parent1 = doc.new_element("parent1".to_string());
    let parent2 = doc.new_element("parent2".to_string());
    let child = doc.new_element("child".to_string());

    parent1.append_child(child.clone());
    parent2.append_child(child.clone());

    assert!(parent1.children().is_empty());
    assert_eq!(1, parent2.children().len());
}

#[test]
fn elements_can_be_renamed() {
    let doc = Document::new();
    let alpha = doc.new_element("alpha".to_string());
    alpha.set_name("beta".to_string());
    assert_eq!(alpha.name().as_slice(), "beta");
}

#[test]
fn elements_have_attributes() {
    let doc = Document::new();
    let e = doc.new_element("element".to_string());

    let a = e.set_attribute("hello".to_string(), "world".to_string());

    assert_eq!(doc, a.document());
}

#[test]
fn attributes_belong_to_a_document() {
    let doc = Document::new();
    let element = doc.new_element("alpha".to_string());

    assert_eq!(doc, element.document());
}

#[test]
fn attributes_know_their_element() {
    let doc = Document::new();
    let e = doc.new_element("element".to_string());

    let a = e.set_attribute("hello".to_string(), "world".to_string());

    assert_eq!(Some(e), a.parent());
}

#[test]
fn attributes_can_be_reset() {
    let doc = Document::new();
    let e = doc.new_element("element".to_string());

    e.set_attribute("hello".to_string(), "world".to_string());
    e.set_attribute("hello".to_string(), "galaxy".to_string());

    assert_eq!(Some("galaxy".to_string()), e.get_attribute("hello"));
}

#[test]
fn attributes_can_be_iterated() {
    let doc = Document::new();
    let e = doc.new_element("element".to_string());

    e.set_attribute("name1".to_string(), "value1".to_string());
    e.set_attribute("name2".to_string(), "value2".to_string());

    let mut attrs = e.attributes();
    attrs.sort_by(|a, b| a.name().cmp(&b.name()));

    assert_eq!(2, attrs.len());
    assert_eq!("name1",  attrs[0].name().as_slice());
    assert_eq!("value1", attrs[0].value().as_slice());
    assert_eq!("name2",  attrs[1].name().as_slice());
    assert_eq!("value2", attrs[1].value().as_slice());
}

#[test]
fn elements_can_have_text_children() {
    let doc = Document::new();
    let sentence = doc.new_element("sentence".to_string());
    let text = doc.new_text("Now is the winter of our discontent.".to_string());

    sentence.append_child(text);

    let children = sentence.children();
    assert_eq!(1, children.len());

    let child_text = children[0].text().unwrap();
    assert_eq!(child_text.text().as_slice(), "Now is the winter of our discontent.");
}

#[test]
fn text_belongs_to_a_document() {
    let doc = Document::new();
    let text = doc.new_text("Now is the winter of our discontent.".to_string());

    assert_eq!(doc, text.document());
}

#[test]
fn text_knows_its_parent() {
    let doc = Document::new();
    let sentence = doc.new_element("sentence".to_string());
    let text = doc.new_text("Now is the winter of our discontent.".to_string());

    sentence.append_child(text.clone());

    assert_eq!(text.parent().unwrap(), sentence);
}

#[test]
fn text_can_be_changed() {
    let doc = Document::new();
    let text = doc.new_text("Now is the winter of our discontent.".to_string());

    text.set_text("Made glorious summer by this sun of York".to_string());

    assert_eq!(text.text().as_slice(), "Made glorious summer by this sun of York");
}

#[test]
fn the_root_belongs_to_a_document() {
    let doc = Document::new();
    let root = doc.root();

    assert_eq!(doc, root.document());
}

#[test]
fn root_can_have_element_children() {
    let doc = Document::new();
    let root = doc.root();
    let element = doc.new_element("alpha".to_string());

    root.append_child(element.clone());

    let children = root.children();
    assert_eq!(1, children.len());

    let child = children[0].element().unwrap();
    assert_eq!(child, element);
}

#[test]
fn root_has_maximum_of_one_element_child() {
    let doc = Document::new();
    let root = doc.root();
    let alpha = doc.new_element("alpha".to_string());
    let beta = doc.new_element("beta".to_string());

    root.append_child(alpha.clone());
    root.append_child(beta.clone());

    let children = root.children();
    assert_eq!(1, children.len());

    let child = children[0].element().unwrap();
    assert_eq!(child, beta);
}

#[test]
fn element_under_a_root_knows_its_parent_root() {
    let doc = Document::new();
    let root = doc.root();
    let alpha = doc.new_element("alpha".to_string());

    root.append_child(alpha.clone());
    let parent = alpha.parent().unwrap();

    assert_eq!(root, parent.root().unwrap());
}

#[test]
fn nodeset_can_include_all_node_types() {
    let doc = Document::new();
    let mut nodes = Nodeset::new();
    let e = doc.new_element("element".to_string());
    let a = e.set_attribute("name".to_string(), "value".to_string());
    let t = doc.new_text("text".to_string());
    let r = doc.root();

    nodes.add(e.clone());
    nodes.add(a.clone());
    nodes.add(t.clone());
    nodes.add(r.clone());

    let node_vec: Vec<&Any> = nodes.iter().collect();

    assert_eq!(4, node_vec.len());
    assert_eq!(e, node_vec[0].element().unwrap());
    assert_eq!(a, node_vec[1].attribute().unwrap());
    assert_eq!(t, node_vec[2].text().unwrap());
    assert_eq!(r, node_vec[3].root().unwrap());
}

#[test]
fn nodesets_can_be_combined() {
    let doc = Document::new();
    let mut all_nodes = Nodeset::new();
    let mut nodes1 = Nodeset::new();
    let mut nodes2 = Nodeset::new();

    let e1 = doc.new_element("element1".to_string());
    let e2 = doc.new_element("element2".to_string());

    all_nodes.add(e1.clone());
    all_nodes.add(e2.clone());

    nodes1.add(e1.clone());
    nodes2.add(e2.clone());

    nodes1.add_nodeset(&nodes2);

    assert_eq!(all_nodes, nodes1);
}
