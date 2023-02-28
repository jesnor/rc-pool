#![allow(unused)]

use rc_pool::{StrongRefTrait, WeakRefTrait};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

type GameRef = Rc<Game>;
type PlayerRef = Rc<Player>;
type PlayerWeakRef = Weak<Player>;

struct Player {
    game: GameRef,
    name: String,
    friends: RefCell<Vec<PlayerWeakRef>>,
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

struct Game {
    players: RefCell<Vec<PlayerRef>>,
}

impl Game {
    fn new() -> Self {
        Self {
            players: Default::default(),
        }
    }

    fn add_player(self: &GameRef, name: &str) -> PlayerRef {
        let p = Rc::new(Player {
            game: self.clone(),
            name: name.to_owned(),
            friends: Default::default(),
        });

        self.players.borrow_mut().push(p.clone());
        p
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

fn main() {
    let game = Rc::new(Game::new());
    let p1 = game.add_player("Sune");
    let p2 = game.add_player("Berra");
    p1.friends.borrow_mut().push(p2.weak());
    p2.friends.borrow_mut().push(p1.weak());

    for p in game.players.borrow().iter() {
        println!("{}", p.name);

        for f in p.friends.borrow().iter().flat_map(|r| r.strong()) {
            println!("  {}", f.name);
        }

        println!();
    }
}
