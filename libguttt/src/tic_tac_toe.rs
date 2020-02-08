use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;

mod generic;
mod super_tic_tac_toe;
mod quantum_tic_tac_toe;

pub use quantum_tic_tac_toe::*;
pub use super_tic_tac_toe::*;
pub use generic::*;

#[derive(Clone,Debug)]
pub struct DummyGame<P: Player> {player: P}

impl<P: Player> DummyGame<P> {
  pub fn new(player: P) -> Self {
    DummyGame{player}
  }
}

impl<P: Player> GenericGame<P> for DummyGame<P> {
  fn create(player: P, pos: Position) -> Self {
    Self::new(player)
  }
  fn mov(&mut self, m: Rc<dyn Any>) -> MoveData<P> {
    let player = self.player;
    self.set_player(player.next_player());
    MoveData {
      jump: None,
      next_player: self.player,
      result: GameResult::Won(player)
    }
  }
  
  fn is_valid_move(&self, m: Rc<dyn Any>) -> bool {
    true
  }
  fn get_valid_moves(&self) -> Vec<Rc<dyn Any>> {
    vec!(Rc::new(true))
  }
  
  fn set_player(&mut self, player: P) {
    self.player = player;
  }
  fn get_player(&self) -> P {
    self.player
  }
}

#[derive(Clone)]
pub struct SuperDummyGame<T: GenericGame<P>,P: Player> {game: SubgameState<T,P>}

impl<T:GenericGame<P>,P: Player> SuperDummyGame<T,P> {
  pub fn new(game: T) -> Self {
    SuperDummyGame{game: SubgameState::<T,P>::Playing(game)}
  }
  pub fn get_game_ref(&self) -> &SubgameState<T,P> {
    &self.game
  }
}

impl<T:GenericGame<P>,P: Player> GenericGame<P> for SuperDummyGame<T,P> {
  fn create(player: P, pos: Position) -> Self {
    Self::new(T::create(player,pos))
  }
  fn mov(&mut self, m: Rc<dyn Any>) -> MoveData<P> {
    let state = std::mem::replace(&mut self.game,SubgameState::None);
    if let SubgameState::Playing(mut game) = state {
      let mdata = game.mov(m);
      match &mdata.result {
        GameResult::Won(player) => {
          self.game = SubgameState::Won(*player,game);
        },
        GameResult::Draw => {
          self.game = SubgameState::Draw(game);
        },
        GameResult::Incomplete => {
          self.game = SubgameState::Playing(game);
        }
      }
      mdata
    } else { 
      panic!("The game is over. Your can't make the move.");
    }
  }
  
  fn is_valid_move(&self, m: Rc<dyn Any>) -> bool {
    if let SubgameState::<T,P>::Playing(game) = &self.game {
      game.is_valid_move(m)
    } else {
      false
    }
  }
  fn get_valid_moves(&self) -> Vec<Rc<dyn Any>> {
    if let SubgameState::<T,P>::Playing(game) = &self.game {
      game.get_valid_moves()
    } else {
      vec!()
    }
  }
  
  fn set_player(&mut self, player: P) {
    if let SubgameState::<T,P>::Playing(game) = &mut self.game {
      game.set_player(player)
    }
  }
  fn get_player(&self) -> P {
    if let SubgameState::<T,P>::Playing(game) = &self.game {
      game.get_player()
    } else {
      panic!("tried to get a player where there is none");
    }
  }
}