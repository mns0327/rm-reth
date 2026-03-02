use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id(TypeId);

pub trait Resource: Any + 'static {
    fn name() -> &'static str
    where
        Self: Sized;

    fn resource_id() -> Id
    where
        Self: Sized,
    {
        Id(TypeId::of::<Self>())
    }
}

pub struct Context {
    resource_ctxs: HashMap<Id, Box<dyn Resource>>,
}

#[allow(clippy::new_without_default)]
impl Context {
    pub fn new() -> Self {
        Self {
            resource_ctxs: HashMap::new(),
        }
    }

    /// Inserts a resource into the context.
    ///
    /// # Panics
    /// Panics if `T` is already registered.
    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        let entry = self.resource_ctxs.entry(T::resource_id());
        match entry {
            Entry::Occupied(_) => panic!("Resource already exists"),
            Entry::Vacant(v) => v.insert(Box::new(resource)),
        };
    }

    /// Returns a reference to the resource of type `T`.
    ///
    /// # Panics
    /// Panics if `T` has not been added or the stored type does not match `T`.
    pub fn get_resource<T: Resource>(&self) -> &T {
        let as_any: &dyn Any = match self.resource_ctxs.get(&T::resource_id()) {
            Some(resource) => resource.as_ref(),
            None => panic!("Resource not found ({})", T::name()),
        };

        as_any.downcast_ref::<T>().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Foo {
        value: i32,
    }

    impl Resource for Foo {
        fn name() -> &'static str {
            "Foo"
        }
    }

    struct Bar {
        label: String,
    }

    impl Resource for Bar {
        fn name() -> &'static str {
            "Bar"
        }
    }

    #[test]
    fn id_same_type_is_equal() {
        assert_eq!(Foo::resource_id(), Foo::resource_id());
    }

    #[test]
    fn id_different_types_are_not_equal() {
        assert_ne!(Foo::resource_id(), Bar::resource_id());
    }

    #[test]
    fn id_is_copy() {
        let id = Foo::resource_id();
        let id2 = id;
        assert_eq!(id, id2);
    }

    #[test]
    fn resource_name() {
        assert_eq!(Foo::name(), "Foo");
        assert_eq!(Bar::name(), "Bar");
    }

    #[test]
    fn resource_id_matches_typeid() {
        assert_eq!(Foo::resource_id(), Id(TypeId::of::<Foo>()));
    }

    #[test]
    fn context_new_is_empty() {
        let ctx = Context::new();
        assert!(ctx.resource_ctxs.is_empty());
    }

    #[test]
    fn add_resource_single() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 1 });
        assert_eq!(ctx.resource_ctxs.len(), 1);
    }

    #[test]
    fn add_resource_multiple_different_types() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 1 });
        ctx.add_resource(Bar { label: "hi".into() });
        assert_eq!(ctx.resource_ctxs.len(), 2);
    }

    #[test]
    #[should_panic(expected = "Resource already exists")]
    fn add_resource_duplicate_panics() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 1 });
        ctx.add_resource(Foo { value: 2 });
    }

    #[test]
    fn get_resource_returns_correct_value() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 42 });
        assert_eq!(ctx.get_resource::<Foo>().value, 42);
    }

    #[test]
    fn get_resource_multiple_types_independent() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 10 });
        ctx.add_resource(Bar {
            label: "hello".into(),
        });
        assert_eq!(ctx.get_resource::<Foo>().value, 10);
        assert_eq!(ctx.get_resource::<Bar>().label, "hello");
    }

    #[test]
    #[should_panic(expected = "Resource not found (Foo)")]
    fn get_resource_not_found_panics() {
        let ctx = Context::new();
        ctx.get_resource::<Foo>();
    }

    #[test]
    fn get_resource_does_not_affect_other_resources() {
        let mut ctx = Context::new();
        ctx.add_resource(Foo { value: 7 });
        ctx.add_resource(Bar {
            label: "world".into(),
        });
        let _ = ctx.get_resource::<Foo>();
        assert_eq!(ctx.get_resource::<Bar>().label, "world");
    }
}
