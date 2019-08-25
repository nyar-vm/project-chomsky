use chomsky_types::{Context, Intent, IntentNode};

#[test]
fn test_intent_node_creation() {
    let node = IntentNode::new(Intent::Map);
    assert_eq!(node.intent, Intent::Map);
    assert_eq!(node.contexts, vec![Context::General]);
    assert!(node.attributes.is_empty());
}

#[test]
fn test_intent_node_with_context() {
    let node = IntentNode::new(Intent::Reduce)
        .with_context(Context::GPU)
        .with_context(Context::SIMD);

    assert_eq!(node.intent, Intent::Reduce);
    assert_eq!(
        node.contexts,
        vec![Context::General, Context::GPU, Context::SIMD]
    );
}

#[test]
fn test_intent_node_with_attributes() {
    let node = IntentNode::new(Intent::StateUpdate)
        .with_attribute("var", "counter")
        .with_attribute("op", "increment");

    assert_eq!(node.attributes.get("var").unwrap(), "counter");
    assert_eq!(node.attributes.get("op").unwrap(), "increment");
}

#[test]
fn test_intent_equality() {
    let node1 = IntentNode::new(Intent::Loop)
        .with_context(Context::Async)
        .with_attribute("times", "10");

    let node2 = IntentNode::new(Intent::Loop)
        .with_context(Context::Async)
        .with_attribute("times", "10");

    assert_eq!(node1, node2);
}
