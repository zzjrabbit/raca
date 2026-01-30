use downcast_rs::{DowncastSync, impl_downcast};

pub use obj::*;

mod obj;

pub trait KernelObject: DowncastSync + Sync + Send {
    fn type_name(&self) -> &'static str {
        core::any::type_name::<Self>()
    }
}
impl_downcast!(sync KernelObject);

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn create_object() {
        struct TestObject {}
        impl KernelObject for TestObject {}

        let obj = KObject::new(TestObject {});
        assert!(obj.downcast_ref::<TestObject>().is_some());
        assert!(obj.type_name().ends_with("TestObject"));
    }

    #[test]
    fn typed_kobject() {
        struct TestObject {
            field: u32,
        }
        impl KernelObject for TestObject {}

        let obj = TypedKObject::new(TestObject { field: 42 });
        assert!(obj.type_name().ends_with("TestObject"));
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn name() {
        struct TestObject {
            field: u32,
        }
        impl KernelObject for TestObject {}

        let obj = TypedKObject::new(TestObject { field: 42 });
        assert!(obj.type_name().ends_with("TestObject"));
        assert_eq!(obj.field, 42);
    }

    #[test]
    fn as_kobject() {
        struct TestObject {
            field: u32,
        }
        impl KernelObject for TestObject {}

        let obj = TypedKObject::new(TestObject { field: 42 });
        let kobj = obj.as_kobject();
        assert!(kobj.downcast_ref::<TestObject>().is_some());
        assert!(kobj.type_name().ends_with("TestObject"));
        assert_eq!(kobj.downcast_ref::<TestObject>().unwrap().field, 42);
    }
}
