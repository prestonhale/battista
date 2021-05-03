use std::collections::HashMap;
use uuid::Uuid;

use rand;
use rand::seq::SliceRandom;

use rusoto_core::Region;
use rusoto_dynamodb::{
    AttributeValue, DynamoDb, DynamoDbClient, GetItemInput, PutItemInput, QueryInput, ScanInput,
    UpdateItemInput,
};

pub async fn get_or_create_user(user_name: &String) -> String {
    let mut user_id;
    user_id = get_user(&user_name).await;
    if user_id == "" {
        user_id = create_user(&user_name).await;
    }
    return user_id;
}

async fn get_user(user_name: &String) -> String {
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let expression_attribute_values: HashMap<String, AttributeValue> = [(
        ":user_name".to_string(),
        AttributeValue {
            s: Some(user_name.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();
    let query_input = QueryInput {
        expression_attribute_values: Some(expression_attribute_values),
        key_condition_expression: Some("user_name = :user_name".to_string()),
        index_name: Some(String::from("user_name_index")),
        table_name: String::from("users"),
        ..Default::default()
    };
    match dynamo_client.query(query_input).await {
        Ok(output) => match output.items {
            Some(items) => {
                if items.len() > 1 {
                    panic!("More than one user with same name!");
                } else if items.len() == 1 {
                    return items[0]["user_id"].s.as_ref().unwrap().to_string();
                } else {
                    return "".to_string();
                }
            }
            None => return "".to_string(),
        },
        Err(error) => {
            println!("Error: {:?}", error);
            return "".to_string();
        }
    };
}

async fn create_user(user_name: &String) -> String {
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let random_user_id = Uuid::new_v4();
    let item: HashMap<String, AttributeValue> = [
        (
            String::from("user_id"),
            AttributeValue {
                s: Some(random_user_id.to_string()),
                ..Default::default()
            },
        ),
        (
            String::from("user_name"),
            AttributeValue {
                s: Some(user_name.to_string()),
                ..Default::default()
            },
        ),
        (
            String::from("relationships"),
            AttributeValue {
                l: Some(vec![]),
                ..Default::default()
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();
    let put_input = PutItemInput {
        item: item,
        table_name: String::from("users"),
        ..Default::default()
    };
    match dynamo_client.put_item(put_input).await {
        Ok(_) => {
            println!("User created.");
            return random_user_id.to_string();
        }
        Err(error) => {
            println!("Error: {:?}", error);
            return "".to_string();
        }
    }
}

pub async fn write_message(user_id: &String, message: &String) {
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let message_id = Uuid::new_v4();

    // Update the player's list of available messages
    let item: HashMap<String, AttributeValue> = [
        (
            String::from("user_id"),
            AttributeValue {
                s: Some(String::from(user_id)),
                ..Default::default()
            },
        ),
        (
            String::from("message_id"),
            AttributeValue {
                s: Some(format!("{}", message_id)),
                ..Default::default()
            },
        ),
        (
            String::from("message"),
            AttributeValue {
                s: Some(message.to_string()),
                ..Default::default()
            },
        ),
        (
            String::from("unread"),
            AttributeValue {
                s: Some(String::from("t")),
                ..Default::default()
            },
        ),
    ]
    .iter()
    .cloned()
    .collect();
    let put_input = PutItemInput {
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

pub async fn get_message(user_id: &String) -> String {
    let mut message = get_friend_message(user_id).await;
    println!("Found a message: {}", message);
    if message == "" {
        message = create_new_relationship(user_id).await;
    }
    return message;
}

async fn get_friend_message(user_id: &String) -> String {
    let mut selected_message_id = "".to_string();
    let mut selected_user_id = "".to_string();
    let mut selected_message_content = "".to_string();

    let dynamo_client = DynamoDbClient::new(Region::UsWest2);

    // Get all friends
    let mut relationships = vec![];
    let key: HashMap<String, AttributeValue> = [(
        "user_id".to_string(),
        AttributeValue {
            s: Some(user_id.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();
    let get_item_input = GetItemInput {
        key: key,
        table_name: String::from("users"),
        ..Default::default()
    };
    match dynamo_client.get_item(get_item_input).await {
        Ok(output) => match output.item {
            Some(item) => {
                let item_relationships = item["relationships"].l.as_ref().unwrap();
                for r in item_relationships.iter() {
                    let relationship = r.m.as_ref().unwrap();
                    relationships.push(relationship["user_id"].s.as_ref().unwrap().clone());
                }
            }
            None => println!("Something went wrong"),
        },
        Err(error) => {
            println!("Error: {:?}", error);
        }
    };

    if relationships.len() == 0 {
        return selected_message_content;
    }

    // Get a random message
    relationships.shuffle(&mut rand::thread_rng());
    for chosen_user in relationships {
        // Get all unread messages from selected user
        let expression_attribute_values: HashMap<String, AttributeValue> = [(
            ":user_id".to_string(),
            AttributeValue {
                s: Some(chosen_user.to_string()),
                ..Default::default()
            },
        )]
        .iter()
        .cloned()
        .collect();
        let query_input = QueryInput {
            expression_attribute_values: Some(expression_attribute_values),
            key_condition_expression: Some("user_id = :user_id".to_string()),
            index_name: Some(String::from("GSI1")),
            table_name: String::from("messages"),
            ..Default::default()
        };
        match dynamo_client.query(query_input).await {
            Ok(output) => match output.items {
                Some(items) => match items.choose(&mut rand::thread_rng()) {
                    Some(selected) => {
                        selected_message_content = selected["message"].s.as_ref().unwrap().to_string();
                        selected_user_id = selected["user_id"].s.as_ref().unwrap().clone();
                        selected_message_id = selected["message_id"].s.as_ref().unwrap().clone();
                    }
                    None => println!("No messages available"),
                },
                None => println!("Something went wrong"),
            },
            Err(error) => {
                println!("Error: {:?}", error);
            }
        };

        if selected_message_id == "" || selected_user_id == "" {
            println!("No valid message found!");
            return selected_message_content;
        }

        // Mark a message as read
        let key: HashMap<String, AttributeValue> = [
            (
                String::from("user_id"),
                AttributeValue {
                    s: Some(selected_user_id.to_string()),
                    ..Default::default()
                },
            ),
            (
                String::from("message_id"),
                AttributeValue {
                    s: Some(selected_message_id.to_string()),
                    ..Default::default()
                },
            ),
        ]
        .iter()
        .cloned()
        .collect();
        let update_input = UpdateItemInput {
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
    return selected_message_content;
}

async fn create_new_relationship(user_id: &String) -> String {
    let dynamo_client = DynamoDbClient::new(Region::UsWest2);
    let mut selected_message_id = "".to_string();
    let mut selected_user_id = "".to_string();
    let mut selected_message_content = "".to_string();

    // Find a random user
    // TODO: Select random user on factors such as activity
    // TODO: This can loop forever if there are no unread messages at all
    while selected_message_id == "" {
        let random_user_id = Uuid::new_v4();
        let start_key: HashMap<String, AttributeValue> = [(
            "user_id".to_string(),
            AttributeValue {
                s: Some(random_user_id.to_string()),
                ..Default::default()
            },
        )]
        .iter()
        .cloned()
        .collect();
        let scan_input = ScanInput {
            exclusive_start_key: Some(start_key),
            table_name: "users".to_string(),
            limit: Some(1),
            ..Default::default()
        };
        match dynamo_client.scan(scan_input).await {
            Ok(output) => match output.items {
                Some(items) => {
                    selected_user_id = String::from(items[0]["user_id"].s.as_ref().unwrap())
                }
                None => println!("NO users exist? Really?"),
            },
            Err(error) => {
                println!("Error: {:?}", error);
                return selected_message_content;
            }
        };
        println!("{} random user", selected_user_id);

        // Check if that player has available messages
        let expression_attribute_values: HashMap<String, AttributeValue> = [(
            ":user_id".to_string(),
            AttributeValue {
                s: Some(selected_user_id.to_string()),
                ..Default::default()
            },
        )]
        .iter()
        .cloned()
        .collect();
        let query_input = QueryInput {
            expression_attribute_values: Some(expression_attribute_values),
            key_condition_expression: Some("user_id = :user_id".to_string()),
            index_name: Some(String::from("GSI1")),
            table_name: String::from("messages"),
            ..Default::default()
        };
        match dynamo_client.query(query_input).await {
            Ok(output) => match output.items {
                Some(items) => match items.choose(&mut rand::thread_rng()) {
                    Some(selected) => {
                        selected_message_content = selected["message"].s.as_ref().unwrap().to_string();
                        selected_user_id = selected["user_id"].s.as_ref().unwrap().clone();
                        selected_message_id = selected["message_id"].s.as_ref().unwrap().clone();
                    }
                    None => println!("No messages available"),
                },
                None => println!("Something went wrong"),
            },
            Err(error) => {
                println!("Error: {:?}", error);
            }
        };
        // If no messages for this random user, check another random
    }

    // If yes, create new relationship for both players
    let key: HashMap<String, AttributeValue> = [(
        String::from("user_id"),
        AttributeValue {
            s: Some(user_id.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();
    let relationship_map: HashMap<String, AttributeValue> = [(
        "user_id".to_string(),
        AttributeValue {
            s: Some(selected_user_id.to_string()),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();
    let expression_attribute_values: HashMap<String, AttributeValue> = [(
        String::from(":relationship"),
        AttributeValue {
            l: Some(vec![AttributeValue {
                m: Some(relationship_map),
                ..Default::default()
            }]),
            ..Default::default()
        },
    )]
    .iter()
    .cloned()
    .collect();
    let update_input = UpdateItemInput {
        key: key,
        expression_attribute_values: Some(expression_attribute_values),
        update_expression: Some(
            "SET relationships = list_append(relationships, :relationship)".to_string(),
        ),
        table_name: String::from("users"),
        ..Default::default()
    };
    match dynamo_client.update_item(update_input).await {
        Ok(output) => println!("Relationship1 created"),
        Err(error) => {
            println!("Error: {:?}", error);
        }
    }

    return selected_message_content;
}
