use std::iter::FusedIterator;
use std::mem::swap;
use std::ops::Deref;
use std::ops::DerefMut;

#[derive(Clone, PartialEq, Eq, Hash)]
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
        let mut vec = vec![(content, children.len())];
        vec.append(&mut children);
        TreeVec(vec)
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct TreeRef<'a, T> {
    pub content: &'a T,
    pub children: &'a TreeSlice<T>,
}
#[derive(PartialEq, Eq, Hash)]
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
        vec.push((content, children.len()));
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
            let (content, len) = vec.remove(0);
            vec.truncate(len);
            Some(Tree {
                content,
                children: Self(vec),
            })
        }
    }
    pub fn into_first_and_rest(self) -> Option<(Tree<T>, Self)> {
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
#[derive(PartialEq, Eq, Hash)]
pub struct TreeSlice<T>([(T, usize)]);
impl<T> TreeSlice<T> {
    fn from_slice(slice: &[(T, usize)]) -> &Self {
        let ptr = slice as *const [(T, usize)] as *const Self;
        unsafe { &*ptr }
    }
    fn from_mut_slice(slice: &mut [(T, usize)]) -> &mut Self {
        let ptr = slice as *mut [(T, usize)] as *mut Self;
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
    pub fn to_tree_vec(&self) -> TreeVec<T>
    where
        T: Clone,
    {
        let Self(slice) = self;
        TreeVec(slice.to_vec())
    }
    pub fn first(&self) -> Option<TreeRef<T>> {
        let arr = &self.0;
        if arr.is_empty() {
            None
        } else {
            let (content, len) = &arr[0];
            let children = &arr[1..1 + len];
            Some(TreeRef {
                content,
                children: Self::from_slice(children),
            })
        }
    }
    pub fn first_and_rest(&self) -> Option<(TreeRef<T>, &Self)> {
        let arr = &self.0;
        if arr.is_empty() {
            None
        } else {
            let (content, len) = &arr[0];
            let children = &arr[1..1 + len];
            let rest = &arr[1 + len..];
            let tree = TreeRef {
                content,
                children: Self::from_slice(children),
            };
            Some((tree, Self::from_slice(rest)))
        }
    }
    pub fn first_mut(&mut self) -> Option<TreeMutRef<T>> {
        let arr = &mut self.0;
        if arr.is_empty() {
            None
        } else {
            let (first, rest) = arr.split_at_mut(1);
            let (content, len) = &mut first[0];
            let children = &mut rest[..*len];
            Some(TreeMutRef {
                content,
                children: Self::from_mut_slice(children),
            })
        }
    }
    pub fn first_and_rest_mut(&mut self) -> Option<(TreeMutRef<T>, &mut Self)> {
        let arr = &mut self.0;
        if arr.is_empty() {
            None
        } else {
            let (first, rest) = arr.split_at_mut(1);
            let (content, len) = &mut first[0];
            let (children, rest) = rest.split_at_mut(*len);
            let tree = TreeMutRef {
                content,
                children: Self::from_mut_slice(children),
            };
            Some((tree, Self::from_mut_slice(rest)))
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
    type Item = TreeRef<'a, T>;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}
pub struct Iter<'a, T>(&'a TreeSlice<T>);
impl<'a, T> Iterator for Iter<'a, T> {
    type Item = TreeRef<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        let Self(tree) = self;
        tree.first_and_rest().map(|(tree, rest)| {
            self.0 = rest;
            tree
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let Self(TreeSlice(slice)) = self;
        let len = slice.len();
        (1.min(len), Some(len))
    }
}
impl<'a, T> FusedIterator for Iter<'a, T> {}
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
        $crate::tree::TreeVec::new()
    };
    ($($content:expr $(=> { $($tree:tt)* })?),* $(,)?) => {{
        let mut tree_vec = $crate::tree::TreeVec::new();
        $(
            let tree = $crate::tree::Tree {
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
