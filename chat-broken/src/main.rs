use std::io::{self, Stdin, Stdout, Write};

mod error;
use error::ChatBrokenError;
use lcu::Lcu;

use serde_json::{json, Value};
use unicode_normalization::UnicodeNormalization;

#[derive(Debug)]
struct Friend {
    name: String,
    summoner_id: u64,
    availability: String,
}

#[tokio::main]
async fn main() -> Result<(), ChatBrokenError> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let lcu = match Lcu::new() {
        Ok(lcu) => lcu,
        Err(_) => {
            println!("League process is not running");
            println!("Press enter to continue...");
            stdin.read_line(&mut String::new())?;
            panic!()
        }
    };
    let friends_json = lcu
        .get("/lol-chat/v1/friends")
        .await?
        .json::<Value>()
        .await?;
    let friends_list = friends_json
        .as_array()
        .ok_or(ChatBrokenError::InvalidDataError(
            "friends list is not array",
        ))?
        .iter()
        .map(|friend_json| {
            let friend = friend_json
                .as_object()
                .ok_or(ChatBrokenError::InvalidDataError("friend is not object"))?;
            Ok::<Friend, ChatBrokenError>(Friend {
                name: friend
                    .get("name")
                    .ok_or(ChatBrokenError::InvalidDataError("no name in friend"))?
                    .as_str()
                    .ok_or(ChatBrokenError::InvalidDataError("name is not string"))?
                    .into(),
                summoner_id: friend
                    .get("summonerId")
                    .ok_or(ChatBrokenError::InvalidDataError(
                        "no summoner id in friend",
                    ))?
                    .as_u64()
                    .ok_or(ChatBrokenError::InvalidDataError(
                        "summoner id is not number",
                    ))?,
                availability: friend
                    .get("availability")
                    .ok_or(ChatBrokenError::InvalidDataError(
                        "no availability in friend",
                    ))?
                    .as_str()
                    .ok_or(ChatBrokenError::InvalidDataError(
                        "availability is not string",
                    ))?
                    .into(),
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    loop {
        if let Err(e) = main_loop(&friends_list, &lcu, &stdin, &mut stdout).await {
            println!("Error: {}", e);
        }
    }
}

async fn main_loop(
    friends_list: &Vec<Friend>,
    lcu: &Lcu,
    stdin: &Stdin,
    stdout: &mut Stdout,
) -> Result<(), ChatBrokenError> {
    println!("{}", "-".repeat(25));
    println!("Search for the name of player to invite and press enter, or press enter without typing anything to accept pending invitations:");
    let mut buf = String::new();
    stdin.read_line(&mut buf)?;
    buf = buf.trim().into();
    if !buf.is_empty() {
        println!();
        let mut possibilities = Vec::with_capacity(friends_list.len());
        for friend in friends_list {
            if friend
                .name
                .nfkd()
                .collect::<String>()
                .to_lowercase()
                .contains(&buf.nfkd().collect::<String>().to_lowercase())
            {
                possibilities.push(friend);
            }
        }
        if possibilities.len() == 0 {
            println!("No matches found");
            return Ok(());
        }

        println!("Possible matches:");
        for (i, friend) in possibilities.iter().enumerate() {
            println!("[{}] {} - {}", i + 1, friend.name, friend.availability);
        }
        print!("Type the number of the player to invite: ");
        stdout.flush()?;
        let mut buf = String::new();
        stdin.read_line(&mut buf)?;
        buf = buf.trim().into();
        let friend = possibilities
            .get(
                buf.parse::<usize>()?
                    .checked_sub(1)
                    .ok_or(ChatBrokenError::InvalidIndexError)?,
            )
            .ok_or(ChatBrokenError::InvalidIndexError)?;

        let r = lcu
            .post(
                "/lol-lobby/v2/lobby/invitations",
                &json!([{"toSummonerId": friend.summoner_id}]),
            )
            .await?;
        if r.status().is_success() {
            println!("Invited {}", friend.name);
        } else {
            println!("Failed to invite {}", friend.name);
        }
    } else {
        let invites_json = lcu
            .get("/lol-lobby/v2/received-invitations")
            .await?
            .json::<Value>()
            .await?;
        let invite_ids = invites_json
            .as_array()
            .ok_or(ChatBrokenError::InvalidDataError("invites is not array"))?
            .iter()
            .map(|inv| {
                Ok::<String, ChatBrokenError>(
                    inv.as_object()
                        .ok_or(ChatBrokenError::InvalidDataError("invite is not object"))?
                        .get("invitationId")
                        .ok_or(ChatBrokenError::InvalidDataError(
                            "no invitation id in invite",
                        ))?
                        .as_str()
                        .ok_or(ChatBrokenError::InvalidDataError(
                            "invitation id is not string",
                        ))?
                        .into(),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        for invite_id in invite_ids {
            let r = lcu
                .post(
                    &format!("/lol-lobby/v2/received-invitations/{}/accept", invite_id),
                    &json!({ "invitationId": invite_id }),
                )
                .await?;
            if r.status().is_success() {
                println!("Successfully accepted pending invitations");
            } else {
                println!("Failed to accept pending invitations");
            }
        }
    }
    Ok(())
}
