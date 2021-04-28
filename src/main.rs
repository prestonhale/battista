use std::io::{stdin,stdout,Write};
use std::collections::HashMap;
use std::vec::Vec;
use uuid::Uuid;

use rand;
use rand::seq::SliceRandom;

use rusoto_dynamodb::{DynamoDbClient, DynamoDb, PutItemInput, GetItemInput, ScanInput, QueryInput, UpdateItemInput, AttributeValue};
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

    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let message_id = Uuid::new_v4();

    // Update the player's list of available messages
    let item: HashMap<String, AttributeValue> = [
        (
            String::from("user_id"),
            AttributeValue{
                s: Some(String::from(user_id)),
                ..Default::default()
            }
        ), (
            String::from("message_id"),
            AttributeValue{
                s: Some(format!("{}", message_id)),
                ..Default::default()
            }
        ), (
            String::from("message"),
            AttributeValue{
                s: Some(input),
                ..Default::default()
            }
        ), (
            String::from("unread"),
            AttributeValue{
                s: Some(String::from("t")),
                ..Default::default()
            }
        )
    ].iter().cloned().collect();
    let put_input = PutItemInput{
        item: item,
        table_name: String::from("messages"),
        ..Default::default()
    };
    match dynamo_client.put_item(put_input).await {
        Ok(_) => println!("Message sent!"),
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }
}

#[tokio::main]
async fn get_message(user_id: &String){
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);

    let mut selected_message_id = "".to_string();
    let mut selected_user_id = "".to_string();

    // Find user with a relationship
    let mut relationships = vec![];
    let key: HashMap<String, AttributeValue> = [
        (
            "user_id".to_string(),
            AttributeValue{
                s: Some(user_id.to_string()) ,
                ..Default::default()
            }
        )
    ].iter().cloned().collect();
    let get_item_input = GetItemInput{
        key: key,
        table_name: String::from("users"),
        ..Default::default()
    };
    match dynamo_client.get_item(get_item_input).await {
        Ok(output) => match output.item{
            Some(item) => {
                let item_relationships = item["relationships"].l.as_ref().unwrap();
                for r in item_relationships.iter(){
                    let relationship = r.m.as_ref().unwrap();
                    relationships.push(relationship["user_id"].s.as_ref().unwrap().clone());
                }
            },
            None => println!("Something went wrong"),
        },
        Err(error) => {
            println!("Error: {:?}", error);
        }
    };

    // Get a random related user
    let chosen_user  = relationships.choose(&mut rand::thread_rng()).unwrap();

    // Get all unread messages from selected user
    let expression_attribute_values: HashMap<String, AttributeValue> = [
        (
            ":user_id".to_string(),
            AttributeValue{
                s: Some(chosen_user.to_string()),
                ..Default::default()
            }
        )
    ].iter().cloned().collect();
    let query_input = QueryInput{
        expression_attribute_values: Some(expression_attribute_values),
        key_condition_expression: Some("user_id = :user_id".to_string()),
        index_name: Some(String::from("GSI1")),
        table_name: String::from("messages"),
        ..Default::default()
    };
    match dynamo_client.query(query_input).await {
        Ok(output) => match output.items{
            Some(items) => {
                match items.choose(&mut rand::thread_rng()) {
                    Some(selected) => {
                        println!("{}", selected["message"].s.as_ref().unwrap());
                        selected_user_id = selected["user_id"].s.as_ref().unwrap().clone();
                        selected_message_id = selected["message_id"].s.as_ref().unwrap().clone();
                    }
                    None => println!("No messages available"),
                }
            }
            None => println!("Something went wrong"),
        },
        Err(error) => {
            println!("Error: {:?}", error);
        }
    };

    if selected_message_id == "" || selected_user_id == "" {
        println!("No valid message found!");
        return;
    }

    // Mark a message as read
    let key: HashMap<String, AttributeValue> = [(
        String::from("user_id"),
        AttributeValue{
            s: Some(selected_user_id.to_string()),
            ..Default::default()
        }), (
        String::from("message_id"),
        AttributeValue{
            s: Some(selected_message_id.to_string()),
            ..Default::default()
        }
    )].iter().cloned().collect();
    let update_input = UpdateItemInput{
        key: key,
        update_expression: Some("REMOVE unread".to_string()),
        table_name: String::from("messages"),
        ..Default::default()
    };
    match dynamo_client.update_item(update_input).await {
        Ok(output) => println!("Read"),
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
