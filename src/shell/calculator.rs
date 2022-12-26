use alloc::string::String;
use alloc::vec::Vec;

pub struct Calculator {
    symbols: Vec<String>
}

impl Calculator {
    pub fn new(symbols: Vec<String>) -> Calculator {
        Calculator {
            symbols
        }
    }

    pub fn calculate(&self) -> i64 {
        let mut result = 0;
        let mut current_operator = String::from("+");
        let mut current_number = String::from("");

        for symbol in &self.symbols {
            if symbol == "+" || symbol == "-" || symbol == "*" || symbol == "/" {
                if current_operator == "+" {
                    result += current_number.parse::<i64>().unwrap();
                } else if current_operator == "-" {
                    result -= current_number.parse::<i64>().unwrap();
                } else if current_operator == "*" {
                    result *= current_number.parse::<i64>().unwrap();
                } else if current_operator == "/" {
                    result /= current_number.parse::<i64>().unwrap();
                }

                current_operator = symbol.clone();
                current_number = String::from("");
            } else {
                current_number.push_str(symbol);
            }
        }

        if current_operator == "+" {
            result += current_number.parse::<i64>().unwrap();
        } else if current_operator == "-" {
            result -= current_number.parse::<i64>().unwrap();
        } else if current_operator == "*" {
            result *= current_number.parse::<i64>().unwrap();
        } else if current_operator == "/" {
            result /= current_number.parse::<i64>().unwrap();
        }

        result
    }
}