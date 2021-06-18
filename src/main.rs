use futures::StreamExt;
use std::time::SystemTime;
use telegram_bot::*;
use reqwest::Url;
use std::io::Write;
use bincode;

pub mod course_tree;

#[tokio::main]
async fn main() {
    let time_begin = SystemTime::now();

    let mut ct = course_tree::CourseTree::new();

    let mut password = String::new();
    print!("Enter the password to be used: ");
    std::io::stdout().flush().unwrap();
    
    std::io::stdin().read_line(&mut password).unwrap();
    let password = password.trim();

    let mut api_key = String::new();
    print!("Enter your api key: ");
    std::io::stdout().flush().unwrap();
    
    std::io::stdin().read_line(&mut api_key).unwrap();
    let api_key = api_key.trim();
    let api = Api::new(api_key);

    let mut stream = api.stream();

    while let Some(update) = stream.next().await {
        let update = update.unwrap();
        if let UpdateKind::CallbackQuery(callback) = update.kind {
            match callback.message {
                Some(message) => {
                    if let MessageOrChannelPost::Message(message) = message {
                        match callback.data {
                            Some(text) => {
                                let change_dir =
                                    text.split("/").map(String::from).collect::<Vec<String>>();
                                let text = text + "/";
                                let (dirs, files) = ct.directory_contents(change_dir).unwrap();
                                let mut keyboard = InlineKeyboardMarkup::new();
                                let row = keyboard.add_empty_row();
                                dirs.iter().for_each(|x| {
                                    row.push(InlineKeyboardButton::callback(
                                        x.get_name(),
                                        text.clone() + x.get_name(),
                                    ))
                                });
                                files.iter().for_each(|x| {
                                    row.push(InlineKeyboardButton::url(x.get_name(), x.get_link()))
                                    //x.get_link()))
                                });
                                let mut message = message.text_reply("Here be the files: ");
                                message.reply_markup(keyboard);
                                api.send(message).await.unwrap();
                            }
                            None => {
                                println!("Error! Bad client!");
                                let message = message.text_reply("Error: Bad Telegram client!");
                                api.send(message).await.unwrap();
                            }
                        };
                    }
                }
                None => {}
            };
        } else if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                match data.as_str() {
                    _ if data.starts_with(format!("/backup {}",password).as_str()) => {
                      let backup = std::fs::File::create(".backup").unwrap();
                      let backup = std::io::BufWriter::new(backup);
                      bincode::serialize_into(backup,&ct).unwrap();
                    }
                    _ if data.starts_with(format!("/edit {} ",password).as_str()) => {
                        let command = data[11..].to_string();
                        if command.starts_with("/add_file ") {
                            let temp = command[10..].to_string();
                            let dir_and_file_and_link = temp
                                .splitn(3, " ")
                                .map(String::from)
                                .collect::<Vec<String>>();
                            match dir_and_file_and_link.len() {
                                3 => {
                                    //directory, file name and link have been provided
                                    if Url::parse(&dir_and_file_and_link.iter().nth(2).unwrap()).is_ok() {
                                      println!("Valid!");
                                    } else {
                                      println!("Invalid!");
                                    }
                                    if Url::parse(&dir_and_file_and_link.iter().nth(2).unwrap()).is_ok() {
                                    if ct
                                        .add_file(
                                            dir_and_file_and_link
                                                .iter()
                                                .nth(0)
                                                .unwrap()
                                                .split("/")
                                                .map(String::from)
                                                .collect::<Vec<String>>(),
                                            &dir_and_file_and_link.iter().nth(1).unwrap(),
                                            &dir_and_file_and_link.iter().nth(2).unwrap(),
                                        )
                                        .is_err()
                                    {
                                        let message = message.text_reply(
                                            "Error! The resource address you entered is invalid!!",
                                        );
                                        api.send(message).await.unwrap();
                                    } else {
                                        api.send(
                                            message
                                                .text_reply("You have succesfully added a file!"),
                                        )
                                        .await
                                        .unwrap();
                                    }
                                    } else {
                                        api.send(
                                            message
                                                .text_reply("Error! The address you entered is invalid!"),
                                        )
                                        .await
                                        .unwrap();
                                    }
                                }
                                2 => {
                                    //directory is expected to be root. File name and link are provided
                                    if Url::parse(&dir_and_file_and_link.iter().nth(1).unwrap()).is_ok() {
                                    if ct
                                        .add_file(
                                            Vec::new(),
                                            &dir_and_file_and_link.iter().nth(0).unwrap(),
                                            &dir_and_file_and_link.iter().nth(1).unwrap(),
                                        )
                                        .is_err()
                                    {
                                        let message = message.text_reply(
                                            "Error: The resource address you entered is invalid!!",
                                        );
                                        api.send(message).await.unwrap();
                                    } else {
                                        api.send(
                                            message
                                                .text_reply("You have succesfully added a file!"),
                                        )
                                        .await
                                        .unwrap();
                                    }}
                                     else {
                                        api.send(
                                            message
                                                .text_reply("Error! The address you entered is invalid!"),
                                        )
                                        .await
                                        .unwrap();
                                    }
                                }
                                _ => {
                                    let message = message.text_reply("Error! /add_file takes either three arguments (resource address at which to create the file, file name and file link) or two (file is created at root)!");
                                    api.send(message).await.unwrap();
                                }
                            }
                        } else if command.starts_with("/add_directory ") {
                            let temp = command[15..].to_string();
                            let dir_and_name = temp
                                .splitn(2, " ")
                                .map(String::from)
                                .collect::<Vec<String>>();
                            match dir_and_name.len() {
                                2 => {
                                    //change directory (path) and name of the new directory are the arguments
                                    if ct
                                        .add_directory(
                                            dir_and_name
                                                .iter()
                                                .nth(0)
                                                .unwrap()
                                                .split("/")
                                                .map(String::from)
                                                .collect::<Vec<String>>(),
                                            &dir_and_name.iter().nth(1).unwrap(),
                                        )
                                        .is_err()
                                    {
                                        let message = message.text_reply(
                                            "Error: The resource address you entered is invalid!!",
                                        );
                                        api.send(message).await.unwrap();
                                    } else {
                                        api.send(
                                            message.text_reply(
                                                "You have succesfully added a directory!",
                                            ),
                                        )
                                        .await
                                        .unwrap();
                                    }
                                }
                                1 => {
                                    //change directory absent. Directory is created at root.
                                    if ct
                                        .add_directory(Vec::new(), &dir_and_name.first().unwrap())
                                        .is_err()
                                    {
                                        let message = message.text_reply(
                                            "Error: The resource address you entered is invalid!!",
                                        );
                                        api.send(message).await.unwrap();
                                    } else {
                                        api.send(
                                            message.text_reply(
                                                "You have succesfully added a directory!",
                                            ),
                                        )
                                        .await
                                        .unwrap();
                                    };
                                }
                                _ => {
                                    let message = message.text_reply("Error: /add_directory requires either one argument (directory name, directory will be created at root), or two(resource address and new directory name)!");
                                    api.send(message).await.unwrap();
                                }
                            }
                        }
                    }
                    "/start" => {
                        println!("{:?}: Got /start", time_begin.elapsed().unwrap());
                        let mut message = message.text_reply(
                                "Hello! This is Gopherine. I'm a Telegram bot and a virtual file system on the cloud.
1. ``` /show {entity}``` tells you what data you have at the \"entity\" resource address.
2. ``` /edit {password} {command}``` allows you to use any of the following commands if your password is correct
      1. ``` /add_file {resource address} {file name} {file link}``` adds a new file to the file system (file is created at root if resource address is omitted.).
      2. ``` /add_directory {resource_address} {directory_name}``` creates a new directory at the specifid address (or root, as above.)
3. To state the obvious, ```/start``` prints this page.
                                ",
                            );
                        api.send(message.parse_mode(ParseMode::Markdown))
                            .await
                            .unwrap();
                    }
                    _ if data.starts_with("/show") => {
                        println!("{:?}: Got print", time_begin.elapsed().unwrap());
                        match data.split(" ").count() {
                            1 => {
                                match ct.directory_contents(Vec::new()) //returns (Vec<Directory>, Vec<File>) 
                            {
                                    Some((dirs, files)) => {
                                        let mut keyboard = InlineKeyboardMarkup::new();
                                        let row = keyboard.add_empty_row();
                                        dirs.iter().for_each(|x| {
                                           row.push(InlineKeyboardButton::callback(
                                               x.get_name(),
                                               data[5..].to_string() + x.get_name(),
                                           ))
                                        });
                                        files.iter().for_each(|x| {
                                            row.push(InlineKeyboardButton::url(
                                                x.get_name(),
                                                x.get_link(),
                                            ))
                                        });
                                        let mut message = message.text_reply("Here be the files: ");
                                        message.reply_markup(keyboard);
                                        api.send(message).await.unwrap();
                                    }
                                    None => {
                                        println!("Error! Bad client!");
                                        let message =
                                            message.text_reply("Error: Bad Telegram client!");
                                        api.send(message).await.unwrap();
                                    }
                            };
                            }
                            2 => {
                                let change_dir = data[5..]
                                    .split("/")
                                    .map(String::from)
                                    .collect::<Vec<String>>();
                                match ct.directory_contents(change_dir) {
                                    Some((dirs, files)) => {
                                        let mut keyboard = InlineKeyboardMarkup::new();
                                        let row = keyboard.add_empty_row();
                                        dirs.iter().for_each(|x| {
                                            row.push(InlineKeyboardButton::callback(
                                                x.get_name(),
                                                data[5..].to_string() + x.get_name(),
                                            ))
                                        });
                                        files.iter().for_each(|x| {
                                            row.push(InlineKeyboardButton::url(
                                                x.get_name(),
                                                x.get_link(),
                                            ))
                                        });
                                        let mut message = message.text_reply("Here be the files: ");
                                        message.reply_markup(keyboard);
                                        api.send(message).await.unwrap();
                                    }
                                    None => {
                                        println!("Error! Bad client!");
                                        let message =
                                            message.text_reply("Error: Bad Telegram client!");
                                        api.send(message).await.unwrap();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }

                    _ => {}
                }
            }
        }
    }
}
