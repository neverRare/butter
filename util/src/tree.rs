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
pub struct TreeVec<T>(Vec<Node<T>>);
impl<T> Default for TreeVec<T> {
    fn default() -> Self {
        Self(vec![])
    }
}
impl<T> TreeVec<T> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_capacity(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
    pub fn node_branch(node: T, children: Self) -> Self {
        let Self(mut branch) = children;
        let mut vec = vec![Node {
            content: node,
            len: branch.len(),
        }];
        vec.append(&mut branch);
        Self(vec)
    }
    pub fn node(node: T) -> Self {
        Self(vec![Node::new(node)])
    }
    pub fn push(&mut self, node: T) {
        self.0.push(Node::new(node))
    }
    pub fn append(&mut self, children: &mut Self) {
        self.0.append(&mut children.0)
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
impl<T> Deref for TreeVec<T> {
    type Target = TreeSlice<T>;
    fn deref(&self) -> &Self::Target {
        TreeSlice::from_slice(&self.0)
    }
}
impl<T> DerefMut for TreeVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        TreeSlice::from_mut_slice(&mut self.0)
    }
}
impl<T> IntoIterator for TreeVec<T> {
    type Item = (T, Self);
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}
impl<T: Debug> Debug for TreeVec<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        let tree: &TreeSlice<T> = self;
        tree.fmt(fmt)
    }
}
#[derive(PartialEq, Eq, Hash)]
pub struct TreeSlice<T>([Node<T>]);
impl<T> TreeSlice<T> {
    fn from_slice(slice: &[Node<T>]) -> &Self {
        let ptr = slice as *const [Node<T>] as *const Self;
        unsafe { &*ptr }
    }
    fn from_mut_slice(slice: &mut [Node<T>]) -> &mut Self {
        let ptr = slice as *mut [Node<T>] as *mut Self;
        unsafe { &mut *ptr }
    }
    pub fn iter(&self) -> Iter<T> {
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
impl<'a, T> Default for &'a TreeSlice<T> {
    fn default() -> Self {
        TreeSlice::from_slice(&[])
    }
}
impl<'a, T> Default for &'a mut TreeSlice<T> {
    fn default() -> Self {
        TreeSlice::from_mut_slice(&mut [])
    }
}
impl<'a, T> IntoIterator for &'a TreeSlice<T> {
    type Item = (&'a T, &'a TreeSlice<T>);
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}
impl<T: Debug> Debug for TreeSlice<T> {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        fmt.debug_list().entries(self.iter()).finish()
    }
}
pub struct Iter<'a, T>(&'a TreeSlice<T>);
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (&'a T, &'a TreeSlice<T>);
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
impl<'a, T> FusedIterator for Iter<'a, T> {}
pub struct IntoIter<T>(TreeVec<T>);
impl<T> Iterator for IntoIter<T> {
    type Item = (T, TreeVec<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let self_vec = &mut (self.0).0;
        if self_vec.is_empty() {
            None
        } else {
            let mut vec = vec![];
            swap(self_vec, &mut vec);
            let (content, children, rest) = TreeVec(vec).into_first_and_rest().unwrap();
            *self_vec = rest.0;
            Some((content, children))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.0).0.len();
        (1.min(len), Some(len))
    }
}
impl<T> FusedIterator for IntoIter<T> {}
