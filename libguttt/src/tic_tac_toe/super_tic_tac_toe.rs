use std::any::Any;
use std::rc::Rc;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use super::*;

pub struct SuperTicTacToeMove {
    pub pos: Position,
    pub submove: Rc<dyn Any>,
}

#[derive(Clone)]
pub struct SuperTicTacToe<T: GenericGame<P>,P: Player> {
    grid: [SubgameState<T,P>; 9],
    player: P,
    jump: Vec<Position>,
}

pub trait SubgameGen<T: GenericGame<P>,P:Player>: Fn(Position, P) -> T {}

impl<T,U: GenericGame<P>,P: Player> SubgameGen<U,P> for T where
    T: 'static + Fn(Position, P) -> U
{}

impl<T: GenericGame<P>, P: Player> SuperTicTacToe<T,P> {
    pub fn new<U: SubgameGen<T,P>>(player: P, new_board: U) -> Self {
        let mut grid: [SubgameState<T,P>; 9] = Default::default();
        for i in 0..9 {
            grid[i] =
                SubgameState::Playing(new_board(Position::from_cid(i as u8), player));
        }
        SuperTicTacToe {
            grid,
            player: player,
            jump: Self::alljump(),
        }
    }

    fn alljump() -> Vec<Position> {
        let mut v: Vec<Position> = Vec::with_capacity(9);
        for i in 0..9 {
            v.push(Position::from_cid(i as u8));
        }
        v
    }

    pub fn get_cell_ref(&self, pos: Position) -> &SubgameState<T,P> {
        &self.grid[pos.get_cid() as usize]
    }
    fn get_cell_mut(&mut self, pos: Position) -> &mut SubgameState<T,P> {
        &mut self.grid[pos.get_cid() as usize]
    }
    fn set_cell(&mut self, pos: Position, s: SubgameState<T,P>) {
        self.grid[pos.get_cid() as usize] = s;
    }
    fn get_cell(&mut self, pos: Position) -> SubgameState<T,P> {
        std::mem::replace(&mut self.grid[pos.get_cid() as usize],SubgameState::None)
    }

    fn set_jump(&mut self, ojump: &Option<Vec<Position>>, from: Position) {
        self.jump = self.sanitize_jump(ojump, from);
    }
    fn get_jump(&self) -> &Vec<Position> {
        &self.jump
    }

    fn sanitize_jump_raw(&self, ojump: &Option<Vec<Position>>, from: Position) -> Vec<Position> {
        let mut gvm = [false; 9];
        let mut res: Vec<Position> = Vec::with_capacity(9);

        for i in 0..9 {
            let x = i as u8;
            if let SubgameState::<T,P>::Playing(_) = self.get_cell_ref(Position::from_cid(x)) {
                gvm[i] = true;
            }
        }

        if let Some(jump) = ojump {
            for pos in jump.iter() {
                if gvm[pos.get_cid() as usize] {
                    res.push(*pos);
                }
            }
        } else {
            return self.sanitize_jump(&Some(vec![from]), from);
        }
        res
    }

    fn sanitize_jump(&self, ojump: &Option<Vec<Position>>, from: Position) -> Vec<Position> {
        let res = self.sanitize_jump_raw(ojump, from);
        if res.len() > 0 {
            res
        } else {
            self.sanitize_jump_raw(&Some(Self::alljump()), from)
        }
    }

    pub fn is_good_cell(&self, mopos: Position) -> bool {
        let mut b = false;

        for pos in self.jump.iter() {
            if *pos == mopos {
                b = true;
                break;
            }
        }

        b
    }

