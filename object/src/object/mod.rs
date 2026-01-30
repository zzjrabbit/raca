use downcast_rs::{DowncastSync, impl_downcast};

pub use handle::*;
pub use obj::*;
pub use rights::*;

use crate::{Errno, Result};

mod handle;
mod obj;
mod rights;

pub trait KernelObject: DowncastSync + Sync + Send {
    fn type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }

    fn peer(&self) -> Result<KObject> {
        Err(Errno::NotSupported.no_message())
    }
}
impl_downcast!(sync KernelObject);

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    struct TestObject {
        field: u32,
    }
    impl KernelObject for TestObject {}

    #[test]
    fn create_object() {
        let obj = KObject::new(TestObject { field: 0 });
        assert!(obj.downcast_ref::<TestObject>().is_some());
        assert!(obj.type_name().ends_with("TestObject"));
    }

    #[test]
    fn typed_kobject() {
        let obj = TypedKObject::new(TestObject { field: 42 });
        assert!(obj.type_name().ends_with("TestObject"));
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn name() {
        let obj = TypedKObject::new(TestObject { field: 42 });
        assert!(obj.type_name().ends_with("TestObject"));
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn as_kobject() {
        let obj = TypedKObject::new(TestObject { field: 42 });
        let kobj = obj.as_kobject();
        assert!(kobj.downcast_ref::<TestObject>().is_some());
        assert!(kobj.type_name().ends_with("TestObject"));
        assert_eq!(kobj.downcast_ref::<TestObject>().unwrap().field, 42);
    }

    #[test]
    fn weak_typed_kobj() {
        let obj = TypedKObject::new(TestObject { field: 42 });
        let weak = obj.downgrade();
        assert!(weak.upgrade().is_some());
    }

    #[test]
    fn weak_kobj() {
        let obj = KObject::new(TestObject { field: 42 });
        let weak = obj.downgrade();
        assert!(weak.upgrade().is_some());
        assert!(
            weak.upgrade()
                .unwrap()
                .downcast_ref::<TestObject>()
                .is_some()
        );
        assert!(
            weak.upgrade()
                .unwrap()
                .downcast_ref::<TestObject>()
                .unwrap()
                .field
                == 42
        );
    }
}
