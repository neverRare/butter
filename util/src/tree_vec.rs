use std::fmt::Debug;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::mem::swap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
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
        let mut vec = Vec::with_capacity(1 + children.len());
        vec.push((content, children.len()));
        vec.append(&mut children);
        TreeVec(vec)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct TreeRef<'a, T> {
    pub content: &'a T,
    pub children: &'a TreeSlice<T>,
}
#[derive(PartialEq, Eq, Hash, Debug)]
pub struct TreeMutRef<'a, T> {
    pub content: &'a mut T,
    pub children: &'a mut TreeSlice<T>,
}
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TreeVec<T>(Vec<(T, usize)>);
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
        vec.reserve(1 + children.len());
        vec.push((content, children.len()));
        vec.append(&mut children);
    }
    pub fn append(&mut self, other: &mut Self) {
        self.0.append(&mut other.0);
    }
    fn into_first_and_rest(self) -> Option<(Tree<T>, Self)> {
        let Self(mut vec) = self;
        if vec.is_empty() {
            None
        } else {
            let (content, len) = vec.remove(0);
            let rest = vec.split_off(len);
            let tree = Tree {
                content,
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
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        TreeSlice::fmt(self, formatter)
    }
}
#[derive(PartialEq, Eq, Hash)]
pub struct TreeSlice<T>([(T, usize)]);
impl<T> TreeSlice<T> {
    fn from_slice(slice: &[(T, usize)]) -> &Self {
        let ptr = slice as *const _ as *const Self;
        unsafe { &*ptr }
    }
    fn from_mut_slice(slice: &mut [(T, usize)]) -> &mut Self {
        let ptr = slice as *mut _ as *mut Self;
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
    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.into_iter()
    }
    pub fn to_tree_vec(&self) -> TreeVec<T>
    where
        T: Clone,
    {
        let Self(slice) = self;
        TreeVec(slice.to_vec())
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
    type Item = TreeRef<'a, T>;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        let ptr = self.0.as_ptr();
        Iter {
            start: ptr,
            end: unsafe { ptr.add(self.0.len()) },
            phantom: PhantomData,
        }
    }
}
impl<'a, T> IntoIterator for &'a mut TreeSlice<T> {
    type Item = TreeMutRef<'a, T>;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        let ptr = self.0.as_mut_ptr();
        IterMut {
            start: ptr,
            end: unsafe { ptr.add(self.0.len()) },
            phantom: PhantomData,
        }
    }
}
impl<T: Debug> Debug for TreeSlice<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.debug_list().entries(self).finish()
    }
}
pub struct Iter<'a, T> {
    start: *const (T, usize),
    end: *const (T, usize),
    phantom: PhantomData<&'a (T, usize)>,
}
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = TreeRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let (content, len) = &*self.start;
                let children = std::slice::from_raw_parts(self.start.add(1), *len);
                self.start = self.start.add(*len + 1);
                Some(TreeRef {
                    content,
                    children: TreeSlice::from_slice(children),
                })
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { self.start.offset_from(self.end) as usize };
        (1.min(len), Some(len))
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
pub struct IterMut<'a, T> {
    start: *mut (T, usize),
    end: *mut (T, usize),
    phantom: PhantomData<&'a mut (T, usize)>,
}
impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = TreeMutRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let (content, len) = &mut *self.start;
                let children = std::slice::from_raw_parts_mut(self.start.add(1), *len);
                self.start = self.start.add(*len + 1);
                Some(TreeMutRef {
                    content,
                    children: TreeSlice::from_mut_slice(children),
                })
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { self.start.offset_from(self.end) as usize };
        (1.min(len), Some(len))
    }
}
pub struct IntoIter<T>(TreeVec<T>);
impl<T> Iterator for IntoIter<T> {
    type Item = Tree<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let Self(TreeVec(self_vec)) = self;
        if self_vec.is_empty() {
            None
        } else {
            let mut vec = vec![];
            swap(self_vec, &mut vec);
            let (tree, TreeVec(rest)) = TreeVec(vec).into_first_and_rest().unwrap();
            *self_vec = rest;
            Some(tree)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(TreeVec(vec)) = self;
        let len = vec.len();
        (1.min(len), Some(len))
    }
}
impl<T> FusedIterator for IntoIter<T> {}
#[macro_export]
macro_rules! tree_vec {
    () => {
        $crate::tree_vec::TreeVec::new()
    };
    ($($content:expr $(=> { $($tree:tt)* })?),* $(,)?) => {{
        let mut tree_vec = $crate::tree_vec::TreeVec::new();
        $(
            let tree = $crate::tree_vec::Tree {
                content: $content,
                children: $crate::tree_vec!($($($tree)*)?),
            };
            tree_vec.push(tree);
        )*
        tree_vec
    }};
}
#[cfg(test)]
mod test {
    #[test]
    fn tree_vec() {
        let tree_vec = tree_vec! {
            7 => {
                2,
                10,
                6 => {
                    5,
                    11,
                },
            },
            5,
        };
        let mut iter = tree_vec.iter();
        let tree = iter.next().unwrap();
        assert_eq!(*tree.content, 7);
        {
            let mut iter = tree.children.iter();
            let tree = iter.next().unwrap();
            assert_eq!(*tree.content, 2);
            assert!(tree.children.is_empty());
            let tree = iter.next().unwrap();
            assert_eq!(*tree.content, 10);
            assert!(tree.children.is_empty());
            let tree = iter.next().unwrap();
            assert_eq!(*tree.content, 6);
            {
                let mut iter = tree.children.iter();
                let tree = iter.next().unwrap();
                assert_eq!(*tree.content, 5);
                assert!(tree.children.is_empty());
                let tree = iter.next().unwrap();
                assert_eq!(*tree.content, 11);
                assert!(tree.children.is_empty());
                assert!(iter.next().is_none());
            }
            assert!(iter.next().is_none());
        }
        let tree = iter.next().unwrap();
        assert_eq!(*tree.content, 5);
        assert!(tree.children.is_empty());
        assert!(iter.next().is_none());
    }
}
