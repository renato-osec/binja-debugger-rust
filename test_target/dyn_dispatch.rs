// Simple test binary with dynamic dispatch for vtable analysis

trait Animal {
    fn speak(&self);
    fn walk(&self);
}

struct Dog {
    name: String,
}

struct Cat {
    name: String,
}

impl Animal for Dog {
    fn speak(&self) {
        println!("Dog {} says: Woof!", self.name);
    }
    fn walk(&self) {
        println!("Dog {} is walking", self.name);
    }
}

impl Animal for Cat {
    fn speak(&self) {
        println!("Cat {} says: Meow!", self.name);
    }
    fn walk(&self) {
        println!("Cat {} is walking", self.name);
    }
}

fn make_animal_speak(animal: &dyn Animal) {
    animal.speak();
    animal.walk();
}

fn main() {
    let dog = Dog { name: String::from("Rex") };
    let cat = Cat { name: String::from("Whiskers") };

    make_animal_speak(&dog);
    make_animal_speak(&cat);

    // Store in Box for additional vtable usage
    let animals: Vec<Box<dyn Animal>> = vec![
        Box::new(Dog { name: String::from("Buddy") }),
        Box::new(Cat { name: String::from("Mittens") }),
    ];

    for animal in &animals {
        animal.speak();
    }
}