    fn check_win(&self) -> GameResult<P> {
        for i in 0..3 {
            let x = i as u8;
            // Check diagonals
            if x < 2 {
                if let SubgameState::<T,P>::Won(player,_) = Self::get_cell_ref(self, Position::from_xy(1, 1))
                {
                    if match x {
                        0 => {
                            if let SubgameState::<T,P>::Won(player2,_) =
                                Self::get_cell_ref(self, Position::from_xy(0, 0))
                            {
                                if let SubgameState::<T,P>::Won(player3,_) =
                                    Self::get_cell_ref(self, Position::from_xy(2, 2))
                                {
                                    player.get_uuid() == player2.get_uuid()
                                        && player2.get_uuid() == player3.get_uuid()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        1 => {
                            if let SubgameState::<T,P>::Won(player2,_) =
                                Self::get_cell_ref(self, Position::from_xy(2, 0))
                            {
                                if let SubgameState::<T,P>::Won(player3,_) =
                                    Self::get_cell_ref(self, Position::from_xy(0, 2))
                                {
                                    player.get_uuid() == player2.get_uuid()
                                        && player2.get_uuid() == player3.get_uuid()
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        }
                        _ => false,
                    } {
                        return GameResult::<P>::Won(*player);
                    };
                };
            };

            //Check vertical line
            if let SubgameState::<T,P>::Won(player,_) = Self::get_cell_ref(self, Position::from_xy(x, 1)) {
                if if let SubgameState::<T,P>::Won(player2,_) =
                    Self::get_cell_ref(self, Position::from_xy(x, 0))
                {
                    if let SubgameState::<T,P>::Won(player3,_) =
                        Self::get_cell_ref(self, Position::from_xy(x, 2))
                    {
                        player.get_uuid() == player2.get_uuid()
                            && player2.get_uuid() == player3.get_uuid()
                    } else {
                        false
                    }
                } else {
                    false
                } {
                    return GameResult::<P>::Won(*player);
                };
            };

            //Check horizontal line
            if let SubgameState::<T,P>::Won(player,_) = Self::get_cell_ref(self, Position::from_xy(1, x)) {
                if if let SubgameState::<T,P>::Won(player2,_) =
                    Self::get_cell_ref(self, Position::from_xy(0, x))
                {
                    if let SubgameState::<T,P>::Won(player3,_) =
                        Self::get_cell_ref(self, Position::from_xy(2, x))
                    {
                        player.get_uuid() == player2.get_uuid()
                            && player2.get_uuid() == player3.get_uuid()
                    } else {
                        false
                    }
                } else {
                    false
                } {
                    return GameResult::<P>::Won(*player);
                };
            };
        }
        let mut result = GameResult::<P>::Draw;
        for i in 0..9 {
            if let SubgameState::<T,P>::Playing(_) = Self::get_cell_ref(self, Position::from_cid(i)) {
                result = GameResult::<P>::Incomplete; break;
            };
        }
        result
    }
}

impl<T: GenericGame<P>, P: Player> GenericGame<P> for SuperTicTacToe<T,P> {
    fn create(player: P, pos: Position) -> Self {
      Self::new(player, |pos: Position, player: P| {
        T::create(player,pos)
      })
    }
    fn set_player(&mut self, player: P) {
        self.player = player;
    }
    fn get_player(&self) -> P {
        self.player
    }
    fn is_valid_move(&self, m: Rc<dyn Any>) -> bool {
        let moo = m.downcast_ref::<SuperTicTacToeMove>();
        
        if let Some(mo) = moo {
          if self.is_good_cell(mo.pos) {
              if let SubgameState::<T,P>::Playing(game) = self.get_cell_ref(mo.pos) {
                  game.is_valid_move(Rc::clone(&mo.submove))
              } else {
                  panic!("wtf?");
              }
          } else {
              false
          }
        } else {false}
    }
    fn get_valid_moves(&self) -> Vec<Rc<dyn Any>> {
        let mut vcells: Vec<Position> = Vec::with_capacity(9);
        for i in 0..9 {
            let pos = Position::from_cid(i as u8);
            if self.is_good_cell(pos) {
                vcells.push(pos);
            }
        }
        let mut goodmoves: Vec<Rc<dyn Any>> = Vec::with_capacity(9);
        for pos in vcells.iter() {
            if let SubgameState::<T,P>::Playing(game) = self.get_cell_ref(*pos) {
                let vmoves = game.get_valid_moves();
                for submove in vmoves.iter() {
                    goodmoves.push(Rc::new(SuperTicTacToeMove {
                        pos: *pos,
                        submove: Rc::clone(submove),
                    }));
                }
            } else {
                panic!("wtf?");
            }
        }
        goodmoves
    }
    fn mov(&mut self, m: Rc<dyn Any>) -> MoveData<P> {
        let mo = m.downcast_ref::<SuperTicTacToeMove>().expect(
            "type mismatch: SuperTicTacToe expects SuperTicTacToeMove, found I don't care what",
        );

        assert!(self.is_valid_move(Rc::clone(&m)), "invalid move");
        let player = self.player;

        if let SubgameState::Playing(mut game) = self.get_cell(mo.pos) {
            game.set_player(player);
            let mdata = game.mov(Rc::clone(&mo.submove));
            self.set_player(mdata.next_player);
            let result;
            let md = match mdata.result {
                GameResult::Won(player) => {
                    self.set_cell(mo.pos, SubgameState::Won(player,game));
                    result = self.check_win();
                    self.set_jump(&Some(Self::alljump()), mo.pos);
                    MoveData {
                        jump: Some(vec![mo.pos]),
                        next_player: mdata.next_player,
                        result,
                    }
                }
                GameResult::Draw => {
                    self.set_cell(mo.pos, SubgameState::Draw(game));
                    result = self.check_win();
                    self.set_jump(&mdata.jump, mo.pos);
                    MoveData {
                        jump: None,
                        next_player: mdata.next_player,
                        result,
                    }
                }
                GameResult::Incomplete => {
                    self.set_cell(mo.pos, SubgameState::Playing(game));
                    result = self.check_win();
                    self.set_jump(&mdata.jump, mo.pos);
                    MoveData {
                        jump: None,
                        next_player: mdata.next_player,
                        result,
                    }
                },
            };
            match result {
              GameResult::Incomplete => (),
              _ => {self.set_jump(&None,mo.pos)}
            };
            md
        } else {
            panic!("wtf?");
        }
    }
}
