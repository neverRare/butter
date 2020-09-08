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
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Tree<T> {
    pub content: T,
    pub children: TreeVec<T>,
}
impl<T> Tree<T> {
    pub fn new(content: T) -> Self {
        Self {
            content,
            children: TreeVec::new(),
        }
    }
    pub fn with_children(content: T, children: TreeVec<T>) -> Self {
        Self { content, children }
    }
    pub fn into_tree_vec(self) -> TreeVec<T> {
        let Self {
            content,
            children: TreeVec(mut children),
        } = self;
        let mut vec = vec![Node {
            content,
            len: children.len(),
        }];
        vec.append(&mut children);
        TreeVec(vec)
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
    pub fn push(&mut self, tree: Tree<T>) {
        let Tree {
            content,
            children: Self(mut children),
        } = tree;
        let Self(vec) = self;
        vec.push(Node {
            content,
            len: children.len(),
        });
        vec.append(&mut children);
    }
    pub fn append(&mut self, Self(children): &mut Self) {
        self.0.append(children);
    }
    pub fn into_first(self) -> Option<Tree<T>> {
        let Self(mut vec) = self;
        if vec.is_empty() {
            None
        } else {
            let first = vec.remove(0);
            let len = first.len;
            vec.truncate(len);
            Some(Tree {
                content: first.content,
                children: Self(vec),
            })
        }
    }
    pub fn into_first_and_rest(self) -> Option<(Tree<T>, Self)> {
        let Self(mut vec) = self;
        if vec.is_empty() {
            None
        } else {
            let first = vec.remove(0);
            let len = first.len;
            let rest = vec.split_off(len);
            let tree = Tree {
                content: first.content,
                children: Self(vec),
            };
            Some((tree, Self(rest)))
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
    type Item = Tree<T>;
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
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn total(&self) -> usize {
        self.0.len()
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
    type Item = Tree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let self_vec = &mut (self.0).0;
        if self_vec.is_empty() {
            None
        } else {
            let mut vec = vec![];
            swap(self_vec, &mut vec);
            let (tree, rest) = TreeVec(vec).into_first_and_rest().unwrap();
            *self_vec = rest.0;
            Some(tree)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = (self.0).0.len();
        (1.min(len), Some(len))
    }
}
impl<T> FusedIterator for IntoIter<T> {}
