use std::io::{self, Stdin, Stdout, Write};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use unicode_normalization::UnicodeNormalization;

use lcu::Lcu;

#[derive(Serialize, Deserialize, Debug)]
struct Friend {
    name: String,
    #[serde(rename(deserialize = "summonerId"))]
    summoner_id: u64,
    availability: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Invite {
    #[serde(rename(deserialize = "invitationId"))]
    invitation_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
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
    let friends_list = lcu
        .get("/lol-chat/v1/friends")
        .await?
        .json::<Vec<Friend>>()
        .await?;
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
) -> Result<()> {
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
        if possibilities.is_empty() {
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
                    .context("invalid index")?,
            )
            .context("invalid index")?;

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
        let invites = lcu
            .get("/lol-lobby/v2/received-invitations")
            .await?
            .json::<Vec<Invite>>()
            .await?;
        if invites.is_empty() {
            println!("No invites found");
        }
        for invite in invites {
            let r = lcu
                .post(
                    &format!(
                        "/lol-lobby/v2/received-invitations/{}/accept",
                        invite.invitation_id
                    ),
                    &json!({ "invitationId": invite.invitation_id }),
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
