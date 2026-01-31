use core::ops::Deref;

use alloc::{
    string::String,
    sync::{Arc, Weak},
};
use spin::Mutex;

use super::KernelObject;

macro_rules! impl_ko {
    ($ty: ident, $target: ty, $(($gen_name: ident: $gen_con: path)),*) => {
        impl<$($gen_name: $gen_con),*> $ty<$($gen_name),*> {
            pub fn name(&self) -> String {
                self.data.name()
            }

            pub fn set_name(&self, name: String) {
                self.data.set_name(name);
            }
        }

        impl<$($gen_name: $gen_con),*> Deref for $ty<$($gen_name),*> {
            type Target = $target;
            fn deref(&self) -> &Self::Target {
                self.inner.deref()
            }
        }
    };
}

pub struct TypedKObject<T: KernelObject> {
    inner: Arc<T>,
    data: Arc<KObjectData>,
}

impl<T: KernelObject> TypedKObject<T> {
    pub fn new(obj: T) -> Self {
        let data = Arc::new(KObjectData::new());

        Self {
            inner: Arc::new(obj),
            data,
        }
    }
}

impl<T: KernelObject> Clone for TypedKObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            data: self.data.clone(),
        }
    }
}

impl_ko!(TypedKObject, T, (T: KernelObject));

impl<T: KernelObject> TypedKObject<T> {
    pub fn as_kobject(&self) -> KObject {
        KObject {
            inner: self.inner.clone(),
            data: self.data.clone(),
        }
    }

    pub fn downgrade(&self) -> WeakTypedKObject<T> {
        WeakTypedKObject {
            inner: Arc::downgrade(&self.inner),
            data: Arc::downgrade(&self.data),
        }
    }

    /// # Safety
    /// If any other TypedKObject or WeakTypedKObject pointers to the same allocation exist,
    /// then they must not be dereferenced or have active borrows for the duration of the returned borrow,
    /// and their inner type must be exactly the same as the inner type of this Rc (including lifetimes).
    /// This is trivially the case if no such pointers exist,
    /// for example immediately after TypedKObject::new.
    pub unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        unsafe { Arc::get_mut_unchecked(&mut self.inner) }
    }
}

impl<T: KernelObject> From<TypedKObject<T>> for KObject {
    fn from(obj: TypedKObject<T>) -> Self {
        obj.as_kobject()
    }
}

pub struct WeakTypedKObject<T: KernelObject> {
    inner: Weak<T>,
    data: Weak<KObjectData>,
}

impl<T: KernelObject> WeakTypedKObject<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<T: KernelObject> Default for WeakTypedKObject<T> {
    fn default() -> Self {
        Self {
            inner: Weak::default(),
            data: Weak::default(),
        }
    }
}

impl<T: KernelObject> Clone for WeakTypedKObject<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T: KernelObject> WeakTypedKObject<T> {
    pub fn upgrade(&self) -> Option<TypedKObject<T>> {
        self.inner
            .upgrade()
            .and_then(|inner| self.data.upgrade().map(|data| TypedKObject { inner, data }))
    }
}

#[derive(Clone)]
pub struct KObject {
    inner: Arc<dyn KernelObject>,
    data: Arc<KObjectData>,
}

impl KObject {
    pub fn new<T: KernelObject>(obj: T) -> Self {
        let data = Arc::new(KObjectData::new());

        Self {
            inner: Arc::new(obj),
            data,
        }
    }
}

impl_ko!(KObject, dyn KernelObject,);

impl KObject {
    pub fn downcast<T: KernelObject>(&self) -> Option<TypedKObject<T>> {
        self.inner
            .clone()
            .downcast_arc::<T>()
            .map(|inner| TypedKObject {
                inner,
                data: self.data.clone(),
            })
            .ok()
    }

    pub fn downgrade(&self) -> WeakKObject {
        WeakKObject {
            inner: Arc::downgrade(&self.inner),
            data: Arc::downgrade(&self.data),
        }
    }
}

#[derive(Clone)]
pub struct WeakKObject {
    inner: Weak<dyn KernelObject>,
    data: Weak<KObjectData>,
}

impl WeakKObject {
    pub fn upgrade(&self) -> Option<KObject> {
        self.inner
            .upgrade()
            .and_then(|inner| self.data.upgrade().map(|data| KObject { inner, data }))
    }
}

struct KObjectData {
    inner: Mutex<KObjectDataInner>,
}

struct KObjectDataInner {
    name: String,
}

impl KObjectData {
    fn new() -> Self {
        Self {
            inner: Mutex::new(KObjectDataInner {
                name: String::new(),
            }),
        }
    }
}

impl KObjectData {
    fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    fn set_name(&self, name: String) {
        self.inner.lock().name = name;
    }
}
