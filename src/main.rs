use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use chess_lib::chess::{Board, Error};
use clap::*;
use colored::*;

#[derive(Debug, Clone)]
enum ServerOrClient {
    Server(u16),
    Client(String, u16),
}

fn parse_key_val<T, U>(
    s: &str,
) -> Result<ServerOrClient, Box<dyn std::error::Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: std::error::Error + Send + Sync + 'static,
{
    let data: Vec<&str> = s.split(':').collect();
    let mut iter = data.iter();

    // check iter length, if length is 1, then it's a server
    if iter.len() == 1 {
        let port = iter.next().unwrap().parse::<u16>()?;
        Ok(ServerOrClient::Server(port))
    } else {
        let host = iter.next().unwrap().parse::<String>()?;
        let port = iter.next().unwrap().parse::<u16>()?;
        Ok(ServerOrClient::Client(host, port))
    }
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_parser = parse_key_val::<String, u16>)]
    multiplayer: Option<ServerOrClient>,
}

fn main() -> Result<(), Error> {
    // arguments
    let args = Args::parse();

    match args.multiplayer {
        Some(ServerOrClient::Server(port)) => {
            server(port)?;
        }
        Some(ServerOrClient::Client(host, port)) => {
            client(host, port)?;
        }
        _ => singleplayer()?,
    }

    Ok(())
}

fn client(host: String, port: u16) -> Result<(), Error> {
    let mut board = Board::default_board()?;
    let mut error: Option<String> = None;

    let mut stream = TcpStream::connect(format!("{}:{}", host, port))?;

    loop {
        // clear screen
        print!("{}[2J", 27 as char);

        if error.is_some() {
            println!("\n{}\n", error.clone().unwrap().red());
        }

        // print board
        draw_for_black(&board);

        let mut input = String::new();

        if board.turn() == chess_lib::chess::Color::White {
            println!("\n{} to move:", board.turn().to_string().bold());
            let mut buffer = [0; 4];
            stream.read_exact(&mut buffer)?;
            input = String::from_utf8_lossy(&buffer).to_string();
        } else {
            // print turn
            println!("\n{} to move:", board.turn().to_string().bold());
            print!("> ");

            // flush stdout
            std::io::stdout().flush().unwrap();

            // a move consists of 4 characters (e.g. e2e4)
            std::io::stdin().read_line(&mut input).unwrap();
            input = input.trim().to_string();

            // send move to server
            stream.write_all(input.as_bytes())?;
        }

        error = match board.move_piece(&input) {
            Ok(_) => None,
            Err(e) => Some(e.to_string()),
        }
    }
}

fn server(port: u16) -> Result<(), Error> {
    let server = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("Server started on port {}", port);

    for stream in server.incoming() {
        let mut stream = stream?;
        let mut board = Board::default_board()?;

        let mut error: Option<String> = None;

        loop {
            // clear screen
            print!("{}[2J", 27 as char);

            if error.is_some() {
                println!("\n{}\n", error.clone().unwrap().red());
            }

            // print board
            draw_for_white(&board);

            let mut input = String::new();

            // server is white, goes first
            if board.turn() == chess_lib::chess::Color::White {
                // print turn
                println!("\n{} to move:", board.turn().to_string().bold());
                print!("> ");

                // flush stdout
                std::io::stdout().flush().unwrap();

                // a move consists of 4 characters (e.g. e2e4)
                std::io::stdin().read_line(&mut input).unwrap();
                input = input.trim().to_string();

                stream.write_all(input.as_bytes())?;
            } else {
                println!("\n{} to move:", board.turn().to_string().bold());
                // client is black, goes second
                let mut buffer = [0; 4];
                stream.read(&mut buffer).unwrap();
                input = String::from_utf8_lossy(&buffer).to_string();
            }

            error = match board.move_piece(&input) {
                Ok(_) => None,
                Err(e) => Some(e.to_string()),
            }
        }
    }

    Ok(())
}

fn singleplayer() -> Result<(), Error> {
    let mut board = Board::default_board()?;

    let mut error: Option<String> = Option::None;

    loop {
        // clear screen
        print!("{}[2J", 27 as char);

        if error.is_some() {
            println!("\n{}\n", error.clone().unwrap().red());
        }

        // draw a chess board with file and ranks identifiers
        draw_for_white(&board);

        // print turn
        println!("\n{} to move:", board.turn().to_string().bold());
        print!("> ");

        // flush stdout
        std::io::stdout().flush().unwrap();

        // a move consists of 4 characters (e.g. e2e4)
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        let mut cmd = input.split_whitespace().into_iter();

        match cmd.next() {
            // Exit commands
            Some("q") => break,
            Some("quit") => break,
            Some("exit") => break,

            // Save command, takes parameter of file
            Some("save") => {
                let filename = cmd.next().unwrap_or("game.txt");
                board.save(filename)?;
            }

            Some("load") => {
                let filename = cmd.next().unwrap_or("game.txt");
                board.load(filename)?;
            }

            Some(turn) => {
                error = match board.move_piece(turn) {
                    Ok(_) => None,
                    Err(e) => Some(e.to_string()),
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn draw_for_white(board: &Board) {
    println!("  ａｂｃｄｅｆｇｈ");
    for rank in 0..8 {
        let rank = 8 - rank;
        print!("{} ", rank);
        for file in 0..8 {
            let square_color = if (file + rank) % 2 == 0 {
                Color::White
            } else {
                Color::BrightBlue
            };
            let piece = board.get_piece(file, rank - 1);

            if piece.is_some() {
                let piece = piece.unwrap();
                print!(
                    "{}",
                    piece.to_string().color(Color::Black).on_color(square_color)
                );
            } else {
                print!("{}", " ".on_color(square_color));
            }

            print!("{}", " ".on_color(square_color));
        }
        println!(" {}", rank);
    }
    println!("  ａｂｃｄｅｆｇｈ");
}

fn draw_for_black(board: &Board) {
    println!("  ｈｇｆｅｄｃｂａ");
    for rank in 0..8 {
        let rank = 1 + rank;
        print!("{} ", rank);
        for file in (0..8).rev() {
            let square_color = if (file + rank) % 2 == 0 {
                Color::White
            } else {
                Color::BrightBlue
            };
            let piece = board.get_piece(file, rank - 1);

            if piece.is_some() {
                let piece = piece.unwrap();
                print!(
                    "{}",
                    piece.to_string().color(Color::Black).on_color(square_color)
                );
            } else {
                print!("{}", " ".on_color(square_color));
            }

            print!("{}", " ".on_color(square_color));
        }
        println!(" {}", rank);
    }
    println!("  ｈｇｆｅｄｃｂａ");
}
