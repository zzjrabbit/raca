use crate::object::{KObject, Rights};

#[derive(Clone)]
pub struct Handle {
    pub object: KObject,
    pub rights: Rights,
}

impl Handle {
    pub fn new(obj: KObject, rights: Rights) -> Self {
        Handle {
            object: obj,
            rights,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        impl_kobj, new_kobj,
        object::{KObjectBase, KernelObject},
    };

    use super::*;

    struct DummyObject {
        base: KObjectBase,
    }
    impl_kobj!(DummyObject);
    impl DummyObject {
        fn new() -> alloc::sync::Arc<Self> {
            new_kobj!({})
        }
    }

    #[test]
    fn new_handle() {
        let obj = DummyObject::new();
        let _handle = Handle::new(obj, Rights::BASIC);
    }
}
