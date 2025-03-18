use matrix_sdk::{
    Client, Room, RoomState,
    config::SyncSettings,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use std::{env, process::exit, time::SystemTime};

async fn on_room_message(init_time: SystemTime, event: OriginalSyncRoomMessageEvent, room: Room) {
    if room.state() != RoomState::Joined || event.sender == "@picovm:matrix.org" {
        return;
    }
    let MessageType::Text(text_content) = event.content.msgtype else {
        return;
    };

    let mut iter = text_content.body.split_whitespace();

    if iter.clone().count() < 1 {
        return;
    }

    let content = match iter.nth(0).unwrap() {
        "uptime" => {
            let s = format!("{:#?}", init_time.elapsed().unwrap());
            RoomMessageEventContent::text_plain(s)
        }
        _ => RoomMessageEventContent::text_plain("mmm... no such command."),
    };

    room.send(content).await.unwrap();
}

async fn login_and_sync(
    homeserver_url: String,
    username: String,
    password: String,
) -> anyhow::Result<()> {
    // Note that when encryption is enabled, you should use a persistent store to be
    // able to restore the session with a working encryption setup.
    // See the `persist_session` example.
    let client = Client::builder()
        .homeserver_url(homeserver_url)
        .build()
        .await
        .unwrap();
    client
        .matrix_auth()
        .login_username(&username, &password)
        .initial_device_display_name("pico")
        .await?;

    let init_time = SystemTime::now();

    let response = client.sync_once(SyncSettings::default()).await.unwrap();
    client.add_event_handler(move |ev, room| on_room_message(init_time.clone(), ev, room));

    let settings = SyncSettings::default().token(response.next_batch);
    client.sync(settings).await?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let (homeserver_url, username, password) =
        match (env::args().nth(1), env::args().nth(2), env::args().nth(3)) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                eprintln!(
                    "Usage: {} <homeserver_url> <username> <password>",
                    env::args().next().unwrap()
                );
                exit(1)
            }
        };

    login_and_sync(homeserver_url, username, password).await?;
    Ok(())
}
