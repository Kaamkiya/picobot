use matrix_sdk::{
    Client, Room, RoomState,
    attachment::AttachmentConfig,
    config::SyncSettings,
    ruma::events::room::message::{
        MessageType, OriginalSyncRoomMessageEvent, RoomMessageEventContent,
    },
};
use std::{fs, time::SystemTime};

mod latex;
mod quotes;
mod xkcd;

#[derive(serde::Deserialize)]
struct Conf {
    username: String,
    password: String,
    homeserver: String,
}

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
        "latex" => {
            let c: Vec<&str> = iter.collect();
            let bytes = latex::render(c.concat()).await.unwrap();

            room.send_attachment(
                "latex.png",
                &mime::IMAGE_PNG,
                bytes,
                AttachmentConfig::new(),
            )
            .await
            .unwrap();
        }
        "md2html" => {
            let c: Vec<&str> = iter.collect();

            room.send(RoomMessageEventContent::text_plain(markdown::to_html(
                c.concat().as_str(),
            )))
            .await
            .unwrap();
        }
        "quote" => {
            let quote = quotes::random().await.unwrap();
            room.send(RoomMessageEventContent::text_plain(format!(
                "{} - {}",
                quote.content, quote.author
            )))
            .await
            .unwrap();
        }
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

    let conf_str = fs::read_to_string("picobot.toml").expect("Failed to read config file.");
    let conf: Conf = toml::from_str(&conf_str).expect("Failed to parse config file.");

    login_and_sync(conf.homeserver, conf.username, conf.password).await?;
    Ok(())
}
