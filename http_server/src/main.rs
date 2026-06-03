struct User {
    active:bool,
    age:u64,
}

fn main() {
    let n:u64 = 5;
    let user = User { active: false, age: n};

    println!("User age = {}", user.age);
    println!("Hello, world!");
}
