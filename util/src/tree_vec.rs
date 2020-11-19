use std::fmt::Debug;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::mem::swap;
use std::ops::Deref;
use std::ops::DerefMut;

// Side note for unsafe codes written here.
//
// TreeVec<T> is internally represented by a Vec of `(T, usize)`. TreeSlice is
// just a slice of TreeVec or an empty slice.
//
// The representation can be explained further through example:
//
// ```
// [("A", 3), ("A.A", 1), ("A.A.A", 0), ("A.B", 0), ("B", 1), ("B.A", 0), ("C", 0)]
// ```
//
// The first tree have content "A" and have descendants
// `[("A.A", 1), ("A.A.A", 0), ("A.B", 0)]`, the length is determined by the
// number next to "A", which is 3. The remaining slice
// `[("B", 1), ("B.A", 0), ("C", 0)]` contains the remaining trees of the slice.
// Both the descendants and the remaining slice can be traversed further with
// the same way.
//
// The validity comes when slicing the descendants: it must never overflow.
//
// Private codes, especially unsafe codes, may assume the TreeVec and TreeSlice
// are always valid. Hence, it must be always upheld.

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
// Methods `from_vec`, `into_vec`, `as_vec`, and `as_vec_mut` should be used
// instead of directly accessing .0 to better reason out the validity of the
// underlying vector (some methods are marked as unsafe)
#[derive(Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TreeVec<T>(Vec<(T, usize)>);
impl<T> Default for TreeVec<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> TreeVec<T> {
    /// # Safety
    ///
    /// The vector must represent a valid internal representation of TreeVec
    unsafe fn from_vec(vec: Vec<(T, usize)>) -> Self {
        Self(vec)
    }
    pub fn new() -> Self {
        unsafe { Self::from_vec(vec![]) }
    }
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe { Self::from_vec(Vec::with_capacity(capacity)) }
    }
    pub fn push(&mut self, tree: Tree<T>) {
        unsafe {
            let vec = self.as_vec_mut();
            let mut children = tree.children.into_vec();
            vec.reserve(1 + children.len());
            vec.push((tree.content, children.len()));
            vec.append(&mut children);
        }
    }
    pub fn append(&mut self, other: &mut Self) {
        unsafe {
            self.as_vec_mut().append(other.as_vec_mut());
        }
    }
    fn into_vec(self) -> Vec<(T, usize)> {
        self.0
    }
    fn as_vec(&self) -> &Vec<(T, usize)> {
        &self.0
    }
    /// # Safety
    ///
    /// The borrowed vector must maintain a valid internal representation of
    /// TreeVec
    unsafe fn as_vec_mut(&mut self) -> &mut Vec<(T, usize)> {
        &mut self.0
    }
    fn into_first_and_rest(self) -> Option<(Tree<T>, Self)> {
        let mut vec = self.into_vec();
        if vec.is_empty() {
            None
        } else {
            let (content, len) = vec.remove(0);
            let rest = vec.split_off(len);
            unsafe {
                let tree = Tree {
                    content,
                    children: Self::from_vec(vec),
                };
                Some((tree, Self::from_vec(rest)))
            }
        }
    }
}
impl<T> Deref for TreeVec<T> {
    type Target = TreeSlice<T>;
    fn deref(&self) -> &Self::Target {
        unsafe { TreeSlice::from_slice(&self.0) }
    }
}
impl<T> DerefMut for TreeVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { TreeSlice::from_mut_slice(&mut self.0) }
    }
}
impl<T> IntoIterator for TreeVec<T> {
    type Item = Tree<T>;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self)
    }
}
impl<T> Extend<Tree<T>> for TreeVec<T> {
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = Tree<T>>,
    {
        for tree in iter {
            self.push(tree);
        }
    }
}
impl<T: Debug> Debug for TreeVec<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        TreeSlice::fmt(self, formatter)
    }
}
// Methods `from_slice`, `from_mut_slice`, `as_slice`, and `as_slice_mut` should
// be used instead of directly accessing .0 to better reason out the validity of
// the underlying slice (some methods are marked as unsafe)
#[derive(PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TreeSlice<T>([(T, usize)]);
impl<T> TreeSlice<T> {
    /// # Safety
    ///
    /// The slice must represent a valid internal representation of TreeSlice
    unsafe fn from_slice(slice: &[(T, usize)]) -> &Self {
        let ptr = slice as *const _ as *const Self;
        &*ptr
    }
    /// # Safety
    ///
    /// The slice must represent a valid internal representation of TreeSlice
    unsafe fn from_mut_slice(slice: &mut [(T, usize)]) -> &mut Self {
        let ptr = slice as *mut _ as *mut Self;
        &mut *ptr
    }
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    pub fn len(&self) -> usize {
        self.iter().count()
    }
    pub fn total(&self) -> usize {
        self.as_slice().len()
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
        unsafe { TreeVec::from_vec(self.as_slice().to_vec()) }
    }
    fn as_slice(&self) -> &[(T, usize)] {
        &self.0
    }
    /// # Safety
    ///
    /// The borrowed slice must stay a valid internal representation of
    /// TreeSlice. The easiest way is to keep the descendant counts constant.
    unsafe fn as_slice_mut(&mut self) -> &mut [(T, usize)] {
        &mut self.0
    }
}
impl<'a, T> Default for &'a TreeSlice<T> {
    fn default() -> Self {
        unsafe { TreeSlice::from_slice(&[]) }
    }
}
impl<'a, T> Default for &'a mut TreeSlice<T> {
    fn default() -> Self {
        unsafe { TreeSlice::from_mut_slice(&mut []) }
    }
}
impl<'a, T> IntoIterator for &'a TreeSlice<T> {
    type Item = TreeRef<'a, T>;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        Iter::new(self)
    }
}
impl<'a, T> IntoIterator for &'a mut TreeSlice<T> {
    type Item = TreeMutRef<'a, T>;
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        IterMut::new(self)
    }
}
impl<T: Debug> Debug for TreeSlice<T> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.debug_list().entries(self).finish()
    }
}
// Iter is considered valid if it satisfies all of the following
//
// - `start` and `end` is non-null and aligned
// - if `start != end` then `start` points to a member of a slice
// - `end` either points to a member of a slice or to the last member with an
//    offset of 1. Because of this, it should never be dereferenced
// - a region from `start` up to `end` (exclusive) represents a slice
// - the slice represents valid internal representation of TreeSlice
//
// `Iter::new` provides valid abstraction
pub struct Iter<'a, T> {
    start: *const (T, usize),
    end: *const (T, usize),
    phantom: PhantomData<&'a (T, usize)>,
}
impl<'a, T> Iter<'a, T> {
    fn new(slice: &'a TreeSlice<T>) -> Self {
        let range = slice.as_slice().as_ptr_range();
        Self {
            start: range.start,
            end: range.end,
            phantom: PhantomData,
        }
    }
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
// the validity of IterMut is the same as the Iter, with addition that the slice
// must not be aliased.
//
// `IterMut::new` provides valid abstraction
pub struct IterMut<'a, T> {
    start: *mut (T, usize),
    end: *mut (T, usize),
    phantom: PhantomData<&'a mut (T, usize)>,
}
impl<'a, T> IterMut<'a, T> {
    fn new(slice: &'a mut TreeSlice<T>) -> Self {
        unsafe {
            let range = slice.as_slice_mut().as_mut_ptr_range();
            IterMut {
                start: range.start,
                end: range.end,
                phantom: PhantomData,
            }
        }
    }
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
        unsafe {
            let self_vec = self.0.as_vec_mut();
            if self_vec.is_empty() {
                None
            } else {
                let mut vec = vec![];
                swap(self_vec, &mut vec);
                let (tree, rest) = TreeVec::from_vec(vec).into_first_and_rest().unwrap();
                let rest = rest.into_vec();
                *self_vec = rest;
                Some(tree)
            }
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let vec = self.0.as_vec();
        let len = vec.len();
        (1.min(len), Some(len))
    }
}
impl<T> FusedIterator for IntoIter<T> {}
#[macro_export]
macro_rules! join_trees {
    () => {
        $crate::tree_vec::TreeVec::new()
    };
    ($($branch:expr),+ $(,)?) => {{
        let mut tree_vec = $crate::tree_vec::TreeVec::new();
        $(
            tree_vec.push($branch);
        )+
        tree_vec
    }};
}
#[macro_export]
macro_rules! tree_vec {
    () => {
        $crate::tree_vec::TreeVec::new()
    };
    ($($content:expr $(=> { $($tree:tt)* })?),+ $(,)?) => {{
        let mut tree_vec = $crate::tree_vec::TreeVec::new();
        $(
            let tree = $crate::tree_vec::Tree {
                content: $content,
                children: $crate::tree_vec!($($($tree)*)?),
            };
            tree_vec.push(tree);
        )+
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
