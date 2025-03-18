use matrix_sdk::{
    Client, Room, RoomState,
    attachment::AttachmentConfig,
    config::SyncSettings,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use std::{env, process::exit, time::SystemTime};

mod xkcd;

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

    match iter.nth(0).unwrap() {
        "uptime" => {
            let s = format!("{:#?}", init_time.elapsed().unwrap());
            room.send(RoomMessageEventContent::text_plain(s))
                .await
                .unwrap();
        }
        "xkcd" => {
            let which = iter.next().unwrap_or("");

            let data = match which {
                "latest" => xkcd::latest().await.unwrap(),
                "random" => xkcd::random().await.unwrap(),
                "" => xkcd::latest().await.unwrap(),
                "404" => xkcd::nth("405").await.unwrap(), // /404/info.0.json returns a 404 error.
                _ => xkcd::nth(which).await.unwrap(),
            };

            room.send_attachment(
                "xkcd.jpg",
                &mime::IMAGE_JPEG,
                data.imgcontent,
                AttachmentConfig::new().caption(Some(format!(
                    "{} #{} - {}/{}/{}",
                    data.comic.title,
                    data.comic.num,
                    data.comic.year,
                    data.comic.month,
                    data.comic.day
                ))),
            )
            .await
            .unwrap();
        }
        _ => (),
    };
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
    //    tracing_subscriber::fmt::init();

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
