use std::io::{stdin,stdout,Write};
use std::collections::HashMap;
use uuid::Uuid;

use rand;
use rand::seq::SliceRandom;

use rusoto_dynamodb::{DynamoDbClient, DynamoDb, PutItemInput, GetItemInput, QueryInput, UpdateItemInput, AttributeValue};
use rusoto_core::{Region};

fn main() {
    println!("Enter user name!");
    let input = get_input(String::from("> "));
    let user_id = input;

    let mut finished = false;
    while !finished {
        println!("What would you like to do?");
        println!("1. Get my messages!");
        println!("2. Write a message!");
        println!("3. Quit!");

        let input = get_input(String::from("> "));
        match &input[..] {
            "1" => get_message(&user_id),
            "2" => write_message(&user_id),
            "3" => finished = true,
            _ => println!("Invalid selection")
        }
    }
}

#[tokio::main]
async fn write_message(user_id: &String){
    println!("Write your message: ");
    let input = get_input(String::from("> "));

    // Insert the message
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let message_id = Uuid::new_v4();
    let mut item = HashMap::new();
    item.insert(
        String::from("message_id"),
        AttributeValue{
            s: Some(message_id.to_string()),
            ..Default::default()
        },
    );
    item.insert(
        String::from("message"),
        AttributeValue{
            s: Some(String::from(input)),
            ..Default::default()
        },
    );
    let dynamo_put_input = PutItemInput{
        item: item,
        table_name: String::from("messages"),
        ..Default::default()
    };
    match dynamo_client.put_item(dynamo_put_input).await {
        Ok(_) => println!("Message saved!"),
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }

    // Update the player's list of available messages
    let mut key = HashMap::new();
    key.insert(
        String::from("user_id"),
        AttributeValue{
            s: Some(user_id.clone()),
            ..Default::default()
        },
    );
    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        String::from(":new_message_id"),
        AttributeValue{
            ss: Some(vec!(message_id.to_string())),
            ..Default::default()
        },
    );
    let update_input = UpdateItemInput{
        key: key,
        update_expression: Some(String::from("ADD available_messages :new_message_id")),
        expression_attribute_values: Some(expression_attribute_values),
        table_name: String::from("available_messages"),
        ..Default::default()
    };
    match dynamo_client.update_item(update_input).await {
        Ok(_) => println!("Message sent!"),
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

#[tokio::main]
async fn get_message(user_id: &String){
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);

    // TODO: Get users the player has a relationship with and/or random players
    // Select a message from those users to deliver
    let mut selected_message_id: String = String::from("");
    let mut expression_attribute_values = HashMap::new();
    expression_attribute_values.insert(
        String::from(":user_id"),
        AttributeValue{
            s: Some(String::from("bob")), // TODO: Hardcoded
            ..Default::default()
        }
    );
    let query_input = QueryInput{
        expression_attribute_values: Some(expression_attribute_values),
        key_condition_expression: Some(format!("user_id = :user_id")),
        table_name: String::from("available_messages"),
        ..Default::default()
    };
    match dynamo_client.query(query_input).await {
        Ok(output) => match output.items{
            Some(items) => {
                for x in items.iter(){
                    let messages = x["available_messages"].ss.as_ref().expect("Available_messages is not a set?");
                    let selected = messages.choose(&mut rand::thread_rng()).expect("Random msg selection failed.");
                    selected_message_id = String::from(selected);
                }
            },
            None => println!("No mail yet!"),
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    };

    if selected_message_id == ""{
        return
    }

    // Deliver the message
    let mut key = HashMap::new();
    key.insert(
        String::from("message_id"),
        AttributeValue{
            s: Some(selected_message_id.to_string()),
            ..Default::default()
        }
    );
    let get_input = GetItemInput{
        key: key,
        table_name: String::from("messages"),
        ..Default::default()
    };
    match dynamo_client.get_item(get_input).await {
        Ok(output) => match output.item{
            Some(item) => {
                let message = item["message"].s.as_ref().expect("Message is not string type?");
                println!("{}", message)
            },
            None => println!("Oh no! Failed to find message! This shouldn't happen."),
        }
        Err(error) => {
            println!("Error: {:?}", error);
        }
    };

    // TODO: Mark the delivered message as read
    // by REMOVEing it from the "available_messages" table
}

fn get_input(prompt: String) -> String {
    print!("{}", prompt);
    stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read user inputted line");
    input.trim().to_string()
}
