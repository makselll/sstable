use std::cmp::Ordering;
use std::io::Error;

#[derive(Debug)]
pub struct AVLNode {
    pub left : Option<Box<AVLNode>>,
    pub right : Option<Box<AVLNode>>,
    pub key: String,
    pub value: String,
    pub height: u32,
}

impl AVLNode {
    pub fn new(key: &str, value: &str, left: Option<Box<AVLNode>>, right: Option<Box<AVLNode>>) -> AVLNode {
        AVLNode{
            left,
            right,
            key: key.to_string(),
            value: value.to_string(),
            height: 0,
        }
    }
}

#[derive(Debug)]
pub struct AVLTree {
    pub root: Option<Box<AVLNode>>,
}

impl AVLTree {
    pub fn new(key: &str, value: &str) -> AVLTree {
        let node = AVLNode::new(key, value, None, None);
        AVLTree {root: Some(Box::new(node))}
    }
    
    pub fn set(&mut self, key: &str, value: &str) -> Result<&AVLNode, Error> {
        let mut current_tree = &mut self.root;
        
        while let Some(current_node) = current_tree{
            match key.cmp(&current_node.key) {
                Ordering::Less => {
                    if current_node.left.is_some() {
                        current_tree = &mut current_node.left
                    } else {
                        current_node.left = Some(Box::new(AVLNode::new(key, value, None, None)));
                        return Ok(current_node.left.as_deref().unwrap())
                    }
                },
                Ordering::Greater => {
                    if current_node.right.is_some() {
                        current_tree = &mut current_node.right
                    } else {
                        current_node.right = Some(Box::new(AVLNode::new(key, value, None, None)));
                        return Ok(current_node.right.as_deref().unwrap())
                    }
                },
                Ordering::Equal => {
                    current_node.value = value.to_string();
                    return Ok(current_node)
                }
            }
        }
    
        Ok(current_tree.as_deref().unwrap())
        
    }

    // pub fn get(&mut self, key: &str,) -> Result<AVLNode, Error> {
    //     Ok()
    // }
    // 
    // pub fn delete(&mut self, key: &str,) -> Result<AVLNode, Error> {
    //     Ok()
    // }
}