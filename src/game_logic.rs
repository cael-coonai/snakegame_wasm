
pub const GRID_W:usize = 30;
pub const GRID_H:usize = (GRID_W/3)*2; //enforces 3:2 aspect ratio for board
pub const SNAKE_LENGTH_DEFAULT:usize = 4;  // length is one-based, ∴ len > 0
const SNAKE_LENGTH_GROWTH:usize = 3;
const SCORE_INCREMENT:u32 = 50;


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction{Up,Dn,Lf,Rt}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum GridCell{
  #[default] Nothing,
  Snake(usize,Direction),
  Apple,
  Wall
}

pub enum GameEvent {
  GeneralMovement,
  GraceTick,
  AppleCollected,
  GameOver
}

#[derive(Debug, PartialEq)]
struct Snake {
  len: usize,
  body: [Option<(usize,usize)>;GRID_W*GRID_H]
}

#[derive(Debug, PartialEq)]
pub struct Board {
  cells: [[GridCell;GRID_W];GRID_H],
  snake: Snake,
  grace_frame: bool,
  score: u32
}


impl Snake {
  const fn new() -> Snake {
    Snake {
      len: 1,
      body: [None;GRID_W*GRID_H]
    }
  }
}

impl Board {
  pub const fn new() -> Board {
    Board {
      cells: [[GridCell::Nothing;GRID_W];GRID_H],
      snake: Snake::new(),
      grace_frame: false,
      score: 0
    }
  }

  pub fn peek(&self, x:usize, y:usize) -> GridCell {
    self.cells[y][x]
  }

  fn owrt_cell(&mut self, new_cell:GridCell, x:usize, y:usize) {
    self.cells[y][x] = new_cell;
  }

  pub fn generate_walls(&mut self) -> &mut Board {
    self.cells[0]        = [GridCell::Wall;GRID_W];
    self.cells[GRID_H-1] = [GridCell::Wall;GRID_W];
    for y in 1..GRID_H-1 {
      self.owrt_cell(GridCell::Wall,       0,y);
      self.owrt_cell(GridCell::Wall,GRID_W-1,y);
    }
    self
  }

