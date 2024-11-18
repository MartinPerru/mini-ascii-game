use std::io::{self, Write};
use termion::raw::IntoRawMode;
use termion::input::TermRead;
use termion::event::Key;
use termion::color::{self, Fg, Reset};
use rand::Rng;

#[derive(Clone, PartialEq)]
enum Terrain {
    Wall,    // #
    Floor,   // .
    Water,   // ~
    Grass,   // ,
    Lava,    // ^
    Portal,  // =
    Key,     // Nueva llave
}

struct Game {
    map: Vec<Vec<Terrain>>,
    player_x: usize,
    player_y: usize,
    lives: u32,
    portals: Vec<((usize, usize), (usize, usize))>, // Lista de portales y sus destinos 
}

impl Game {
    fn new() -> Game {
        let mut rng = rand::thread_rng();
        let mut map = vec![vec![Terrain::Wall; 80]; 32];
        let mut portals = Vec::new(); // Lista de portales

        // Llenar los bordes con paredes
        for y in 0..32 {
            for x in 0..80 {
                if y == 11 || y == 31 || x == 0 || x == 79 {
                    map[y][x] = Terrain::Wall;
                } else {
                    // Generar terreno aleatorio para el interior
                    let random_num = rng.gen_range(0.0..1.0);
                    map[y][x] = match random_num {
                        n if n < 0.70 => Terrain::Floor,  // 70% probabilidad - más espacio para moverse
                        n if n < 0.80 => Terrain::Grass,  // 10% probabilidad - decorativo
                        n if n < 0.85 => Terrain::Water,  // 5% probabilidad - obstáculos de agua
                        n if n < 0.87 => Terrain::Lava,   // 2% probabilidad - menos lava para no frustrar
                        n if n < 0.90 => {
                            // Crear un portal y agregarlo a la lista
                            let portal_dest = (rng.gen_range(12..32), rng.gen_range(1..79));
                            // Asegurarse de que el destino no sea el mismo que la posición actual
                            if (y, x) != portal_dest {
                                portals.push(((y, x), portal_dest)); // Agregar el portal y su destino
                            }
                            Terrain::Portal 
                        },
                        _ => Terrain::Wall,               // 10% probabilidad - obstáculos dispersos
                    };
                }
            }
        }
        
        map[rng.gen_range(12..32)][rng.gen_range(1..79)]=Terrain::Key;

        Game {
            map,
            player_x: 1,
            player_y: 12,
            lives: 3,
            portals, // Agregar la lista de portales
        }
    }

    fn draw(&self) {
        print!("{}{}", termion::clear::All, termion::cursor::Goto(1, 12));
        io::stdout().flush().unwrap();
    
        let mut stdout = io::stdout();
        
        for y in 11..self.map.len() {
            for x in 0..self.map[y].len() {
                if x == self.player_x && y == self.player_y {
                    // Jugador en amarillo brillante
                    write!(stdout, "{}@{}", Fg(color::Yellow), Fg(Reset)).unwrap();
                } else {
                    match self.map[y][x] {
                        Terrain::Wall => {
                            // Paredes en gris
                            write!(stdout, "{}#{}", Fg(color::White), Fg(Reset)).unwrap();
                        },
                        Terrain::Floor => {
                            // Suelo en gris oscuro
                            write!(stdout, "{}.{}", Fg(color::LightBlack), Fg(Reset)).unwrap();
                        },
                        Terrain::Water => {
                            // Agua en azul
                            write!(stdout, "{}~{}", Fg(color::Blue), Fg(Reset)).unwrap();
                        },
                        Terrain::Grass => {
                            // Hierba en verde
                            write!(stdout, "{},{}", Fg(color::Green), Fg(Reset)).unwrap();
                        },
                        Terrain::Lava => {
                            // Lava en rojo
                            write!(stdout, "{}^{}", Fg(color::Red), Fg(Reset)).unwrap();
                        },
                        Terrain::Portal => {
                            // Portal en amarillo
                            write!(stdout, "{}={}", Fg(color::LightYellow), Fg(Reset)).unwrap();
                        },
                        Terrain::Key => {
                            // Llave en amarillo
                            write!(stdout, "{}?{}", Fg(color::Yellow), Fg(Reset)).unwrap();
                        },
                    }
                }
            }
            write!(stdout, "\r\n").unwrap();
        }
        // Mostrar el número de vidas restantes
        print!("{}", termion::cursor::Goto(1, 42));
        println!("{}Vidas restantes: {}{}", Fg(color::Cyan), self.lives, Fg(Reset));

        stdout.flush().unwrap();
    }

