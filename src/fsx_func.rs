use std::{
    collections::{ HashMap, HashSet },
    fs::DirEntry,
    path::{ Path, PathBuf },
    process::Command,
};

use color_eyre::eyre::Ok;
use ratatui::{ layout::{ Constraint, Direction, Layout, Rect }, style::Style, widgets::ListState };
pub struct App {
    pub logic: Logic,
    pub graphics: Graphics,
    pub input: String,
}

pub struct Logic {
    pub dir: PathBuf,
    pub app_mode: AppMode,
    pub selected_files: Vec<String>,
    pub list_items: Vec<String>,
    pub parent_list: Vec<String>,
    pub list_state: ListState,
    pub parent_list_state: ListState,
    pub prompt_list_state: ListState,
    pub ask_delete: bool,
    pub index: Option<usize>,
    pub hidden_files: bool,
    pub contents: Option<String>,
    pub file: Option<String>,
}
pub struct Graphics {
    pub widget: widget_type,
    pub styles: Styles,
}
pub enum widget_type {
    None,
    AddFile,
    DeletePrompt,
}
pub struct Styles {
    pub list_style: Style,
    pub highlight_style: Style,
}

pub enum NormalActions {
    NextFile,
    PrevFile,
    NextDir,
    PrevDir,
    AddFile,
    DelFile,
    Select,
    ToggleHidden,
}

pub enum AppMode {
    Normal,
    Prompt,
}
impl App {
    pub fn get_file_index(files: &Vec<String>, file_name: &str) -> Option<usize> {
        if let Some(index) = files.iter().position(|x| x == file_name) { Some(index) } else { None }
    }
    pub fn get_file_selected(&self) -> Option<String> {
        let dir = get_dir(&self.logic.dir, self.logic.hidden_files);
        let mut dir_names = dir
            .iter()
            .map(|x| x.file_name().to_string_lossy().to_string())
            .collect::<Vec<String>>();
        dir_names.sort();
        if let Some(index) = self.logic.index {
            Some(dir_names[index].clone())
        } else {
            None
        }
    }
    pub fn get_path_selected(&self) -> PathBuf {
        self.logic.dir.join(PathBuf::from(Self::get_file_selected(&self).unwrap()))
    }
    pub fn get_junior_list(&self) -> Vec<String> {
        let mut dir = self.logic.dir.clone();
        if let Some(index) = self.logic.list_state.selected() {
            dir = dir.join(PathBuf::from(self.logic.list_items[index].clone()));
        }
        let mut dir_items = get_dir(&dir, self.logic.hidden_files)
            .iter()
            .map(|x| x.file_name().to_string_lossy().to_string())
            .collect::<Vec<String>>();
        dir_items.sort();
        return dir_items;
    }
    pub fn normal_action(&mut self, action: NormalActions) {
        match action {
            NormalActions::NextFile => {
                if self.logic.list_state.selected() == Some(self.logic.list_items.len() - 1) {
                    self.logic.list_state.select_first();
                } else {
                    self.logic.list_state.select_next();
                }
                self.logic.contents = Self::build_contents(
                    &self.logic.dir.join(
                        PathBuf::from(
                            self.logic.list_items[self.logic.list_state.selected().unwrap()].clone()
                        )
                    )
                );
                self.update_selected_file();
            }
            NormalActions::PrevFile => {
                if self.logic.list_state.selected() == Some(0) {
                    self.logic.list_state.select_last();
                    //For some reason index is out of bounds when going to last item
                    self.logic.contents = Self::build_contents(
                        &self.logic.dir.join(
                            PathBuf::from(
                                self.logic.list_items[self.logic.list_items.len() - 1].clone()
                            )
                        )
                    );
                } else {
                    self.logic.list_state.select_previous();
                    self.logic.contents = Self::build_contents(
                        &self.logic.dir.join(
                            PathBuf::from(
                                self.logic.list_items[
                                    self.logic.list_state.selected().unwrap()
                                ].clone()
                            )
                        )
                    );
                }
                self.update_selected_file();
            }
            NormalActions::NextDir => {
                if self.get_path_selected().is_dir() {
                    self.logic.parent_list = self.logic.list_items.clone();
                    self.logic.list_items = self.get_junior_list();
                    self.logic.list_state.select_first();
                    self.logic.dir = self.get_path_selected();
                    self.update_selected_file();
                }
            }
            NormalActions::PrevDir => {
                self.logic.dir = self.logic.dir.parent().unwrap().to_path_buf();
                self.logic.list_items = self.logic.parent_list.clone();
                self.logic.list_state.select(self.logic.parent_list_state.selected());
                let parent_dir = &self.logic.dir.parent().unwrap().to_path_buf();
                self.logic.parent_list = get_file_names(parent_dir, self.logic.hidden_files);
                self.update_selected_file();
            }
            NormalActions::AddFile => {
                self.logic.app_mode = AppMode::Prompt;
                self.graphics.widget = widget_type::AddFile;
            }
            NormalActions::DelFile => {
                self.logic.app_mode = AppMode::Prompt;
                self.graphics.widget = widget_type::DeletePrompt;
            }
            NormalActions::Select => {
                if self.logic.selected_files.contains(&self.get_file_selected().unwrap()) {
                    self.logic.selected_files.remove(
                        self.logic.selected_files
                            .iter()
                            .position(|x| x == &self.get_file_selected().unwrap())
                            .unwrap()
                    );
                } else {
                    self.logic.selected_files.push(self.get_file_selected().unwrap());
                }
                if self.logic.list_state.selected() == Some(self.logic.list_items.len() - 1) {
                    self.logic.list_state.select_first();
                } else {
                    self.logic.list_state.select_next();
                }
            }
            NormalActions::ToggleHidden => {
                self.logic.hidden_files = !self.logic.hidden_files;
                self.logic.selected_files.clear();
                self.logic.list_items = get_file_names(&self.logic.dir, self.logic.hidden_files);
            }
        }
    }
    pub fn build_contents(path: &PathBuf) -> Option<String> {
        if path.is_file() && path.metadata().unwrap().len() < 1024 * 1024 {
            Some(cat(path))
        } else {
            None
        }
    }
    pub fn update_selected_file(&mut self) {
        if self.get_file_selected().is_some() {
            self.logic.file = self.get_file_selected();
        } else {
            self.logic.file = None;
        }
    }
}