  fn catalogue_empty_spaces(&self) -> Vec<(usize,usize)> {
    (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| (x,y))).filter(|pos| {
      let (x,y) = *pos;
      self.peek(x,y) == GridCell::Nothing
     }).collect::<Vec<(usize,usize)>>()
  }

  fn spawn_snake(&mut self, x:usize, y:usize, len:usize, dir:Direction) {
    self.owrt_cell(GridCell::Snake(len-1,dir),x,y);
    self.snake.body[0] = Some((x,y));
  }

  pub fn generate_snake(&mut self) -> Result<&mut Board,&str>  {
    let available_locations = self.catalogue_empty_spaces();
    if available_locations.is_empty() {return Err("Board Full")}
    let (snake_x,snake_y) =
      available_locations[crate::rand::usize(0,available_locations.len())];
    let relative_x = snake_x as f32 - (GRID_W as f32 / 2.0);
    let relative_y = snake_y as f32 - (GRID_H as f32 / 2.0);
    let direction;
    if relative_x.abs() < relative_y.abs() {
      if relative_y > 0.0      {direction = Direction::Up}
      else                     {direction = Direction::Dn}
    } else if relative_x > 0.0 {direction = Direction::Lf}
    else                       {direction = Direction::Rt}
    self.spawn_snake(snake_x,snake_y,SNAKE_LENGTH_DEFAULT,direction);
    Ok(self)
  }

  pub fn change_facing_direction(&mut self, direction:Direction) {
    let (head_x,head_y) = self.snake.body[0].unwrap();
    let stack = match self.peek(head_x,head_y) {
      GridCell::Snake(s,_) => s,
      _ =>
        panic!(
          "Attempted get stack of non-snake at {head_x},{head_y}",
        )
    };
    if self.snake.len > 1 {
      let (neck_x,neck_y) = self.snake.body[1].unwrap();
      let (next_x,next_y) = match direction {
        Direction::Up => (head_x as isize, head_y as isize - 1isize),
        Direction::Dn => (head_x as isize, head_y as isize + 1isize),
        Direction::Lf => (head_x as isize - 1isize, head_y as isize),
        Direction::Rt => (head_x as isize + 1isize, head_y as isize),
      };
      let (next_x,next_y) = wrap_cells(next_x,next_y);
       if next_x == neck_x && next_y == neck_y {return;}
    }
    self.owrt_cell(GridCell::Snake(stack,direction),head_x,head_y);
  }

  fn spawn_apple(&mut self, x:usize, y:usize) {
    // self.apple = (x,y);
    self.owrt_cell(GridCell::Apple,x,y);
  }

  pub fn generate_apple(&mut self) -> Result<&mut Board,&str> {
    let available_locations = self.catalogue_empty_spaces();
    if available_locations.len() == 0 {return Err("Board Full")}
    let (apple_x,apple_y) =
      available_locations[crate::rand::usize(0,available_locations.len())];
    // self.apple = (apple_x,apple_y);
    self.spawn_apple(apple_x,apple_y);
    Ok(self)
  }

  fn swap_cells(&mut self,(x0,y0):(usize,usize),(x1,y1):(usize,usize)) {
    let temp = self.peek(x1,y1);
    self.owrt_cell(self.peek(x0,y0),x1,y1);
    self.owrt_cell(temp,x0,y0);
  }

  pub fn do_game_tick(&mut self) -> GameEvent {
    let game_event; 
    //Look ahead
    let (x0,y0) = self.snake.body[0].unwrap();
    let (x1,y1,d) = match self.peek(x0,y0) {
      GridCell::Snake(_,Direction::Up) =>
        (x0 as isize, y0 as isize - 1isize, Direction::Up),
      GridCell::Snake(_,Direction::Dn) =>
        (x0 as isize, y0 as isize + 1isize, Direction::Dn),
      GridCell::Snake(_,Direction::Lf) =>
        (x0 as isize - 1isize, y0 as isize, Direction::Lf),
      GridCell::Snake(_,Direction::Rt) =>
        (x0 as isize + 1isize, y0 as isize, Direction::Rt),
      _ => panic!("Snake head not found at {x0}, {y0}")
    };
 
    //Handle Collisions
    let (x1,y1) = wrap_cells(x1,y1);
    match self.peek(x1,y1) {
      GridCell::Nothing    => {
        self.grace_frame = false;
        game_event = GameEvent::GeneralMovement;
      },
      GridCell::Apple      => {
        self.grace_frame = false;
        let (x,y) = self.snake.body[self.snake.len-1].unwrap();
        match self.peek(x,y) {
          GridCell::Snake(s,d) => {self.owrt_cell(
              GridCell::Snake(s+SNAKE_LENGTH_GROWTH,d),x,y);}
          _ => panic!("Attemted to add to stack of non-snake at {x},{y}")
        };
        match self.generate_apple() {
          Ok(_) => self.increase_score(1),
          Err("Board Full") => self.increase_score(50),
          Err(e) => panic!("An unknown error has occured:\n{e}"),
        }
        game_event = GameEvent::AppleCollected;
      },
      GridCell::Snake(0,_) if self.snake.body[self.snake.len-1] == Some((x1,y1))
        => {
      self.grace_frame = false;
      self.swap_cells(self.snake.body[0].unwrap(),
        self.snake.body[self.snake.len-1].unwrap());
      self.snake.body[0..self.snake.len].rotate_right(1);
      game_event = GameEvent::GeneralMovement;
      return game_event;
    }
      GridCell::Snake(_,_)  |
      GridCell::Wall        => {
        if !self.grace_frame {
          self.grace_frame = true;
          game_event = GameEvent::GraceTick;
          return game_event;
        } else {
          game_event = GameEvent::GameOver;
          return game_event;
        }
      },
    }

    //Move Snake
    self.snake.body[0..=self.snake.len].rotate_right(1);
    { //Move or Unstack Tail
      let (x,y) = self.snake.body[self.snake.len].unwrap();
      (self.snake.body[self.snake.len],self.snake.len) =
        match self.peek(x,y) {
        GridCell::Snake(0,_) => {self.owrt_cell(GridCell::Nothing,x,y);
                                  (None,self.snake.len)},
        GridCell::Snake(s,d) => {self.owrt_cell(GridCell::Snake(s-1,d),x,y);
                                  (Some((x,y)),self.snake.len+1)},
        _ => panic!("Attempted to prone non-snake cell at {x},{y}")
      }
    }
    self.owrt_cell(GridCell::Snake(0,d),x1,y1);
    self.snake.body[0] = Some((x1,y1));
    game_event
  }

  pub fn query_score(&self) -> u32 {
    self.score
  }

  pub fn query_grace(&self) -> bool {
    self.grace_frame
  }

  pub fn query_head_location(&self) -> Option<(usize,usize)> {
    self.snake.body[0]
  }

  fn increase_score(&mut self,increase:u32) {
    self.score += increase*SCORE_INCREMENT;
  }
}

fn wrap_cells(x:isize, y:isize) -> (usize,usize) {
  let x = if x >= GRID_W as isize{0}
    else if x < 0isize {GRID_W-1}
    else {x as usize};
  let y = if y >= GRID_H as isize{0}
    else if y < 0isize {GRID_H-1}
    else {y as usize};
  (x,y)
}