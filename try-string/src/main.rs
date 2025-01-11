

fn main() {
    let mut string_1 = String::from("Hello");
    let mut string_2 = String::from("world");

    println!("{}, {}", string_1, string_2);

    let mut string_3 = string_1.clone() + ", " + &string_2;
    string_2.clear();
    println!("{}, {}", string_1, string_2);
    println!("{string_3}");

    string_2 = String::from("world");
    let mut string_3 = [string_1, string_2].join(", ");
    println!("{}", string_3);

    string_3 = String::from("    ") + &string_3 + "!   ";
    string_3 = string_3.trim().to_owned();
    println!("{}", string_3);
    
    println!("{}", &string_3[2..]);

    let mut string_4 = String::from("Привет, мир");
    let mut char_iter = string_4.char_indices();
    let (start, _) = char_iter.nth(2).unwrap();
    println!("{}", &string_4[start..]);

}
