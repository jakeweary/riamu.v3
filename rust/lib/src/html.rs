use ego_tree::NodeRef;
use scraper::{Html, Node};

pub fn strip(html: &str) -> String {
  fn visit(acc: &mut String, node: NodeRef<'_, Node>) {
    for node in node.children() {
      match node.value() {
        Node::Text(text) => acc.push_str(text),
        Node::Element(_) => visit(acc, node),
        _ => {}
      }
    }
  }

  let html = Html::parse_fragment(html);
  let root = html.tree.root();

  let mut acc = String::new();
  visit(&mut acc, root);
  acc
}
