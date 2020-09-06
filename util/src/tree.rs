use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::mem::transmute;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, PartialEq, Eq, Hash)]
struct Node<T> {
    content: T,
    len: usize,
}
impl<T> Node<T> {
    fn new(content: T) -> Self {
        Self { content, len: 0 }
    }
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct BigTree<T>(Vec<Node<T>>);
impl<T> Default for BigTree<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
impl<T> BigTree<T> {
    pub fn branch(leaf: T, branch: Self) -> Self {
        let Self(mut branch) = branch;
        let mut vec = vec![Node {
            content: leaf,
            len: branch.len(),
        }];
        vec.append(&mut branch);
        Self(vec)
    }
    pub fn leaf(leaf: T) -> Self {
        Self(vec![Node::new(leaf)])
    }
    pub fn push(&mut self, leaf: T) {
        self.0.push(Node::new(leaf))
    }
    pub fn append(&mut self, branch: &mut Self) {
        self.0.append(&mut branch.0)
    }
}
impl<T> Deref for BigTree<T> {
    type Target = Tree<T>;
    fn deref(&self) -> &Self::Target {
        Tree::from_slice(&self.0)
    }
}
impl<T> DerefMut for BigTree<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Tree::from_mut_slice(&mut self.0)
    }
}
impl<T: Debug> Debug for BigTree<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let tree: &Tree<T> = self;
        tree.fmt(fmt)
    }
}
#[derive(PartialEq, Eq, Hash)]
pub struct Tree<T>([Node<T>]);
impl<T> Tree<T> {
    fn from_slice(slice: &[Node<T>]) -> &Self {
        unsafe { transmute(slice) }
    }
    fn from_mut_slice(slice: &mut [Node<T>]) -> &mut Self {
        unsafe { transmute(slice) }
    }
    pub fn iter(&self) -> TreeIter<T> {
        self.into_iter()
    }
}
impl<'a, T> Default for &'a Tree<T> {
    fn default() -> Self {
        Tree::from_slice(&[])
    }
}
impl<'a, T> Default for &'a mut Tree<T> {
    fn default() -> Self {
        Tree::from_mut_slice(&mut [])
    }
}
impl<'a, T> IntoIterator for &'a Tree<T> {
    type Item = (&'a T, &'a Tree<T>);
    type IntoIter = TreeIter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIter(&self.0)
    }
}
impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}
pub struct TreeIter<'a, T>(&'a [Node<T>]);
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = (&'a T, &'a Tree<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let arr = self.0;
        if arr.is_empty() {
            None
        } else {
            let first = &arr[0];
            let len = first.len;
            self.0 = &arr[1 + len..];
            Some((&first.content, Tree::from_slice(&arr[1..1 + len])))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.0.len();
        (1.min(len), Some(len))
    }
}
