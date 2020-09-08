use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::iter::FusedIterator;
use std::mem::swap;
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
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
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
    pub fn into_first(self) -> Option<(T, Self)> {
        let mut vec = self.0;
        if vec.is_empty() {
            None
        } else {
            let first = vec.remove(0);
            let len = first.len;
            vec.truncate(len);
            Some((first.content, Self(vec)))
        }
    }
    pub fn into_first_and_rest(self) -> Option<(T, Self, Self)> {
        let mut vec = self.0;
        if vec.is_empty() {
            None
        } else {
            let first = vec.remove(0);
            let len = first.len;
            let rest = vec.split_off(len);
            Some((first.content, Self(vec), Self(rest)))
        }
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
impl<T> IntoIterator for BigTree<T> {
    type Item = (T, Self);
    type IntoIter = TreeIntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        TreeIntoIter(self)
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
        let ptr = slice as *const [Node<T>] as *const Tree<T>;
        unsafe { &*ptr }
    }
    fn from_mut_slice(slice: &mut [Node<T>]) -> &mut Self {
        let ptr = slice as *mut [Node<T>] as *mut Tree<T>;
        unsafe { &mut *ptr }
    }
    pub fn iter(&self) -> TreeIter<T> {
        self.into_iter()
    }
    pub fn first(&self) -> Option<(&T, &Self)> {
        let arr = &self.0;
        if arr.is_empty() {
            None
        } else {
            let first = &arr[0];
            let children = &arr[1..1 + first.len];
            Some((&first.content, Self::from_slice(children)))
        }
    }
    pub fn first_and_rest(&self) -> Option<(&T, &Self, &Self)> {
        let arr = &self.0;
        if arr.is_empty() {
            None
        } else {
            let first = &arr[0];
            let children = &arr[1..1 + first.len];
            let rest = &arr[1 + first.len..];
            Some((
                &first.content,
                Self::from_slice(children),
                Self::from_slice(rest),
            ))
        }
    }
    pub fn first_mut(&mut self) -> Option<(&mut T, &mut Self)> {
        let arr = &mut self.0;
        if arr.is_empty() {
            None
        } else {
            let (first, rest) = arr.split_at_mut(1);
            let first = &mut first[0];
            let children = &mut rest[..first.len];
            Some((&mut first.content, Self::from_mut_slice(children)))
        }
    }
    pub fn first_and_rest_mut(&mut self) -> Option<(&mut T, &mut Self, &mut Self)> {
        let arr = &mut self.0;
        if arr.is_empty() {
            None
        } else {
            let (first, rest) = arr.split_at_mut(1);
            let first = &mut first[0];
            let (children, rest) = rest.split_at_mut(first.len);
            Some((
                &mut first.content,
                Self::from_mut_slice(children),
                Self::from_mut_slice(rest),
            ))
        }
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
        TreeIter(self)
    }
}
impl<T: Debug> Debug for Tree<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}
pub struct TreeIter<'a, T>(&'a Tree<T>);
impl<'a, T> Iterator for TreeIter<'a, T> {
    type Item = (&'a T, &'a Tree<T>);
    fn next(&mut self) -> Option<Self::Item> {
        self.0.first_and_rest().map(|(content, children, rest)| {
            self.0 = rest;
            (content, children)
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.0).0.len();
        (1.min(len), Some(len))
    }
}
impl<'a, T> FusedIterator for TreeIter<'a, T> {}
pub struct TreeIntoIter<T>(BigTree<T>);
impl<T> Iterator for TreeIntoIter<T> {
    type Item = (T, BigTree<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let self_vec = &mut (self.0).0;
        if self_vec.is_empty() {
            None
        } else {
            let mut vec = vec![];
            swap(self_vec, &mut vec);
            let (content, children, rest) = BigTree(vec).into_first_and_rest().unwrap();
            *self_vec = rest.0;
            Some((content, children))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.0).0.len();
        (1.min(len), Some(len))
    }
}
impl<T> FusedIterator for TreeIntoIter<T> {}
