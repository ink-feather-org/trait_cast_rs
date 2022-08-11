extern crate trait_cast_rs;

// #[make_trait_castable(Dog, Cat)]
struct HybridPet {
    name: String,
}
impl HybridPet {
    /// Pass this function pointer to register_downcast
    pub fn to_dyn_baup(input: &dyn Any) -> Option<&dyn Dog> {
        input.downcast_ref::<Self>().map(|selv| selv as &dyn Dog)
    }
}
impl Dog for HybridPet {
    fn bark(&self) {
        println!("Woof {}!", self.name);
    }
}
impl Cat for HybridPet {
    fn meow(&self) {
        println!("Meow {}!", self.name);
    }
}
impl Traitcastable for HybridPet {
    fn traitcastable_from(&self) -> Vec<TraitcastTarget> {
        unsafe {
            vec![TraitcastTarget {
                target_type_id: std::any::TypeId::of::<dyn Dog>(),
                to_dyn_func: std::mem::transmute(Self::to_dyn_baup as fn(_) -> _),
            }]
        }
    }
}

trait Dog {
    fn bark(&self);
}
trait Cat {
    fn meow(&self);
}

fn main() {}
