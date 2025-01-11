

use std::vec::{self, Vec};
use std::string::String;

fn main() {

    let mut vector_1: Vec<f64> = Vec::new();

    for i in 0..10 {
        vector_1.push(f64::from(i));
    }

    for value in &vector_1 {
        println!("{}", *value);
    }

    add_new_elements(&mut vector_1);
    
    let string_vector: Vec<String> = vector_1.iter().map(|v| v.to_string()).collect();
    let joined_string: String = string_vector.join(", ");
    println!("{}", joined_string);

    println!("{}", vector_1[0]);
    println!("{}", vector_1.get(0).unwrap());
    println!("{:?}", &vector_1[10..]);

    // linear search
    let position_1 = vector_1.iter().position(|&x| x == 5.0);
    match position_1 {
        Some(i) => println!("{}", i),
        None => {
            println!("Not found");
            panic!("Failed to find the element");
        }
    }

    let position_2 = vector_1.binary_search_by(|x| x.total_cmp(&5.0));
    match position_2 {
        Ok(i) => println!("{}", i),
        Err(i) => {
            println!("Not found");
            panic!("Failed to find the element");
        }
    }

    vector_1.reverse();
    println!("{:?}", &vector_1[..10]);

    vector_1.sort_by(|a, b| a.total_cmp(b));
    println!("{:?}", &vector_1[..10]);

}

fn add_new_elements(vector:&mut Vec<f64>) -> () {
    for i in 10..20 {
        vector.push(f64::from(i));
    }

    vector.push(5.0);
}
