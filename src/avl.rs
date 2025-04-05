use std::cmp::Ordering;
use std::sync::{Arc, RwLock};
use std::thread::sleep;
use std::time::Duration;
use std::mem::size_of;
use crate::idx::IDX;

#[derive(Debug)]
pub struct AVLNode {
    pub left: Option<Box<AVLNode>>,
    pub right: Option<Box<AVLNode>>,
    pub key: String,
    pub value: String,
    pub height: i32,
}

impl AVLNode {
    pub fn new(
        key: &str,
        value: &str,
        left: Option<Box<AVLNode>>,
        right: Option<Box<AVLNode>>,
    ) -> AVLNode {
        AVLNode {
            left,
            right,
            key: key.to_string(),
            value: value.to_string(),
            height: 1,
        }
    }
}

#[derive(Debug)]
pub struct AVLTree {
    pub root: Option<Box<AVLNode>>,
}

impl AVLTree {
    pub fn new() -> AVLTree {
        AVLTree { root: None }
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn get(&self, key: &str) -> Option<&AVLNode> {
        let mut root_node = self.root.as_ref();

        while let Some(current_node) = root_node {
            match key.cmp(current_node.key.as_str()) {
                Ordering::Equal => {
                    return Some(current_node);
                }
                Ordering::Less => {
                    root_node = current_node.left.as_ref();
                }
                Ordering::Greater => {
                    root_node = current_node.right.as_ref();
                }
            }
        }

        None
    }

    pub fn unset(&mut self, key: &str) {
        self.root = Self::remove(self.root.take(), key);
    }

    fn get_largest_node(node: &mut Option<Box<AVLNode>>) -> Option<Box<AVLNode>> {
        if let Some(n) = node {
            return if n.right.is_none() {
                node.take() // извлекаем самый правый узел
            } else {
                Self::get_largest_node(&mut n.right)
            }
        }
        None
    }

    fn remove(node: Option<Box<AVLNode>>, key: &str) -> Option<Box<AVLNode>> {
        match node {
            Some(mut n) => {
                match key.cmp(&n.key) {
                    Ordering::Less => {
                        n.left = Self::remove(n.left.take(), key);
                    }
                    Ordering::Greater => {
                        n.right = Self::remove(n.right.take(), key);
                    }
                    Ordering::Equal => {
                        if n.right.is_none() {
                            return n.left;
                        } else if n.left.is_none() {
                            return n.right;
                        }

                        let mut largest_left_node = Self::get_largest_node(&mut n.left).unwrap();
                        largest_left_node.left = n.left.take();
                        largest_left_node.right = n.right.take();
                        n = largest_left_node;
                    }
                }
                Some(Self::balance(n))
            }
            None => None,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        self.root = Self::insert(self.root.take(), key, value);
    }

    fn insert(node: Option<Box<AVLNode>>, key: &str, value: &str) -> Option<Box<AVLNode>> {
        match node {
            Some(mut n) => {
                match key.cmp(&n.key) {
                    Ordering::Less => {
                        n.left = Self::insert(n.left.take(), key, value);
                    }
                    Ordering::Greater => {
                        n.right = Self::insert(n.right.take(), key, value);
                    }
                    Ordering::Equal => {
                        n.value = value.to_string();
                        return Some(n);
                    }
                }
                Some(Self::balance(n))
            }
            None => Some(Box::new(AVLNode::new(key, value, None, None))),
        }
    }

    fn get_height(node: Option<&Box<AVLNode>>) -> i32 {
        node.map_or(0, |n| n.height)
    }

    fn get_balance(node: Option<&Box<AVLNode>>) -> i32 {
        node.map_or(0, |n| {
            Self::get_height(n.left.as_ref()) - Self::get_height(n.right.as_ref())
        })
    }

    fn right_rotate(mut node: Box<AVLNode>) -> Box<AVLNode> {
        let mut new_main_node = node.left.take().unwrap();

        node.left = new_main_node.right.take();
        Self::update_height(&mut node);

        new_main_node.right = Some(node);
        Self::update_height(&mut new_main_node);

        new_main_node
    }

    fn left_rotate(mut node: Box<AVLNode>) -> Box<AVLNode> {
        let mut new_main_node = node.right.take().unwrap();

        node.right = new_main_node.left.take();
        Self::update_height(&mut node);

        new_main_node.left = Some(node);
        Self::update_height(&mut new_main_node);

        new_main_node
    }

    fn update_height(node: &mut Box<AVLNode>) {
        node.height =
            1 + Self::get_height(node.left.as_ref()).max(Self::get_height(node.right.as_ref()));
    }

    fn balance(mut node: Box<AVLNode>) -> Box<AVLNode> {
        Self::update_height(&mut node);
        let balance = Self::get_balance(Some(&node));

        // Left Left
        if balance > 1 && Self::get_balance(node.left.as_ref()) >= 0 {
            return Self::right_rotate(node);
        }

        // Left Right
        if balance > 1 && Self::get_balance(node.left.as_ref()) < 0 {
            node.left = Some(Self::left_rotate(node.left.unwrap()));
            return Self::right_rotate(node);
        }

        // Right Right
        if balance < -1 && Self::get_balance(node.right.as_ref()) <= 0 {
            return Self::left_rotate(node);
        }

        // Right Left
        if balance < -1 && Self::get_balance(node.right.as_ref()) > 0 {
            node.right = Some(Self::right_rotate(node.right.unwrap()));
            return Self::left_rotate(node);
        }

        node
    }
}

pub struct AVLTreeSingleton {
    instance: RwLock<AVLTree>,
}

impl AVLTreeSingleton {
    pub fn new() -> AVLTreeSingleton {
        AVLTreeSingleton { 
            instance: RwLock::new(AVLTree::new()),
        }
    }
    
    pub fn get_instance(&self) -> &RwLock<AVLTree> {
        &self.instance
    }
}

fn calculate_size(node: &Option<Box<AVLNode>>) -> usize {
    match node {
        Some(n) => {
            let node_size = size_of::<AVLNode>()
                + n.key.len()
                + n.value.len();
            node_size + calculate_size(&n.left) + calculate_size(&n.right)
        }
        None => 0,
    }
}

pub fn check_size(tree: Arc<AVLTreeSingleton>) {
    let tree = tree.get_instance();
    loop {
        sleep(Duration::from_secs(5));
        let current_tree = tree.read();
        let megabytes =  calculate_size(&current_tree.unwrap().root) as f64 / 1_048_576_f64;
        println!("AVL Tree Size > {megabytes:.2} MB");
        if megabytes > 1f64 {
            println!("AVL Tree Size has reached the limit, lets save it to the disk");
            
            let mut tree = tree.write().unwrap();
            let idx = IDX::new();
            idx.fill_from_avl(&tree).unwrap();
            
            tree.clear();
            dbg!(&tree);
            
        }
    }
}