use rc_pool::{RcPool, StrongRef, StrongRefTrait, WeakRef, WeakRefTrait};

const MANUAL_DROP: bool = true;

struct Player<'t> {
    #[allow(unused)]
    game: GameRef<'t>,

    name: String,
    friends: Vec<PlayerWeakRef<'t>>,
}

type PlayerRef<'t> = StrongRef<'t, Player<'t>, MANUAL_DROP>;
type PlayerWeakRef<'t> = WeakRef<'t, Player<'t>, MANUAL_DROP>;
type PlayerPool<'t> = RcPool<Player<'t>, MANUAL_DROP>;

struct Game<'t> {
    players: &'t PlayerPool<'t>,
}

impl<'t> Game<'t> {
    fn new(players: &'t PlayerPool<'t>) -> Self {
        Self { players }
    }

    fn add_player(&'t self, name: &str) -> PlayerRef<'t> {
        self.players.insert(Player {
            game: self,
            name: name.to_owned(),
            friends: Default::default(),
        })
    }
}

impl<'t> PartialEq for Game<'t> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

type GameRef<'t> = &'t Game<'t>;

fn main() {
    let players = RcPool::new(1);
    let game = Game::new(&players);
    let mut p1 = game.add_player("Sune");
    let mut p2 = game.add_player("Berra");
    p1.borrow_mut().friends.push(p2.weak());
    p2.borrow_mut().friends.push(p1.weak());

    for _ in 0..2 {
        for p in game.players.iter() {
            println!("{}", p.name);

            for f in p.friends.iter().flat_map(|r| r.strong()) {
                println!("  {}", f.name);
            }

            println!();
        }
    }
}
