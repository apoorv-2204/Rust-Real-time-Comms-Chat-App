#[macro_use]
extern crate rocket;

use rocket::form::Form;
use rocket::fs::{relative, FileServer};
use rocket::response::stream::{Event, EventStream};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::select;
use rocket::tokio::sync::broadcast::{channel, error::RecvError, Sender};
use rocket::{Shutdown, State};

// #[get("/rtc")]
// fn rtc() -> &'static str {
//     "Hello, Starting RTC"
// }

#[derive(Debug, Clone, Serialize, Deserialize, FromForm)]
#[serde(crate = "rocket::serde")]

struct RTCResponse {
    #[field(validate = len(..50))]
    pub room: String,
    #[field(validate = len(..20))]
    pub user_name: String,
    #[field(validate = len(..301))]
    pub message: String,
}

#[get("/events")]
async fn events(queue: &State<Sender<RTCResponse>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };

            yield Event::json(&msg);
        }
    }
}

#[post("/response", data = "<form>")]
fn post(form: Form<RTCResponse>, queue: &State<Sender<RTCResponse>>) {
    // sending will fail if no one is listening
    let _res = queue.send(form.into_inner());
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<RTCResponse>(4096).0)
        // .mount("/home", routes![rtc]);
        .mount("/", routes![post, events])
        .mount("/", FileServer::from(relative!("static")))
}
