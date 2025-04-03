use rust::avl::AVLTree;

#[test]
fn valid_tree() {
    let first_pair = ("qw", "first");
    let second_pair = ("q", "second");
    let third_pair = ("qwe", "third");
    
    let mut tree = AVLTree::new(first_pair.0, first_pair.1);
    assert_eq!(tree.root.as_deref().unwrap().key, first_pair.0);
    assert_eq!(tree.root.as_deref().unwrap().value, first_pair.1);
    
    tree.set(second_pair.0, second_pair.1).unwrap();
    
    assert_eq!(tree.root.as_deref().unwrap().left.as_deref().unwrap().key, second_pair.0);
    assert_eq!(tree.root.as_deref().unwrap().left.as_deref().unwrap().value, second_pair.1);
    
    tree.set(third_pair.0, third_pair.1).unwrap();
    assert_eq!(tree.root.as_deref().unwrap().right.as_deref().unwrap().key, third_pair.0);
    assert_eq!(tree.root.as_deref().unwrap().right.as_deref().unwrap().value, third_pair.1);
}