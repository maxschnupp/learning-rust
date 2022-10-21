use std::ops::Range;

fn print_fizz_buzz(number: i32) {
    if number % 3 == 0 && number % 5 == 0 {
        println!("FizzBuzz");
    } else if number % 3 == 0 {
        println!("Fizz");
    } else if number % 5 == 0{
        println!("Buzz");
    } else {
        println!("{}", number)
    }
}

fn main() {
    const NUMBERS: Range<i32> = 1..100;
    for number in NUMBERS {
        print_fizz_buzz(number)
    }
}

