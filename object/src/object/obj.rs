use alloc::{string::String, sync::Arc};
use downcast_rs::{DowncastSync, impl_downcast};
use spin::Mutex;

use crate::{Errno, Result};

pub type KObject = Arc<dyn KernelObject>;

pub trait KernelObject: DowncastSync + Sync + Send {
    fn type_name(&self) -> &str;
    fn name(&self) -> String;
    fn set_name(&self, name: String);

    fn peer(&self) -> Result<Arc<dyn KernelObject>> {
        Err(Errno::NotSupported.no_message())
    }
}
impl_downcast!(sync KernelObject);

pub trait Upcast {
    fn upcast(self: &Arc<Self>) -> Arc<dyn KernelObject>;
}

impl<T: KernelObject> Upcast for T {
    fn upcast(self: &Arc<Self>) -> Arc<dyn KernelObject> {
        self.clone()
    }
}

#[derive(Debug, Default)]
pub struct KObjectBase {
    inner: Mutex<KObjectBaseInner>,
}

#[derive(Debug, Default)]
struct KObjectBaseInner {
    name: String,
}

impl KObjectBase {
    pub fn name(&self) -> String {
        self.inner.lock().name.clone()
    }

    pub fn set_name(&self, name: String) {
        self.inner.lock().name = name;
    }
}

#[macro_export]
macro_rules! impl_kobj {
    ($ty: ident $( $f: tt )*) => {
        impl $crate::object::KernelObject for $ty {
            fn type_name(&self) -> &str {
                stringify!($ty)
            }

            fn name(&self) -> ::alloc::string::String {
                self.base.name()
            }

            fn set_name(&self, name: ::alloc::string::String) {
                self.base.set_name(name);
            }

            $($f)*
        }
    };
}

#[macro_export]
macro_rules! new_kobj {
    ({
        $(
            $field: ident $( : $fval: expr )?
        ),*
        $(,)?
    }) => {
        ::alloc::sync::Arc::new(Self {
            $(
                $field $( : $fval )?,
            )*
            base: Default::default(),
        })
    };
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    struct TestObject {
        field: u32,
        base: KObjectBase,
    }
    impl_kobj!(TestObject);

    impl TestObject {
        fn new(field: u32) -> Arc<Self> {
            new_kobj!({
                field,
            })
        }
    }

    #[test]
    fn create_object() {
        let obj: Arc<dyn KernelObject> = TestObject::new(42);
        assert!(obj.downcast_ref::<TestObject>().is_some());
        assert!(obj.type_name().ends_with("TestObject"));
    }

    #[test]
    fn new_kobj() {
        let obj = TestObject::new(42);
        assert_eq!(obj.type_name(), "TestObject");
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn name() {
        let obj = TestObject::new(42);
        assert!(obj.type_name().ends_with("TestObject"));
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn upcast() {
        let obj = TestObject::new(42);
        let kobj = obj.upcast();
        assert!(kobj.downcast_ref::<TestObject>().is_some());
        assert!(kobj.type_name().ends_with("TestObject"));
        assert_eq!(kobj.downcast_ref::<TestObject>().unwrap().field, 42);
    }
}
