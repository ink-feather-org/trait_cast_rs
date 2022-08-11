use std::any::{type_name_of_val, Any, TypeId};

struct TraitcastTarget {
    target_type_id: TypeId,
    to_dyn_func: fn(&dyn Traitcastable) -> *const (),
}

trait Traitcastable {
    fn traitcastable_from(&self) -> Vec<TraitcastTarget>;
}

// fn trait_cast<'a, Target: Sized + 'static>(
//   source: &'a dyn Any,
//   trait_cast_target: &[TraitcastTarget],
// ) -> Option<&'a Target> {
//   source.downcast_ref::<Target>()
// }

fn trait_cast<'a, Target: ?Sized + 'static>(
    source: &'a dyn Any,
    trait_cast_target: &[TraitcastTarget],
) -> Option<&'a Target> {
    unsafe {
        let target = trait_cast_target
            .iter()
            .find(|possible| possible.target_type_id == std::any::TypeId::of::<Target>());
        if let Some(target) = target {
            let fn_ptr: fn(&dyn Any) -> Option<&Target> = std::mem::transmute(target.to_dyn_func);
            println!(
                "fn_ptr {} {:p}",
                type_name_of_val(&fn_ptr),
                fn_ptr as *const ()
            );
            println!("source {:p}", source);
            fn_ptr(source)
        } else {
            None
        }
    }
}

struct Foo {
    member: u8,
}
impl Foo {
    /// Pass this function pointer to register_downcast
    pub fn to_dyn_baup(input: &dyn Any) -> Option<&dyn Baup> {
        input.downcast_ref::<Self>().map(|selv| selv as &dyn Baup)
    }
}
impl Baup for Foo {
    fn baup(&self) {
        println!("BAUP {}!", self.member);
    }
}
impl Traitcastable for Foo {
    fn traitcastable_from(&self) -> Vec<TraitcastTarget> {
        unsafe {
            //   let to_dyn_func: fn(&dyn Traitcastable) -> *const () =
            //     std::mem::transmute(Self::to_dyn_baup as fn(_) -> _);
            println!("orig_to_dyn_func {:p}", Self::to_dyn_baup as *const ());
            vec![TraitcastTarget {
                target_type_id: std::any::TypeId::of::<dyn Baup>(),
                to_dyn_func: std::mem::transmute(Self::to_dyn_baup as fn(_) -> _),
            }]
        }
    }
}

trait Baup {
    fn baup(&self);
}

fn main() {
    let stack_foo = Foo { member: 2 };
    println!("stack {:p}", &stack_foo);

    let foo = Box::new(Foo { member: 8 });
    println!("foo {:p}", foo);
    let real_baup: &dyn Baup = foo.as_ref();
    real_baup.baup();
    println!("real_baup {:p}", real_baup);

    let targets = foo.traitcastable_from();
    let any_foo: Box<dyn Any> = foo;
    println!("any_foo {:p}", any_foo);
    let baup = trait_cast::<dyn Baup>(any_foo.as_ref(), &targets).unwrap();
    baup.baup();

    let baup = trait_cast::<Foo>(any_foo.as_ref(), &targets);
    baup.unwrap();

    // BAD BOX AS ANY
    // See very save even when source is wrong!
    assert!(trait_cast::<dyn Baup>(&any_foo, &targets).is_none());
}
