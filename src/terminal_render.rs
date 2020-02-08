use std::fmt;
use fmt::Formatter;
use std::ops::{Range, Deref, DerefMut};
use std::cmp::min;
use std::rc::Rc;
use std::cell::{RefCell,RefMut,BorrowMutError};
use std::any::Any;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

pub struct TerminalCanvas {
  grid: Vec<Vec<Grapheme>>,
  prevclips: Vec<Rect>,
  clip: Rect,
  w: usize,
  h: usize,
  fg: ColorEnum,
  bg: ColorEnum
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ColorEnum {
  Reset,

  Black,
  Blue,
  Cyan,
  Green,
  Magenta,
  Red,
  White,
  Yellow,

  LightBlack,
  LightBlue,
  LightCyan,
  LightGreen,
  LightMagenta,
  LightRed,
  LightWhite,
  LightYellow,
}

impl color::Color for ColorEnum {
  fn write_fg(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      ColorEnum::Reset => color::Reset.write_fg(f),

      ColorEnum::Black => color::Black.write_fg(f),
      ColorEnum::Blue => color::Blue.write_fg(f),
      ColorEnum::Cyan => color::Cyan.write_fg(f),
      ColorEnum::Green => color::Green.write_fg(f),
      ColorEnum::Magenta => color::Magenta.write_fg(f),
      ColorEnum::Red => color::Red.write_fg(f),
      ColorEnum::White => color::White.write_fg(f),
      ColorEnum::Yellow => color::Yellow.write_fg(f),

      ColorEnum::LightBlack => color::LightBlack.write_fg(f),
      ColorEnum::LightBlue => color::LightBlue.write_fg(f),
      ColorEnum::LightCyan => color::LightCyan.write_fg(f),
      ColorEnum::LightGreen => color::LightGreen.write_fg(f),
      ColorEnum::LightMagenta => color::LightMagenta.write_fg(f),
      ColorEnum::LightRed => color::LightRed.write_fg(f),
      ColorEnum::LightWhite => color::LightWhite.write_fg(f),
      ColorEnum::LightYellow => color::LightYellow.write_fg(f),
    }
  }
  fn write_bg(&self, f: &mut Formatter) -> fmt::Result {
    match self {
      ColorEnum::Reset => color::Reset.write_bg(f),

      ColorEnum::Black => color::Black.write_bg(f),
      ColorEnum::Blue => color::Blue.write_bg(f),
      ColorEnum::Cyan => color::Cyan.write_bg(f),
      ColorEnum::Green => color::Green.write_bg(f),
      ColorEnum::Magenta => color::Magenta.write_bg(f),
      ColorEnum::Red => color::Red.write_bg(f),
      ColorEnum::White => color::White.write_bg(f),
      ColorEnum::Yellow => color::Yellow.write_bg(f),

      ColorEnum::LightBlack => color::LightBlack.write_bg(f),
      ColorEnum::LightBlue => color::LightBlue.write_bg(f),
      ColorEnum::LightCyan => color::LightCyan.write_bg(f),
      ColorEnum::LightGreen => color::LightGreen.write_bg(f),
      ColorEnum::LightMagenta => color::LightMagenta.write_bg(f),
      ColorEnum::LightRed => color::LightRed.write_bg(f),
      ColorEnum::LightWhite => color::LightWhite.write_bg(f),
      ColorEnum::LightYellow => color::LightYellow.write_bg(f),
    }
  }
}

struct Grapheme {
  fg: ColorEnum,
  bg: ColorEnum,
  glyph: String
}

#[derive(Debug, Copy, Clone)]
struct Rect {
  x:usize,
  y:usize,
  w:usize,
  h:usize
}

impl Clone for Grapheme {
  fn clone(&self) -> Self {
    Grapheme{
      fg: self.fg,
      bg: self.bg,
      glyph: self.glyph.clone(),
    }
  }
}

