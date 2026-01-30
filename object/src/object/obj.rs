use core::ops::Deref;

use alloc::{string::String, sync::Arc};
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

#[derive(Clone)]
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

impl_ko!(TypedKObject, T, (T: KernelObject));

impl<T: KernelObject> TypedKObject<T> {
    pub fn as_kobject(&self) -> KObject {
        KObject {
            inner: self.inner.clone(),
            data: self.data.clone(),
        }
    }
}

impl<T: KernelObject> From<TypedKObject<T>> for KObject {
    fn from(obj: TypedKObject<T>) -> Self {
        obj.as_kobject()
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
