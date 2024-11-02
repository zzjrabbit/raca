use core::{
    fmt::Display,
    ops::{Add, AddAssign, Deref, DerefMut},
};

use alloc::{format, string::String, vec::Vec};

#[derive(Clone, Debug)]
pub struct Path {
    inner: String,
}

impl Path {
    pub fn new<S>(path: S) -> Path
    where
        String: From<S>,
    {
        Path {
            inner: String::from(path),
        }
    }

    pub fn parent(&self) -> Option<Path> {
        if self.inner.is_empty() {
            return None;
        }

        let mut path = self.inner.clone();

        if path.ends_with("/") {
            path.pop();
        }

        while !path.ends_with("/") && !path.is_empty() {
            path.pop();
        }
        Some(Path::new(path))
    }

    pub fn ancestors(&self) -> Vec<Path> {
        let mut ancestors = Vec::new();
        let mut current_path = self.clone();

        while let Some(parent) = current_path.parent() {
            ancestors.push(parent.clone());
            current_path = parent.clone();
        }

        ancestors.reverse();
        ancestors
    }
}

impl<A> Add<A> for Path
where
    String: From<A>,
{
    type Output = Path;
    fn add(self, rhs: A) -> Self::Output {
        Path::new::<String>(format!("{}{}", self.inner, String::from(rhs)))
    }
}

impl<A> AddAssign<A> for Path
where String: From<A> {
    fn add_assign(&mut self, rhs: A) {
        self.inner = format!("{}{}", self.inner, String::from(rhs));
    }
}

impl Deref for Path {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