impl TerminalCanvas {
  pub fn new(w: usize,h: usize, fg: ColorEnum, bg: ColorEnum) -> TerminalCanvas {
    let mut v: Vec<Vec<Grapheme>> = Vec::with_capacity(h);
    for _y in (Range{start: 0, end: h}) {
      let mut vv: Vec<Grapheme> = Vec::with_capacity(w);
      for _x in (Range{start: 0, end: w}) {
        vv.push(Grapheme{fg, bg, glyph: String::from(" ")});
      }
      v.push(vv);
    };
    TerminalCanvas{grid: v, w,h, fg, bg, prevclips: vec!(), clip: Rect{x:0,y:0, w,h}}
  }
  pub fn set_color(&mut self, fg: ColorEnum, bg: ColorEnum) {
    self.fg = fg;
    self.bg = bg;
  }
  pub fn get_color(&self) -> (ColorEnum,ColorEnum) {
    (self.fg,self.bg)
  }
  pub fn set_fg(&mut self, c: ColorEnum) {
    self.fg = c;
  }
  pub fn set_bg(&mut self, c: ColorEnum) {
    self.bg = c;
  }
  pub fn get_fg(&self) -> ColorEnum {
    self.fg
  }
  pub fn get_bg(&self) -> ColorEnum {
    self.bg
  }
  pub fn clip(&mut self, x: usize, y: usize, w: usize, h: usize) {
    let (rx,ry,rw,rh) = (self.clip.x,self.clip.y,self.clip.w,self.clip.h);
    let clip = Rect{x: rx+x, y: ry+y, w,h};
    assert!((x+w <= rw) && (y+h <= rh),format!("Clipping rectangle {:?} is out of bounds {:?}", clip, self.clip));
    self.prevclips.push(self.clip);
    self.clip = clip;
  }
  pub fn unclip(&mut self) {
    self.clip = self.prevclips.pop().expect("stack underflow: more clip()'s than unclip()'s");
  }

  pub fn set(&mut self, x: isize, y: isize, ss: &str) {
    if (x < self.clip.w as isize) && (y < self.clip.h as isize) {
      let mut yi: isize = 0;
      let mut xi: isize = 0;
      for glyph in ss.graphemes(true) {
        let mut isEscape = false;
        for ch in glyph.chars() {
          if ch.is_ascii_control() {
            isEscape = true;
            match ch {
              '\r' => {xi = 0;},
              '\n' => {yi = yi+1;},
              _ => (),//(panic!("Unknown escape: {:?}",ch)),
            }
          }
        }
        if !isEscape {
          let (px,py) = (x+xi,y+yi);
          if (px < self.clip.w as isize) && (py < self.clip.h as isize) &&
             (px >= 0) && (py >= 0) {
            self.grid[py as usize + self.clip.y][px as usize + self.clip.x] = Grapheme{
              fg: self.fg,
              bg: self.bg,
              glyph: String::from(glyph),
            }
          }
          xi = xi + 1;
        }
      }
    };
  }
  pub fn set_canv(&mut self, x: isize, y: isize, canv: &TerminalCanvas) {
    let (w,h) = canv.get_size();
    let (fg,bg) = self.get_color();
    for iy in (Range{start:0, end: h}) {
      for ix in (Range{start:0, end: w}) {
        let (px,py) = (x+(ix as isize),y+(iy as isize));
        let g = canv.get(ix as usize,iy as usize);
        self.set_color(g.fg,g.bg);
        self.set(px,py, g.glyph.as_str());
      }
    }
    self.set_color(fg,bg);
  }
  fn get(&self, x: usize, y: usize) -> &Grapheme {
    assert!((x < self.clip.w) && (y < self.clip.h),"out of bounds");
    &self.grid[y+self.clip.y][x+self.clip.x]
  }
}

impl fmt::Display for TerminalCanvas {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut var = String::new();
    let mut fg = ColorEnum::Reset;
    let mut bg = ColorEnum::Reset;
    for line in self.grid.iter() {
      for glyph in line.iter() {
        if fg != glyph.fg {
          fg = glyph.fg;
          var.push_str(format!("{}",color::Fg(fg)).as_str());
        }
        if bg != glyph.bg {
          bg = glyph.bg;
          var.push_str(format!("{}",color::Bg(bg)).as_str());
        }
        var.push_str(glyph.glyph.as_str());
      }
      var.push_str("\r\n");
    }
    var.pop();var.pop();
    write!(f,"{1}{0}{1}",var,format!("{}{}",color::Fg(color::Reset),color::Bg(color::Reset)))
  }
}

impl TerminalRender for TerminalCanvas {
  fn get_size(&self) -> (usize,usize) {
    return (self.clip.w,self.clip.h)
  }
  fn render(&self,term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {
    term.set_canv(0,0,&self);
    vec!()
  }
}

pub trait TerminalRender {
  fn get_size(&self) -> (usize,usize) {
    (1,1)
  }
  fn render(&self,term: &mut TerminalCanvas, args: Vec<Rc<dyn Any>>) -> Vec<Rc<dyn Any>> {vec!()}
}
