use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

fn main() {
    let language = tree_sitter_rust::LANGUAGE.into();
    let mut parser = Parser::new();
    parser.set_language(&language).unwrap();
    let query_source = r#"
        (struct_item name: (type_identifier) @name) @struct
    "#;
    let query = Query::new(&language, query_source).unwrap();
    let content = "struct Foo { x: i32 }";
    let tree = parser.parse(&content, None).unwrap();
    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), content.as_bytes());
    while let Some(m) = matches.next() {
        for cap in m.captures {
            let node: tree_sitter::Node = cap.node;
            if let Ok(text) = node.utf8_text(content.as_bytes()) {
                if let Some(parent) = node.parent() {
                    let kind: &str = parent.kind();
                    println!("Found: {} in {}", text, kind);
                }
            }
        }
    }
}
