use std::io::{stdin,stdout,Write};
use std::collections::HashMap;

use rusoto_dynamodb::{DynamoDbClient, DynamoDb, PutItemInput, GetItemInput, AttributeValue};
use rusoto_core::{Region};

fn main() {
    let mut finished = false;
    while !finished {
        println!("What would you like to do?");
        println!("1. Get my messages!");
        println!("2. Write a message!");
        println!("3. Quit!");

        let input = get_input(String::from("> "));
        match &input[..] {
            "1" => get_message(),
            "2" => write_message(),
            "3" => finished = true,
            _ => println!("Invalid selection")
        }
    }
}

#[tokio::main]
async fn write_message(){
    println!("Write your message: ");
    let input = get_input(String::from("> "));

    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let mut item = HashMap::new();
    item.insert(
        String::from("user"),
        AttributeValue{
            b: None, 
            bs: None, 
            bool: None, 
            l: None, 
            m: None,
            n: None,
            ns: None,
            null: None,
            s: Some(String::from("preston")),
            ss: None,
        },
    );
    item.insert(
        String::from("message"),
        AttributeValue{
            b: None, 
            bs: None, 
            bool: None, 
            l: None, 
            m: None,
            n: None,
            ns: None,
            null: None,
            s: Some(String::from(input)),
            ss: None,
        },
    );
    let dynamo_put_input = PutItemInput{
        condition_expression: None,
        expected: None,
        expression_attribute_names: None,
        conditional_operator: None,
        expression_attribute_values: None,
        return_consumed_capacity: None,
        return_item_collection_metrics: None,
        return_values: None,
        item: item,
        table_name: String::from("messages"),
    };
    match dynamo_client.put_item(dynamo_put_input).await {
        Ok(_) => println!("Message sent!"),
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

#[tokio::main]
async fn get_message(){
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let mut key = HashMap::new();
    key.insert(
        String::from("user"),
        AttributeValue{
            b: None, 
            bs: None, 
            bool: None, 
            l: None, 
            m: None,
            n: None,
            ns: None,
            null: None,
            s: Some(String::from("preston")),
            ss: None,
        },
    );
    let dynamo_get_input = GetItemInput{
        attributes_to_get: None,
        consistent_read: None,
        expression_attribute_names: None,
        key: key,
        projection_expression: None,
        return_consumed_capacity: None,
        table_name: String::from("messages"),
    };
    match dynamo_client.get_item(dynamo_get_input).await {
        Ok(output) => match output.item{
            Some(item) => {
                println!("You've got mail: ");
                let message = &item["message"]; // null check?
                println!("{}", message.s.as_ref().expect("Found dynamo message but was not string type!"));
            },
            None => println!("No mail yet!"),
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

fn get_input(prompt: String) -> String {
    print!("{}", prompt);
    stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read user inputted line");
    input.trim().to_string()
}
