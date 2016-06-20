#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate serde_json;
extern crate hyper;

use std::io::Read;
use hyper::client::*;
use hyper::header::ContentType;
use hyper::Url;

#[derive(Deserialize, Debug)]
pub enum Direction {
    North,
    East,
    South,
    West,
    Stay
}

#[derive(Deserialize, Debug)]
pub struct Position {
    x: i8,
    y: i8,
}

#[derive(Deserialize, Debug)]
pub struct Hero {
    id: u8,
    name: String,
	//#[serde(rename="userId")]
    //user_id: String,
    //elo: u16,
    pos: Position,
    //lastDir: Option<Direction>,
    life: u8,
    gold: u16,
	#[serde(rename="mineCount")]
    mine_count: u8,
	#[serde(rename="spawnPos")]
    spawn_pos: Position,
    crashed: bool,
}

#[derive(Deserialize, Debug)]
pub struct Board {
    size: u8,
    tiles: String,
}

#[derive(Deserialize, Debug)]
pub struct Game {
    id: String,
    turn: u16,
	#[serde(rename="maxTurns")]
    max_turns: u16,
    heroes: [Hero; 4],
    board: Board,
    finished: bool,
}

#[derive(Deserialize, Debug)]
pub struct State {
    game: Game,
    hero: Hero,
    token: String,
	#[serde(rename="viewUrl")]
    view_url: String,
	#[serde(rename="playUrl")]
    play_url: String,
}

fn main() {
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
	
    let mut state : State = serde_json::from_str(&body).unwrap();
	
	println!("{}", state.view_url);
	
	while !state.game.finished {
		res = client.post(Url::parse(&state.play_url).unwrap())
			.header(ContentType("application/x-www-form-urlencoded".parse().unwrap()))
			.body("key=eqwxqpa8&dir=Stay")
			.send()
			.unwrap();
			
		if res.status != hyper::Ok {
		  println!("{:?}", res.status_raw());
		  return;
		}
		
		body = "".to_owned();
		res.read_to_string(&mut body).ok();
		state = serde_json::from_str(&body).unwrap();
	}
}
