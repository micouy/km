use powierza_coefficient::powierża_coefficient;
use std::convert::AsRef;
use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::{fs, io};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

fn print_current_path<W, P>(mut stdout: W, current_path: P)
where
    W: Write,
    P: AsRef<Path>,
{
    writeln!(
        stdout,
        "\r {}{}{}",
        termion::color::Fg(termion::color::Yellow),
        current_path
            .as_ref()
            .canonicalize()
            .unwrap()
            .to_string_lossy(),
        termion::color::Fg(termion::color::Reset),
    )
    .unwrap();
}

fn clear_screen<W>(mut stdout: W)
where
    W: Write,
{
    write!(
        stdout,
        "\r{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::clear::CurrentLine
    )
    .unwrap();

    stdout.flush().unwrap();
}

fn print_query<W>(mut stdout: W, query: &str)
where
    W: Write,
{
    writeln!(
        stdout,
        "\r> {}{}{}",
        termion::color::Fg(termion::color::Red),
        query,
        termion::color::Fg(termion::color::Reset),
    )
    .unwrap();

    stdout.flush().unwrap();
}

fn print_entries<W>(mut stdout: W, entries: &[PathBuf], cursor: usize)
where
    W: Write,
{
    let n_shown = 30;
    let n_skip = cursor
        .saturating_sub(n_shown / 2)
        .min(entries.len().saturating_sub(n_shown));

    for (i, entry) in entries.iter().enumerate().skip(n_skip).take(n_shown) {
        let background_color = if i == cursor {
            &termion::color::Green as &dyn termion::color::Color
        } else {
            &termion::color::Reset as &dyn termion::color::Color
        };

        if entry.is_dir() {
            writeln!(
                stdout,
                "\r  {}{}{}{}{}",
                termion::color::Fg(termion::color::Blue),
                termion::color::Bg(background_color),
                entry.file_name().unwrap().to_string_lossy(),
                termion::color::Fg(termion::color::Reset),
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
        } else {
            writeln!(
                stdout,
                "\r  {}{}{}",
                termion::color::Bg(background_color),
                entry.file_name().unwrap().to_string_lossy(),
                termion::color::Bg(termion::color::Reset),
            )
            .unwrap();
        }
    }

    stdout.flush().unwrap();
}

fn read_entries<P>(path: P) -> Vec<PathBuf>
where
    P: AsRef<Path>,
{
    let mut entries = fs::read_dir(path)
        .unwrap()
        .map(|res| res.map(|e| e.path()).unwrap())
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| {
        let by_is_dir = a.is_dir().cmp(&b.is_dir()).reverse();
        let by_path = a.cmp(b);

        by_is_dir.then(by_path)
    });

    return entries;
}

fn main() {
    let mut current_path = env::current_dir().unwrap();
    let mut entries = read_entries(&current_path);

    let stdin = io::stdin();
    let mut stdout = io::stdout()
        .into_raw_mode()
        .unwrap()
        .into_alternate_screen()
        .unwrap();

    let mut movement_cursor: usize = 0;
    let mut query_cursor: usize = 0;
    let mut query = String::new();

    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    clear_screen(&mut stdout);
    print_current_path(&mut stdout, &current_path);
    print_query(&mut stdout, &query);
    print_entries(&mut stdout, &entries, movement_cursor);

    for c in stdin.keys() {
        write!(
            stdout,
            "{}{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            termion::clear::CurrentLine
        )
        .unwrap();

        match c.unwrap() {
            Key::Alt('k') => {
                let final_cursor = if query.is_empty() {
                    movement_cursor
                } else {
                    query_cursor
                };

                movement_cursor = final_cursor.saturating_sub(1);
                query_cursor = 0;
                query.clear();
            }
            Key::Alt('j') => {
                let final_cursor = if query.is_empty() {
                    movement_cursor
                } else {
                    query_cursor
                };

                movement_cursor = (final_cursor + 1).min(entries.len().saturating_sub(1));
                query_cursor = 0;
                query.clear();
            }
            Key::Alt('l') => {
                let final_cursor = if query.is_empty() {
                    movement_cursor
                } else {
                    query_cursor
                };

                if entries[final_cursor].is_dir() {
                    current_path = entries[final_cursor].clone();
                    entries = read_entries(&current_path);

                    movement_cursor = 0;
                    query_cursor = 0;
                    query.clear();
                }
            }
            Key::Alt('h') => {
                let previous_path = current_path.clone();
                current_path = current_path
                    .parent()
                    .map(|path| path.into())
                    .unwrap_or(current_path.clone());
                entries = read_entries(&current_path);

                movement_cursor = entries
                    .iter()
                    .position(|entry| entry == &previous_path)
                    .unwrap_or(0);
                query_cursor = 0;
                query.clear();
            }
            Key::Alt('\r') => {
                let final_cursor = if query.is_empty() {
                    movement_cursor
                } else {
                    query_cursor
                };

                if entries[final_cursor].is_dir() {
                    current_path = entries[final_cursor].clone();

                    write!(io::stderr(), "{}", current_path.display()).unwrap();
                    io::stderr().flush().unwrap();

                    break;
                }
            }
            Key::Char('\n') => {
                write!(io::stderr(), "{}", current_path.display()).unwrap();
                io::stderr().flush().unwrap();

                break;
            }
            Key::Char(c) => {
                query.push(c);

                query_cursor = entries
                    .iter()
                    .enumerate()
                    .filter(|(_i, entry)| entry.is_dir())
                    .filter_map(|(i, entry)| {
                        let file_name = entry.file_name().unwrap().to_string_lossy().to_lowercase();

                        powierża_coefficient(&query, &file_name)
                            .map(|coeff| (i, (coeff, file_name.len())))
                    })
                    .min_by_key(|(_i, score)| *score)
                    .map(|(i, _coeff)| i)
                    .unwrap_or(movement_cursor);
            }
            Key::Backspace => {
                query.clear();

                query_cursor = 0;
            }
            Key::Ctrl('c') => break,
            Key::Esc => break,
            _ => {}
        }

        let final_cursor = if query.is_empty() {
            movement_cursor
        } else {
            query_cursor
        };

        clear_screen(&mut stdout);
        print_current_path(&mut stdout, &current_path);
        print_query(&mut stdout, &query);
        print_entries(&mut stdout, &entries, final_cursor);
    }

    write!(stdout, "{}", termion::cursor::Show).unwrap();
    stdout.flush().unwrap();
}