pub fn get_dir(dir: &PathBuf, hidden_files: bool) -> Vec<DirEntry> {
    let mut dir_entries = std::fs
        ::read_dir(dir)
        .unwrap()
        .map(|x| x.unwrap())
        .collect::<Vec<DirEntry>>();
    if !hidden_files {
        dir_entries.retain(|x| !x.file_name().to_string_lossy().to_string().starts_with('.'));
    }
    dir_entries
}
pub fn get_file_names(dir: &PathBuf, hidden_files: bool) -> Vec<String> {
    let mut direntry = get_dir(dir, hidden_files)
        .iter()
        .map(|x| x.file_name().to_string_lossy().to_string())
        .collect::<Vec<String>>();
    direntry.sort();
    direntry
}
//Explanation at https://ratatui.rs/tutorials/json-editor/ui/
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn centered_rect_parameter(width: u16, height: u16, r: Rect) -> Rect {
    // Cut the given rectangle into three vertical pieces
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Fill(1), Constraint::Length(height), Constraint::Fill(1)])
        .split(r);

    // Then cut the middle vertical piece into three width-wise pieces
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Fill(1), Constraint::Length(width), Constraint::Fill(1)])
        .split(popup_layout[1])[1] // Return the middle chunk
}

pub fn get_file_names_format(dir: &PathBuf, hidden_files: bool) -> Vec<String> {
    let mut direntry = get_dir(dir, hidden_files);
    let mut file_names = Vec::new();
    for x in direntry.iter_mut() {
        if x.path().is_dir() {
            file_names.push(format!("📁{}", x.file_name().to_string_lossy().to_string()));
        } else {
            file_names.push(x.file_name().to_string_lossy().to_string());
        }
    }
    file_names.sort();
    file_names
}

pub fn cat(path: &PathBuf) -> String {
    let output = Command::new("cat")
        .arg(format!("{}", path.display()))
        .output()
        .expect("Failed to execute cat");
    output.stdout
        .iter()
        .map(|x| *x as char)
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cat() {
        let output = cat(&PathBuf::from("./test/example.txt"));
        assert_eq!(output, "Hello World!\n");
    }
}
