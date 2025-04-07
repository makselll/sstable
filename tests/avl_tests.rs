use sstable::avl::AVLTree;

#[test]
fn valid_tree() {
    let first_pair = ("qw", "first");
    let second_pair = ("q", "second");
    let third_pair = ("qwe", "third");
    let fourth_pair = ("qwer", "fourth");
    let fifth_pair = ("qwert", "fifth");

    let mut tree = AVLTree::new();
    tree.set(first_pair.0, first_pair.1);
    let root = tree.root.as_ref().unwrap();
    assert_eq!(root.key, first_pair.0);
    assert_eq!(root.value, first_pair.1);
    assert_eq!(root.height, 1);

    // Value should be inserted to the left
    tree.set(second_pair.0, second_pair.1);
    let root = tree.root.as_ref().unwrap();
    let root_left = root.left.as_ref().unwrap();
    assert_eq!(root_left.key, second_pair.0);
    assert_eq!(root_left.value, second_pair.1);
    assert_eq!(root.height, 2);

    // Value should be inserted to the right
    tree.set(third_pair.0, third_pair.1);
    let root = tree.root.as_ref().unwrap();
    let root_right = root.right.as_ref().unwrap();
    assert_eq!(root_right.key, third_pair.0);
    assert_eq!(root_right.value, third_pair.1);
    assert_eq!(root.height, 2);

    // ReBalance...
    tree.set(fourth_pair.0, fourth_pair.1);
    tree.set(fifth_pair.0, fifth_pair.1);
    let root = tree.root.as_ref().unwrap();
    assert_eq!(root.key, first_pair.0);
    assert_eq!(root.height, 3);

    assert_eq!(root.left.as_ref().unwrap().key, second_pair.0);
    assert_eq!(root.left.as_ref().unwrap().height, 1);

    let root_right = root.right.as_ref().unwrap();
    assert_eq!(root_right.key, fourth_pair.0);
    assert_eq!(root_right.height, 2);

    let root_right_left = root_right.left.as_ref().unwrap();
    assert_eq!(root_right_left.key, third_pair.0);
    assert_eq!(root_right_left.height, 1);

    let root_right_right = root_right.right.as_ref().unwrap();
    assert_eq!(root_right_right.key, fifth_pair.0);
    assert_eq!(root_right_right.height, 1);

    // Get 
    assert_eq!(tree.get(first_pair.0).unwrap().value, first_pair.1);
    assert_eq!(tree.get(second_pair.0).unwrap().value, second_pair.1);
    assert_eq!(tree.get(third_pair.0).unwrap().value, third_pair.1);
    assert_eq!(tree.get(fourth_pair.0).unwrap().value, fourth_pair.1);
    assert_eq!(tree.get(fifth_pair.0).unwrap().value, fifth_pair.1);
    
    // Unset with left and right node
    tree.unset(fourth_pair.0);
    let root = tree.root.as_ref().unwrap();
    assert_eq!(root.key, first_pair.0);
    assert_eq!(root.height, 3);

    assert_eq!(root.left.as_ref().unwrap().key, second_pair.0);
    assert_eq!(root.left.as_ref().unwrap().height, 1);

    let root_right = root.right.as_ref().unwrap();
    assert_eq!(root_right.key, third_pair.0);
    assert_eq!(root_right.height, 2);
    
    let root_right_right = root_right.right.as_ref().unwrap();
    assert_eq!(root_right_right.key, fifth_pair.0);
    assert_eq!(root_right_right.height, 1);

    // Unset with right node
    tree.unset(third_pair.0);
    let root = tree.root.as_ref().unwrap();
    assert_eq!(root.key, first_pair.0);
    assert_eq!(root.height, 2);

    assert_eq!(root.left.as_ref().unwrap().key, second_pair.0);
    assert_eq!(root.left.as_ref().unwrap().height, 1);

    let root_right = root.right.as_ref().unwrap();
    assert_eq!(root_right.key, fifth_pair.0);
    assert_eq!(root_right.height, 1);
    
    // Unset with left node
    tree.unset(second_pair.0);
    let root = tree.root.as_ref().unwrap();
    assert_eq!(root.key, first_pair.0);
    assert_eq!(root.height, 2);
    
    let root_right = root.right.as_ref().unwrap();
    assert_eq!(root_right.key, fifth_pair.0);
    assert_eq!(root_right.height, 1);
}