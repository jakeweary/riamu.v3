use ego_tree::NodeRef;
use scraper::{Html, Node};

pub fn strip(html: &str) -> String {
  let mut acc = String::new();
  strip_to(&mut acc, html);
  acc
}

pub fn strip_to(acc: &mut String, html: &str) {
  fn visit(acc: &mut String, node: NodeRef<'_, Node>) {
    for node in node.children() {
      match node.value() {
        Node::Text(text) => acc.push_str(text),
        Node::Element(e) if e.name() == "br" => acc.push('\n'),
        Node::Element(_) => visit(acc, node),
        _ => {}
      }
    }
  }

  let html = Html::parse_fragment(html);
  let root = html.tree.root();
  visit(acc, root);
}
