#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate time;
extern crate rand;
extern crate serde_json;
extern crate hyper;

mod state;
mod game;
mod hero;
mod board;
mod position;
mod tile;
mod bot;

use std::io::Read;
use std::hash::{Hash, Hasher, SipHasher};
use hyper::client::*;
use hyper::header::ContentType;
use hyper::Url;

fn main() {
    let mut bot = bot::Bot::new();
    let client = Client::new();
	let mut res = client.post("http://vindinium.org/api/training")
		.header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
		.body("key=eqwxqpa8")
		.send()
		.unwrap();
	let mut body : String = "".to_owned();

	if res.status != hyper::Ok {
	  println!("{:?}", res.status_raw());
	  return;
	}

	res.read_to_string(&mut body).ok();

    let mut state : state::State = serde_json::from_str(&body).unwrap();
    state.game.board.initialize();

    let mut new_state : state::State;

	while !state.game.finished {
        let mv = bot.choose_move(&state);
		println!("{}: {}", state.game.turn, mv);
		
		res = client.post(Url::parse(&state.play_url).unwrap())
			.header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
			.body(&(String::from("key=eqwxqpa8&dir=") + mv))
			.send()
			.unwrap();

		if res.status != hyper::Ok {
		  println!("{:?}", res.status_raw());
		  return;
		}

		body = String::default();
		res.read_to_string(&mut body).ok();
		new_state = serde_json::from_str(&body).unwrap();
        new_state.game.board.initialize();

        state.make_move(mv);
        let h_idx = new_state.game.turn % 4;
        for i in 1..4 {
            let ref mv = new_state.game.heroes[(h_idx + i) % 4].last_dir;
            state.make_move(&mv);

            println!("{}", state.game.board);
            println!("\n\n");            
        }

        if state != new_state {
            println!("{}", new_state.game.board);
            assert_eq!(state, new_state);
        }
	}

	println!("{}", state.view_url);
}
