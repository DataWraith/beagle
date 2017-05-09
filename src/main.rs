extern crate time;
extern crate rand;
extern crate hyper;
extern crate fnv;

#[macro_use]
extern crate serde_derive;

extern crate serde_json;


mod direction;
mod mv;
mod state;
mod game;
mod hero;
mod board;
mod position;
mod tile;
mod bot;
mod transposition_table;
mod zobrist;
mod lru;

use direction::Direction;
use std::io::Read;
// use std::hash::{Hash, Hasher, SipHasher};
use hyper::client::*;
use hyper::header::ContentType;
use hyper::Url;

fn main() {
    unsafe {
        zobrist::ZOBRIST = zobrist::ZobristTable::default();
    }
    let mut bot = bot::Bot::new();
    let client = Client::new();
    let mut res = client
        .post("http://vindinium.org/api/arena")
        .header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
        .body("key=eqwxqpa8")
        .send()
        .unwrap();
    let mut body: String = "".to_owned();

    if res.status != hyper::Ok {
        println!("{:?}", res.status_raw());
        return;
    }

    res.read_to_string(&mut body).ok();

    let mut state: state::State = serde_json::from_str(&body).unwrap();
    state.game.board.initialize();

    let mut new_state: state::State;

    loop {
        let mv = bot.choose_move(&mut state);
        println!("{}: {}", state.game.turn, mv);

        res = client
            .post(Url::parse(&state.play_url).unwrap())
            .header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
            .body(&(String::from("key=eqwxqpa8&dir=") + mv.into()))
            .send()
            .unwrap();

        if res.status != hyper::Ok {
            println!("{:?}", res.status_raw());
            return;
        }

        body = String::default();
        res.read_to_string(&mut body).ok();
        new_state = serde_json::from_str(&body).unwrap();

        if new_state.game.finished {
            break;
        }

        if state.game.heroes[0].crashed != new_state.game.heroes[0].crashed ||
           state.game.heroes[1].crashed != new_state.game.heroes[1].crashed ||
           state.game.heroes[2].crashed != new_state.game.heroes[2].crashed ||
           state.game.heroes[3].crashed != new_state.game.heroes[3].crashed {

            state = new_state.clone();
            state.game.board.initialize();
        } else {
            state.make_move(mv);
            let h_idx = new_state.game.turn % 4;
            for i in 1..4 {
                let nextmv = &new_state.game.heroes[(h_idx + i) % 4].last_dir;
                match nextmv.as_ref() {
                    "North" => state.make_move(Direction::North),
                    "East" => state.make_move(Direction::East),
                    "South" => state.make_move(Direction::South),
                    "West" => state.make_move(Direction::West),
                    "Stay" => state.make_move(Direction::Stay),
                    _ => unreachable!(),
                };
            }
        }
    }

    println!("{}", state.view_url);
}