    fn move_player(&mut self, dx: i32, dy: i32) {
        let new_x = (self.player_x as i32 + dx) as usize;
        let new_y = (self.player_y as i32 + dy) as usize;

        if new_x >= 80 || new_y >= 32 {
            return; // No permitir movimiento fuera del mapa
        }

        // Actualizar la posición del jugador antes de verificar el terreno
        self.player_x = new_x;
        self.player_y = new_y;

        match self.map[new_y][new_x] {
            Terrain::Wall => {
                // Revertir la posición si hay una pared
                self.player_x = (self.player_x as i32 - dx) as usize;
                self.player_y = (self.player_y as i32 - dy) as usize;
                return;
            },
            Terrain::Water | Terrain::Lava => {
                self.lives -= 1; // Restar una vida
                if self.lives == 0 {
                    print!("{}", termion::clear::All);
                    println!("{}¡Has perdido todas tus vidas!{}", Fg(color::Red), Fg(Reset));
                    std::process::exit(0); // Salir del juego
                } else {
                    self.player_x = 1;
                    self.player_y = 12;
                    print!("{}", termion::cursor::Goto(1, 12)); 
                    println!("{}¡Has caído! Te quedan {} vidas. Volviendo al inicio...{}", Fg(color::Red), self.lives, Fg(Reset));
                    return;
                }
            },
            Terrain::Portal => {
                // Transportar al jugador a la posición del portal
                for ((portal_x, portal_y), (dest_x, dest_y)) in &self.portals {
                    if *portal_x == new_y && *portal_y == new_x {
                        // Verificar que la posición nueva esté en los límites jugables del mapa.
                        if *dest_x >= 1 && *dest_x <= 80 && *dest_y >= 12 && *dest_y <= 32 {
                            // Asegurarse de que el destino no sea el mismo que la posición actual
                            if self.player_x != *dest_x || self.player_y != *dest_y {
                                self.player_x = *dest_x;
                                self.player_y = *dest_y;
                                print!("{}", termion::cursor::Goto(*dest_x as u16, *dest_y as u16));
                                println!("{}¡Te has transportado a un nuevo lugar!{}", Fg(color::Cyan), Fg(Reset));
                                return; // Salir después de transportarse
                            }
                        }
                    }
                }
            },
            Terrain::Key => {
                print!("{}", termion::clear::All);
                println!("{}¡Has recogido la llave! Ganaste el juego!{}", Fg(color::Green), Fg(Reset));
                std::process::exit(0); // Salir del juego
            },
            _ => (),
        }
}
}

fn draw_help_text() {
    let base_y = 43; 
    print!("{}", termion::cursor::Goto(1, base_y));
    println!("{}Leyenda:{}", Fg(color::White), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 1));
    println!("{}@{} = Jugador", Fg(color::Yellow), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 2));
    println!("{}#{} = Pared", Fg(color::White), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 3));
    println!("{}.{} = Suelo", Fg(color::LightBlack), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 4));
    println!("{}~{} = Agua (¡Cuidado! Volverás al inicio)", Fg(color::Blue), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 5));
    println!("{},{} = Hierba", Fg(color::Green), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 6));
    println!("{}^{} = Lava (¡Cuidado! Volverás al inicio)", Fg(color::Red), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 7));
    println!("{}={} = Portal", Fg(color::LightYellow), Fg(Reset));

    print!("{}", termion::cursor::Goto(1, base_y + 8));
    println!("{}?{} = Llave", Fg(color::Yellow), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 10));
    println!("{}Controles:{}", Fg(color::White), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 11));
    println!("{}Flechas{} = Mover", Fg(color::Cyan), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 12));
    println!("{}q{} = Salir", Fg(color::Cyan), Fg(Reset));
    
    print!("{}", termion::cursor::Goto(1, base_y + 13));
    println!("{}r{} = Regenerar mapa", Fg(color::Cyan), Fg(Reset));
    
    io::stdout().flush().unwrap();
}

fn main() {
    let stdin = io::stdin();
    let _raw = io::stdout().into_raw_mode().unwrap();
    
    print!("{}{}", termion::cursor::Hide, termion::clear::All);
    io::stdout().flush().unwrap();
    
    let mut game = Game::new();
    game.draw();
    draw_help_text();
    
    for key in stdin.keys() {
        match key.unwrap() {
            Key::Char('q') => break,
            Key::Char('r') => {
                game = Game::new();
                game.draw();
                print!("{}", termion::cursor::Goto(1, 12)); 
                println!("{}¡Mapa regenerado!{}", Fg(color::Green), Fg(Reset));
            },
            Key::Left  => game.move_player(-1, 0),
            Key::Right => game.move_player(1, 0),
            Key::Up    => game.move_player(0, -1),
            Key::Down  => game.move_player(0, 1),
            _ => (),
        }
        game.draw();
        draw_help_text();
    }

    print!("{}", termion::cursor::Show);
    io::stdout().flush().unwrap();
}