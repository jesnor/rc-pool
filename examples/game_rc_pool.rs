use rc_pool::{RcPool, StrongRef, StrongRefTrait, WeakRef, WeakSliceExt};

const MANUAL_DROP: bool = true;

type PlayerRef = StrongRef<'static, Player, MANUAL_DROP>;
type PlayerWeakRef = WeakRef<'static, Player, MANUAL_DROP>;
type PlayerPool = RcPool<Player, MANUAL_DROP>;
type PlayerPoolRef = &'static PlayerPool;
type GameRef = &'static Game;

struct Player {
    #[allow(unused)]
    game: GameRef,

    name: String,
    friends: Vec<PlayerWeakRef>,
}

struct Game {
    players: PlayerPoolRef,
}

impl Game {
    fn new(players: PlayerPoolRef) -> Self {
        Self { players }
    }

    fn add_player(self: GameRef, name: &str) -> PlayerRef {
        self.players.insert(Player {
            game: self,
            name: name.to_owned(),
            friends: Default::default(),
        })
    }
}

impl PartialEq for Game {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self as *const Self, other as *const Self)
    }
}

fn main() {
    let players = Box::leak(Box::new(RcPool::new(1)));
    let game = Box::leak(Box::new(Game::new(players)));

    {
        let mut p1 = game.add_player("Sune");
        let mut p2 = game.add_player("Berra");
        let mut p3 = game.add_player("Arne");
        let mut p4 = game.add_player("Svenne");

        p1.borrow_mut().friends.push(p2.weak());
        p1.borrow_mut().friends.push(p3.weak());
        p2.borrow_mut().friends.push(p1.weak());
        p2.borrow_mut().friends.push(p4.weak());
        p3.borrow_mut().friends.push(p1.weak());
        p3.borrow_mut().friends.push(p2.weak());
        p3.borrow_mut().friends.push(p4.weak());
        p4.borrow_mut().friends.push(p3.weak());
    }

    // Since MANUAL_DROP is true, players are not dropped when all strong references are dropped
    // You can still iterate over live player slots in the pool to obtain strong references
    for p in game.players.iter() {
        println!("{}", p.name);

        for f in p.friends.iter_strong() {
            println!("  {}", f.name);
        }

        println!();
    }
}
