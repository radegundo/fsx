use std::{ env::current_dir, fs::{ remove_dir_all, remove_file, File }, path::PathBuf, vec };

use color_eyre::owo_colors::OwoColorize;
use crossterm::{ event::{ self, Event, KeyCode, KeyModifiers } };

use ratatui::{
    layout::{ Alignment, Constraint, Direction, Layout, Rect },
    style::*,
    widgets::{
        Block,
        BorderType,
        Borders,
        List,
        ListItem,
        ListState,
        Paragraph,
        Row,
        Table,
        TableState,
        Widget,
    },
    DefaultTerminal,
    Frame,
};
mod fsx_func;
use crate::fsx_func::*;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let mut app = App::new();
    app.logic.list_state.select_first();
    app.logic.list_items.sort();
    app.logic.parent_list.sort();
    app.logic.contents = App::build_contents(
        &app.logic.dir.join(PathBuf::from(app.logic.list_items[0].clone()))
    );
    app.update_selected_file();
    let app_result = app.run(terminal);
    ratatui::restore();
    app_result
}

impl App {
    fn run(&mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        loop {
            //Render
            terminal.draw(|f| self.render(f))?;
            self.logic.index = self.logic.list_state.selected();
            //Handle input
            if let Event::Key(key) = event::read()? {
                match self.logic.app_mode {
                    AppMode::Normal => {
                        match (key.modifiers, key.code) {
                            | (KeyModifiers::CONTROL, KeyCode::Char('c'))
                            | (_, KeyCode::Esc)
                            | (_, KeyCode::Char('q')) => {
                                return Ok(());
                            }
                            (_, KeyCode::Char('j')) => {
                                self.normal_action(NormalActions::NextFile);
                            }
                            (_, KeyCode::Char('k')) => {
                                self.normal_action(NormalActions::PrevFile);
                            }
                            //PREVIOUS DIRECTORY
                            (KeyModifiers::NONE, KeyCode::Char('h')) => {
                                self.normal_action(NormalActions::PrevDir);
                            }
                            //NEXT DIRECTORY
                            (_, KeyCode::Char('l')) => {
                                self.normal_action(NormalActions::NextDir);
                            }
                            //ADD FILE
                            (_, KeyCode::Char('a')) => {
                                self.normal_action(NormalActions::AddFile);
                            }
                            //DELETE
                            (KeyModifiers::SHIFT, KeyCode::Char('D')) => {
                                self.normal_action(NormalActions::DelFile);
                            }
                            (_, KeyCode::Char(' ')) => {
                                self.normal_action(NormalActions::Select);
                            }
                            (KeyModifiers::CONTROL, KeyCode::Char('h')) => {
                                self.normal_action(NormalActions::ToggleHidden);
                            }
                            (_, _) => {}
                        }
                    }
                    AppMode::Prompt => {
                        match self.graphics.widget {
                            widget_type::AddFile => {
                                match (key.modifiers, key.code) {
                                    (_, KeyCode::Esc) => {
                                        self.logic.app_mode = AppMode::Normal;
                                        self.graphics.widget = widget_type::None;
                                        self.input.clear();
                                    }
                                    (_, KeyCode::Char(c)) => {
                                        self.input.push(c);
                                    }
                                    (_, KeyCode::Enter) => {
                                        self.logic.app_mode = AppMode::Normal;
                                        self.graphics.widget = widget_type::None;
                                        if self.input.is_empty() {
                                            return Ok(());
                                        }
                                        File::create(
                                            self.logic.dir.join(self.input.clone())
                                        ).unwrap();
                                        self.input.clear();
                                        self.logic.list_items = get_file_names(
                                            &self.logic.dir,
                                            self.logic.hidden_files
                                        );
                                    }
                                    (_, _) => {}
                                }
                            }
                            widget_type::DeletePrompt => {
                                if self.logic.ask_delete {
                                    if self.logic.selected_files.is_empty() {
                                        remove_file(self.get_path_selected())?;
                                        self.logic.list_items = get_file_names(
                                            &self.logic.dir,
                                            self.logic.hidden_files
                                        );
                                    } else {
                                        for x in self.logic.selected_files.iter() {
                                            remove_file(self.logic.dir.join(PathBuf::from(x)))?;
                                        }
                                    }
                                }

                                match (key.modifiers, key.code) {
                                    (_, KeyCode::Esc) => {
                                        self.logic.app_mode = AppMode::Normal;
                                        self.graphics.widget = widget_type::None;
                                    }
                                    (_, KeyCode::Enter) => {
                                        if
                                            self.logic.prompt_list_state.selected() == Some(0) ||
                                            self.logic.prompt_list_state.selected() == Some(2)
                                        {
                                            if self.logic.selected_files.is_empty() {
                                                if self.get_path_selected().is_dir() {
                                                    remove_dir_all(self.get_path_selected())?;
                                                } else {
                                                    remove_file(self.get_path_selected())?;
                                                }
                                                self.logic.list_items = get_file_names(
                                                    &self.logic.dir,
                                                    self.logic.hidden_files
                                                );
                                            } else {
                                                for x in self.logic.selected_files.iter() {
                                                    if
                                                        self.logic.dir
                                                            .join(PathBuf::from(x))
                                                            .is_dir()
                                                    {
                                                        remove_dir_all(
                                                            self.logic.dir.join(PathBuf::from(x))
                                                        )?;
                                                    } else {
                                                        remove_file(
                                                            self.logic.dir.join(PathBuf::from(x))
                                                        )?;
                                                    }
                                                }
                                            }
                                        }
                                        if self.logic.prompt_list_state.selected() == Some(2) {
                                            self.logic.ask_delete = true;
                                        }
                                        self.logic.selected_files.clear();
                                        self.logic.list_items = get_file_names(
                                            &self.logic.dir,
                                            self.logic.hidden_files
                                        );
                                        self.logic.list_state.select_first();
                                        self.logic.app_mode = AppMode::Normal;
                                        self.graphics.widget = widget_type::None;
                                    }
                                    (_, KeyCode::Backspace) => {
                                        self.input.pop();
                                    }
                                    (_, KeyCode::Char('j')) => {
                                        self.logic.prompt_list_state.select_next();
                                    }
                                    (_, KeyCode::Char('k')) => {
                                        self.logic.prompt_list_state.select_previous();
                                    }
                                    (_, KeyCode::Char(c)) => {
                                        self.input.push(c);
                                    }
                                    (_, _) => {}
                                }
                            }
                            widget_type::None => {}
                        }
                    }
                }
            }
        }
    }
    fn render(&mut self, frame: &mut Frame) {
        let layout: [Rect; 2] = Layout::default()
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .direction(Direction::Horizontal)
            .areas(Rect {
                x: frame.area().x,
                y: frame.area().y + 1,
                width: frame.area().width,
                height: frame.area().height - 4,
            });

        let files_area: [Rect; 2] = Layout::default()
            .constraints([Constraint::Fill(1), Constraint::Fill(3)])
            .direction(Direction::Horizontal)
            .areas(layout[0]);

        Paragraph::new(
            format!(
                "{}/{}",
                self.logic.dir.display(),
                self.logic.file.clone().unwrap_or(String::from(""))
            )
        ).render(frame.area(), frame.buffer_mut());

        let list = List::new(
            self.logic.list_items.iter().map(|x| {
                let mut name = x.clone();
                if self.logic.dir.join(PathBuf::from(format!("{}", name))).is_dir() {
                    name = format!("📁{}", name);
                }
                if self.logic.selected_files.contains(x) {
                    ListItem::new(format!(" {}", name)).style(self.graphics.styles.highlight_style)
                } else {
                    ListItem::new(format!("{}", name)).style(self.graphics.styles.list_style)
                }
            })
        )
            .highlight_symbol(">")
            .highlight_style(Style::default().on_blue());
        frame.render_stateful_widget(list, files_area[1], &mut self.logic.list_state);

        //CREATE LEFT LIST
        let parent_list = List::default()
            .items(get_file_names(&self.logic.dir.parent().unwrap().to_path_buf(), false))
            .highlight_style(Style::default().on_gray())
            .highlight_symbol(">");

        let parent_index = Self::get_file_index(
            &get_file_names(&self.logic.dir.parent().unwrap().to_path_buf(), false),
            self.logic.dir.file_name().unwrap().to_str().unwrap()
        ).unwrap();
        self.logic.parent_list_state.select(Some(parent_index));

        //RENDER LEFT LIST
        frame.render_stateful_widget(parent_list, files_area[0], &mut self.logic.parent_list_state);

        // let binding = self.logic.contents
        //     .get(&(self.logic.list_state.selected().unwrap() as u16))
        //     .unwrap()
        //     .clone();
        // let lines: Vec<Line> = binding.lines().map(Line::from).collect();

        if self.logic.contents.is_some() {
            Paragraph::new(self.logic.contents.clone().unwrap())
                .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded))
                .render(layout[1], frame.buffer_mut());
        }

        match self.graphics.widget {
            widget_type::AddFile => {
                let add_file_rect = Rect {
                    x: 0,
                    y: frame.area().height - 3,
                    width: files_area[1].width,
                    height: 3,
                };
                Paragraph::new(self.input.as_str())
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_type(BorderType::Rounded)
                            .title("Add file----[Enter/Esc]")
                    )
                    .render(add_file_rect, frame.buffer_mut());
            }
            widget_type::DeletePrompt => {
                if !self.logic.ask_delete {
                    let msg = if self.logic.selected_files.is_empty() {
                        "Are you sure you want to delete this file?"
                    } else {
                        "Are you sure you want to delete these files?"
                    };
                    let border_area = Rect {
                        x: 0,
                        y: frame.area().height - 3,
                        width: files_area[1].width,
                        height: 3,
                    };
                    let delete_rect = Rect {
                        x: 1,
                        y: frame.area().height - 2,
                        width: files_area[1].width - 2,
                        height: 1,
                    };
                    Block::new()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(msg)
                        .render(border_area, frame.buffer_mut());
                    let rows = [Row::new(vec!["Yes", "No", "Don't ask again"])];
                    let widths = [
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                        Constraint::Ratio(1, 3),
                    ];
                    let table = Table::new(rows, widths).cell_highlight_style(
                        self.graphics.styles.highlight_style
                    );
                    let mut table_state = TableState::default();
                    table_state.select_first();
                    frame.render_stateful_widget(table, delete_rect, &mut table_state);
                }

                //     let delete_rect = centered_rect_parameter(msg.len() as u16, 5, frame.area());

                //     let border_area = Rect::new(
                //         delete_rect.x - 1,
                //         delete_rect.y - 1,
                //         delete_rect.width + 2,
                //         delete_rect.height + 2
                //     );

                //     let layout = Layout::vertical(
                //         vec![Constraint::Length(2), Constraint::Length(3)]
                //     ).split(delete_rect);

                //     Block::new()
                //         .borders(Borders::ALL)
                //         .border_type(BorderType::Rounded)
                //         .render(border_area, frame.buffer_mut());

                //     Paragraph::new(format!("{}", msg.red()))
                //         .block(Block::new().borders(Borders::BOTTOM).border_type(BorderType::Thick))
                //         .alignment(Alignment::Center)
                //         .render(layout[0], frame.buffer_mut());
                //     let list = List::new(vec!["Yes", "No", "Don't ask again"])
                //         .highlight_symbol(">")
                //         .highlight_style(self.graphics.styles.highlight_style);
                //     frame.render_stateful_widget(
                //         list,
                //         layout[1],
                //         &mut self.logic.prompt_list_state
                //     );
                // }
            }
            widget_type::None => {}
        }
    }

    fn new() -> App {
        App {
            logic: Logic {
                dir: current_dir().unwrap(),
                app_mode: AppMode::Normal,
                selected_files: Vec::new(),
                list_items: get_file_names(&current_dir().unwrap(), false),
                parent_list: get_file_names(
                    &current_dir().unwrap().parent().unwrap().to_path_buf(),
                    false
                ),
                list_state: ListState::default(),
                parent_list_state: ListState::default(),
                prompt_list_state: ListState::default().with_selected(Some(0)),
                index: Some(0),
                ask_delete: false,
                hidden_files: false,
                contents: None,
                file: None,
            },
            graphics: Graphics {
                widget: widget_type::None,
                styles: Styles {
                    list_style: Style::default(),
                    highlight_style: Style::default().on_yellow(),
                },
            },
            input: String::new(),
        }
    }
}

//CODE FOR POPUP WITH TEXT
// let msg = "Are you sure you want to delete this file?";
// let delete_rect = centered_rect_parameter(msg.len() as u16, 4, frame.area());
// let border_area = Rect::new(
//     delete_rect.x - 1,
//     delete_rect.y - 1,
//     delete_rect.width + 2,
//     delete_rect.height + 2
// );
// let layout = Layout::vertical(
//     vec![Constraint::Length(3), Constraint::Length(1)]
// ).split(delete_rect);
// Block::new()
//     .padding(Padding::uniform(5))
//     .borders(Borders::ALL)
//     .border_type(BorderType::Rounded)
//     .render(border_area, frame.buffer_mut());
// Paragraph::new(format!("{}\n{}", msg.red(), "[Y/N]".yellow()))
//     .block(Block::new().borders(Borders::BOTTOM).border_type(BorderType::Thick))
//     .alignment(Alignment::Center)
//     .render(layout[0], frame.buffer_mut());
// Paragraph::new(self.input.as_str())
//     .alignment(Alignment::Center)
//     .block(Block::new())
//     .render(layout[1], frame.buffer_mut());
