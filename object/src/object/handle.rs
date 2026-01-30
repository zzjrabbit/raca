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
    use crate::object::KernelObject;

    use super::*;

    struct DummyObject;
    impl KernelObject for DummyObject {}

    #[test]
    fn new_handle() {
        let obj = KObject::new(DummyObject);
        let _handle = Handle::new(obj, Rights::BASIC);
    }
}
