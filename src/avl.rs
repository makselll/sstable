use std::cmp::Ordering;
use std::io::Error;

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

    pub fn unset(&mut self, key: &str) -> Result<(), Error> {
        self.root = Self::remove(self.root.take(), key);
        Ok(())
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
                            return n.left
                        } else if n.left.is_none() {
                            return n.right
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

    pub fn set(&mut self, key: &str, value: &str) -> Result<(), Error> {
        self.root = Self::insert(self.root.take(), key, value);
        Ok(())
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
        if let Some(n) = node {
            return n.height;
        } else {
            0
        }
    }

    fn get_balance(node: Option<&Box<AVLNode>>) -> i32 {
        if let Some(n) = node {
            return Self::get_height(n.left.as_ref()) - Self::get_height(n.right.as_ref());
        } else {
            0
        }
    }

    fn right_rotate(mut node: Box<AVLNode>) -> Box<AVLNode> {
        println!("Rotate right on node {}", node.key);

        let mut new_main_node = node.left.take().unwrap();

        node.left = new_main_node.right.take();
        node.height =
            1 + Self::get_height(node.left.as_ref()).max(Self::get_height(node.right.as_ref()));

        new_main_node.right = Some(node);
        new_main_node.height = 1 + Self::get_height(new_main_node.left.as_ref())
            .max(Self::get_height(new_main_node.right.as_ref()));
        new_main_node
    }

    fn left_rotate(mut node: Box<AVLNode>) -> Box<AVLNode> {
        println!("Rotate left on node {}", node.key);

        let mut new_main_node = node.right.take().unwrap();

        node.right = new_main_node.left.take();
        node.height =
            1 + Self::get_height(node.right.as_ref()).max(Self::get_height(node.right.as_ref()));

        new_main_node.left = Some(node);
        new_main_node.height = 1 + Self::get_height(new_main_node.left.as_ref())
            .max(Self::get_height(new_main_node.right.as_ref()));
        new_main_node
    }

    fn balance(mut node: Box<AVLNode>) -> Box<AVLNode> {
        node.height =
            1 + Self::get_height(node.left.as_ref()).max(Self::get_height(node.right.as_ref()));
        let balance = Self::get_balance(Some(&node));
        dbg!(&node);

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
