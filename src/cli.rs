use std::env;
use crate::idx;


fn cli() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        panic!("No arguments provided, Use 'set' or 'get'");
    }


    if !["set", "get"].contains(&args[1].as_str()) {
        panic!("Invalid arguments! Use 'set' or 'get'");
    }

    if &args[1] == "set" && args.len() != 4 {
        panic!("Invalid arguments! Use set 'key' 'value'");
    } else if &args[1] == "get" && args.len() != 3 {
        panic!("Invalid arguments! Use get 'key'");
    }

    let idx = idx::IDX::new(None);


    if args[2].len() as u8 > u8::MAX || !args[2].chars().all(|x| x.is_alphabetic()) {
        panic!("Key must be alphabetic and less then 11 chars");
    }

    if &args[1] == "set" {
        match idx.set_key(args[2].as_str(), args[3].as_str()) {
            Ok(key) => println!("Key set {:?}", key),
            Err(e) => panic!("{}", e),
        }
    } else if &args[1] == "get" {
        match idx.get_value(args[2].as_str()) {
            Ok(value) => println!("Value get {:?}", value),
            Err(e) => panic!("{}", e),
        };

    }

}
