use std::io;
use std::io::Read;
use std::cmp::Ordering;
use rand::Rng;

fn main() {
    let secret_number = rand::thread_rng().gen_range(1, 101);

    //println!("The secret number is {}", secret_number);

    println!("Guess the number!");

    println!("Please input round");
    let mut count = String::new();

    io::stdin().read_line(&mut count)
        .expect("failed to read line");

    let mut count:i32 = count.trim().parse()
        .expect("wrong!");


    while count > 0 {

        println!("Please input your guess round {}", count);
        count = count - 1;
        let mut guess = String::new();

        io::stdin().read_line(&mut guess)
            .expect("failed to read line");

        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        println!("You guessed : {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too Large!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            }
        }
    }
}

