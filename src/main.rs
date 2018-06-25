extern crate termion;
use std::error::Error;
use std::fs::File;
use std::io::{stdout, Read, Write};
use termion::raw::IntoRawMode;

struct Display {
  width: u16,
  height: u16,
  left: usize,
  top: usize,
}

struct World {
  mat: [[char; 1024]; 30],
  width: u16,
  height: u16,
}

struct Player {
  x: f32,
  y: f32,
  speed_x: f32,
  speed_y: f32,
  on_ground: bool,
  walking: bool,
  walking_dir: i8,
}

struct DrawingError {
  msg: String,
}

const SPEED_X_MAX: f32 = 0.6;
const GRAVITY: f32 = 0.1;
const JUMP: f32 = 0.85;

fn main() {
  let stdout = stdout();
  let mut stdout = stdout.lock().into_raw_mode().unwrap();
  let mut stdin = termion::async_stdin().bytes();

  let (width, height) = match termion::terminal_size() {
    Ok((height, width)) => (height, width),
    Err(_) => panic!("Could not get terminal size!"),
  };

  let mut display = Display {
    width,
    height,
    left: 0,
    top: 0,
  };

  let mut player = Player {
    x: 1.0,
    y: 1.0,
    speed_x: 0.0,
    speed_y: 0.0,
    on_ground: false,
    walking: false,
    walking_dir: 0,
  };

  let mut world = World {
    mat: [[' '; 1024]; 30],
    width: 1024,
    height: 30,
  };

  load_level(&mut world, &mut player);

  loop {
    match stdin.next() {
      Some(Ok(b)) => {
        if b == b'q' {
          panic!("Quit");
        } else if b == b'w' {
          jump(&mut player)
        } else if b == b'a' {
          if player.walking && player.walking_dir > 0 {
            player.walking = false;
          } else {
            walk(&mut player, -1)
          }
        } else if b == b'd' {
          if player.walking && player.walking_dir < 0 {
            player.walking = false;
          } else {
            walk(&mut player, 1)
          }
        } else {
          ()
        }
      }
      _ => (),
    }
    simulate(&world, &mut player);
    match draw(&mut stdout, &mut display, &world, &player) {
      Ok(_) => (),
      Err(e) => panic!("Error during draw: {}", e.msg),
    }
    std::thread::sleep(std::time::Duration::from_millis(20));
  }
}

impl std::convert::From<std::io::Error> for DrawingError {
  fn from(err: std::io::Error) -> Self {
    DrawingError {
      msg: String::from(err.description()),
    }
  }
}

fn walk(player: &mut Player, dir: i8) {
  player.walking = true;
  player.walking_dir = dir;
}

fn jump(player: &mut Player) {
  if player.on_ground {
    player.speed_y = -JUMP;
  }
}

fn simulate(world: &World, player: &mut Player) {
  player.on_ground = false;

  let mut new_y = player.y + player.speed_y;
  let mut new_x = player.x + player.speed_x;
  let (steps, step_x, step_y) = if player.speed_y.abs() > player.speed_x.abs() {
    (
      player.speed_y,
      player.speed_x / player.speed_y.abs(),
      player.speed_y / player.speed_y.abs(),
    )
  } else {
    (
      player.speed_x,
      player.speed_x / player.speed_x.abs(),
      player.speed_y / player.speed_x.abs(),
    )
  };
  for step in 0..(steps.abs() as usize + 1) {
    let check_x = player.x + step as f32 * step_x;
    let check_y = player.y + step as f32 * step_y;
    let mut collision = false;

    if player.speed_y < 0.0 && check_y >= 1.0 {
      if world.mat[(check_y - 1.0) as usize][check_x as usize] == '#' {
        player.speed_y = 0.0;
        new_y = check_y;
        collision = true;
      }
    }

    if player.speed_y > 0.0 {
      if world.mat[(check_y + 1.0) as usize][check_x as usize] == '#' {
        player.speed_y = 0.0;
        new_y = check_y;
        player.on_ground = true;
        collision = true;
      }
    }

    if player.speed_x < 0.0 && check_x >= 1.0 {
      if world.mat[check_y as usize][(check_x - 1.0) as usize] == '#' {
        player.speed_x = 0.0;
        new_x = player.x;
        collision = true;
      }
    }

    if player.speed_x > 0.0 {
      if world.mat[check_y as usize][(check_x + 1.0) as usize] == '#' {
        player.speed_x = 0.0;
        new_x = player.x;
        collision = true;
      }
    }

    if collision {
      break;
    }
  }

  // update position
  player.y = new_y;
  player.x = new_x;

  if player.x < 1.0 {
    player.x = 1.0;
  } else if player.x > world.width as f32 - 1.0 {
    player.x = world.width as f32 - 1.0;
  }

  // calculate next
  if player.on_ground {
    if player.walking {
      player.speed_x = player.speed_x + player.walking_dir as f32 * 0.15;
    } else {
      player.speed_x = player.speed_x * 0.85;
    }
  } else {
    if player.walking {
      player.speed_x = player.speed_x + player.walking_dir as f32 * 0.15;
    } else {
      player.speed_x = player.speed_x * 0.99;
    }
  }

  if player.speed_x.abs() > SPEED_X_MAX {
    player.speed_x = player.walking_dir as f32 * SPEED_X_MAX;
  } else if player.speed_x.abs() < 0.1 {
    player.speed_x = 0.0;
  }

  player.speed_y += GRAVITY;
}

fn draw(
  stdout: &mut termion::raw::RawTerminal<std::io::StdoutLock>,
  display: &mut Display,
  world: &World,
  player: &Player,
) -> Result<(), DrawingError> {
  if player.x > display.left as f32 + (display.width as f32 * 0.66) {
    if display.left + (display.width as usize) < world.width as usize {
      display.left = (player.x - display.width as f32 * 0.66) as usize;
    }
  } else if player.x < display.left as f32 + (display.width as f32 * 0.33) {
    if display.left > 0 {
      display.left = (player.x - display.width as f32 * 0.33) as usize;
    }
  }

  let mut buffer = String::new();
  let width_max = if display.width > world.width {
    world.width
  } else {
    display.width
  };
  let height_max = if display.height > world.height {
    world.height
  } else {
    display.height
  };
  for y in display.top..(display.top + height_max as usize) {
    let mut cur_line = String::new();
    for x in display.left..(display.left + width_max as usize) {
      if x == player.x as usize && y == player.y as usize {
        cur_line.push_str("@");
      } else {
        cur_line.push(world.mat[y as usize][x as usize]);
      }
    }
    buffer.push_str(cur_line.as_str());
    if y < world.height as usize - 1 {
      buffer.push_str("\n\r");
    }
  }
  write!(stdout, "{}{}", termion::cursor::Goto(1, 1), buffer)?;
  Ok(())
}

fn load_level(world: &mut World, player: &mut Player) -> () {
  let mut file = File::open("./levels/1.lvl").expect("File could not be opened!");
  let mut contents = String::new();
  match file.read_to_string(&mut contents) {
    Ok(_) => (),
    Err(_) => panic!("Could not load level!"),
  }
  let lines = contents.split("\n");
  let mut y = 0;
  let mut x = 0;
  for line in lines {
    for pos in line.chars() {
      if pos == '@' {
        player.x = x as f32;
        player.y = y as f32;
      } else {
        world.mat[y][x] = pos;
      }
      x += 1;
    }
    y += 1;
    x = 0;
  }
}
