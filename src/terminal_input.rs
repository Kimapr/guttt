use crate::tic_tac_toe::*;

pub trait TerminalInput {
  fn input(&mut self, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>>;
}

pub struct InputHandle {
  store: Vec<Rc<dyn Any>>,
}

#[derive(Copy, Clone, Debug)]
pub enum Control {
  Left,
  Right,
  Up,
  Down,
  Push
}

struct ControlBackpropagation {
  con: Option<Control>,
  pos: Position,
  rec: Option<Box<ControlBackpropagation>>
}

struct Substore([Vec<Rc<dyn Any>>;9]);

struct PositionStore(Position);

struct HighestLevel;

impl InputHandle {
  fn input<T: GenericGame<P> + TerminalInput, P: Player>(&mut self, game: &mut T, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let mut arg: Vec<Rc<dyn Any>> = vec!(Rc::new(HighestLevel));
    arg.extend(args);
    arg.extend(self.store);
    self.store = game.input(arg);
  }
}

impl<T: GenericGame<P>, P: Player> TerminalInput for SuperTicTacToe<T,P> {
  fn input(&mut self, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    let mut control: Option<Control> = None;
    let mut cpos = Position::default();
    let mut substore: [Vec<Rc<dyn Any>>] = [vec!();9];
    let mut store: Vec<Rc<dyn Any>> = vec!();
    
    for rv in args.iter() {
      if let Some(c) = rv.downcast_ref::<Control>() {
        control = Some(c);
      }
      if let Some(PositionStore(pos)) = rv.downcast_ref::<PositionStore>() {
        cpos = pos;
      }
      if let Some(subs) = rv.downcast_ref::<Substore>() {
        substore = subs;
      }
    }
    
    let arg: Vec<Rc<dyn Any>> = vec!();
    
    for rv in substore[cpos.get_cid() as usize].iter() {
      if let Some(c) = rv.downcast_ref::<Position> {
        
      }
    }
    
    let sgr = self.get_cell_ref(cpos).input();
    
    for rv in sgr.iter() {
      if let Some(c) = rv.downcast_ref::<ControlBackpropagation>() {
        
      }
    }
    
    substore[cpos.get_cid() as usize] = sgr;
  }
}