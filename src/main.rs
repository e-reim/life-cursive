#![windows_subsystem = "windows"]
#![allow(clippy::many_single_char_names)]

extern crate cursive;

use cursive::align::HAlign;
use cursive::{
    direction::Direction,
    event::{Event, EventResult, Key, MouseButton, MouseEvent},
    menu,
    theme::{Color, ColorStyle},
    traits::*,
    view::CannotFocus,
    view::SizeConstraint,
    views::{
        Canvas, Dialog, EditView, LinearLayout, OnLayoutView, ResizedView, SelectView, TextView,
    },  
    Cursive, Printer,
};

use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::fmt::write;
use std::rc::Rc;
//use std::result::Result;

use db::Result as Dbres;
//use std::sync::atomic::{AtomicUsize, Ordering};

mod db;
use db::Storage;

fn _get_field_style(cursor: bool) -> ColorStyle {
    if cursor {
        ColorStyle::new(Color::Rgb(32, 32, 244), Color::Rgb(144, 144, 255))
    } else {
        ColorStyle::new(Color::Rgb(144, 144, 255), Color::Rgb(32, 32, 244))
    }
}

struct Gamedata {
    storage: Storage,
    field: HashSet<(i32, i32)>,
    search: Box<Vec<(i32, i32)>>,
    start_x: i32,
    start_y: i32,
    edit_x: i32,
    edit_y: i32,
    edit_mode: bool,
    do_center: bool,
    do_search: bool,
}

impl Gamedata {
    pub fn new() -> Gamedata {
        Gamedata {
            storage: Storage::new("lf.db"),
            field: HashSet::new(),
            search: Box::new(vec![]),
            start_x: 0,
            start_y: 0,
            edit_x: 0,
            edit_y: 0,
            edit_mode: true,
            do_center: false,
            do_search: false,
        }
    }

    pub fn toggle_cell(&mut self, x: i32, y: i32) {
        if self.edit_mode {
            self.edit_x = x;
            self.edit_y = y;
        }
        if self.field.contains(&(x, y)) {
            self.field.remove(&(x, y));
        } else {
            self.field.insert((x, y));
        }
    }

    pub fn save(&mut self, name: &str) -> Dbres<i64> {
        self.storage.save(name, &self.field)
    }

    pub fn records(&self) -> Dbres<Vec<(i64, String)>> {
        self.storage.get_records()
    }

    pub fn update(&mut self) {
        self.search.clear();
        let mut nc: HashMap<(i32, i32), i32> = HashMap::new();
        for (cx, cy) in self.field.iter() {
            for dx in -1..2 {
                for dy in -1..2 {
                    if dx != 0 || dy != 0 {
                        let (nx, ny) = (cx + dx, cy + dy);
                        match nc.get_mut(&(nx, ny)) {
                            Some(q) => {
                                *q = *q + 1;
                            }
                            None => {
                                nc.insert((nx, ny), 1);
                            }
                        }
                    }
                }
            }
        }
        let born: Vec<(i32, i32)> = nc
            .keys()
            .filter(|&c| nc[c] == 3 && !self.field.contains(c))
            .cloned()
            .collect();
        let died: Vec<(i32, i32)> = self.field
            .iter()
            .filter(|&c| {
                let n = match nc.get(c) {
                    Some(v) => *v,
                    None => 0,
                };
                (n < 2 || n > 3) && self.field.contains(c)
            })
            .cloned()
            .collect();
        for d in died {
            self.field.remove(&d);
        }
        for b in born {
            self.field.insert(b);
        }
    }
}

struct FieldView {
    gamedata: Rc<RefCell<Gamedata>>,
}

impl FieldView {
    pub fn new(gd: Rc<RefCell<Gamedata>>) -> Self {
        FieldView {
            gamedata: Rc::clone(&gd),
        }
    }
}

impl cursive::view::View for FieldView {
    fn take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }

    fn on_event(&mut self, ev: Event) -> EventResult {
        let mut gdata = (*self.gamedata).borrow_mut();
        if gdata.edit_mode {
            match ev {
                Event::Key(k) => {
                    //let mut gdata = (*self.gamedata).borrow_mut();
                    match k {
                        Key::Left => {
                            gdata.edit_x -= 1;
                        }
                        Key::Right => {
                            gdata.edit_x += 1;
                        }
                        Key::Up => {
                            gdata.edit_y -= 1;
                        }
                        Key::Down => {
                            gdata.edit_y += 1;
                        }
                        Key::F4 => {
                            gdata.edit_mode = false;
                        }
                        Key::F5 => {
                            gdata.do_search = true;
                        },
                        _ => (),
                    }
                }
                Event::Char(' ') => {
                    let (x, y) = (gdata.edit_x, gdata.edit_y);
                    gdata.toggle_cell(x, y);
                }
                Event::Mouse {
                    offset,
                    position,
                    event: MouseEvent::Press(btn),
                } => {
                    let x = ((position.x - offset.x) as i32) / 2 + gdata.start_x;
                    let y = ((position.y - offset.y) as i32) + gdata.start_y;
                    match btn {
                        MouseButton::Right => {
                            gdata.edit_x = x;
                            gdata.edit_y = y;
                            gdata.do_center = true;
                        }
                        MouseButton::Left => {
                            gdata.toggle_cell(x, y);
                        }
                        _ => (),
                    };
                }
                _ => (),
            }
        } else {
            // Play mode
            match ev {
                Event::Key(k) => {
                    //let mut gdata = (*self.gamedata).borrow_mut();
                    match k {
                        Key::Left => {
                            gdata.start_x -= 1;
                        }
                        Key::Right => {
                            gdata.start_x += 1;
                        }
                        Key::Up => {
                            gdata.start_y -= 1;
                        }
                        Key::Down => {
                            gdata.start_y += 1;
                        }
                        Key::F4 => {
                            gdata.edit_mode = true;
                        },
                        Key::F5 => {
                            gdata.do_search = true;
                        },
                        _ => (),
                    }
                }
                Event::Char(' ') => {
                    gdata.update();
                }
                Event::Mouse {
                    offset,
                    position,
                    event: MouseEvent::Press(btn),
                } => {
                    let x = ((position.x - offset.x) as i32) / 2 + gdata.start_x;
                    let y = ((position.y - offset.y) as i32) + gdata.start_y;
                    match btn {
                        MouseButton::Right => {
                            gdata.edit_x = x;
                            gdata.edit_y = y;
                            gdata.do_center = true;
                        }
                        MouseButton::Left => {
                            gdata.toggle_cell(x, y);
                        }
                        _ => (),
                    };
                }
                _ => (),
            }
            gdata.edit_x = gdata.start_x;
            gdata.edit_y = gdata.start_y;
        }
        EventResult::Ignored
    }

    fn draw(&self, p: &Printer) {
        let mut gdata = (*self.gamedata).borrow_mut();

        let x_max = p.size.x as i32;
        let y_max = p.size.y as i32;
        let style = _get_field_style(false);
        let cursor_style = _get_field_style(true);

        let x_f = ((x_max + 1) / 2) as i32;

        let visible = |x: i32, y: i32, sx: i32, sy: i32| {
            x >= sx && y >= sy && x < sx+x_f && y < sy+y_max
        };

        if gdata.do_search {
            if gdata.search.is_empty() {
                gdata.search = Box::new(gdata.field.iter().cloned().collect::<Vec<(i32, i32)>>());
            }
            if !gdata.search.is_empty() {
                let (x, y) = gdata.search[0];
                gdata.edit_x = x;
                gdata.edit_y = y;
                gdata.start_x = gdata.edit_x - x_f / 2;
                gdata.start_y = gdata.edit_y - y_max / 2;
                    gdata.search = Box::new(gdata.search.iter().filter(|&p| {
                    let (px, py) = *p;
                    !visible(px, py, gdata.start_x, gdata.start_y)
                }).cloned().collect::<Vec<(i32, i32)>>());
            }
        }
        gdata.do_search = false;

        if gdata.do_center {
            gdata.start_x = gdata.edit_x - x_f / 2;
            gdata.start_y = gdata.edit_y - y_max / 2;
        }
        gdata.do_center = false;

        if gdata.edit_x < gdata.start_x {
            gdata.start_x = gdata.edit_x;
        }
        if gdata.edit_y < gdata.start_y {
            gdata.start_y = gdata.edit_y;
        }
        if gdata.edit_x >= gdata.start_x + x_f {
            gdata.start_x = gdata.edit_x - x_f + 1;
        }
        if gdata.edit_y >= gdata.start_y + y_max {
            gdata.start_y = gdata.edit_y - y_max + 1;
        }

        for y in gdata.start_y..y_max + gdata.start_y {
            let mut s = String::new();
            for x in gdata.start_x..x_f + gdata.start_x {
                if gdata.field.contains(&(x, y)) {
                    write(&mut s, format_args!("@ ")).unwrap();
                } else {
                    write(&mut s, format_args!(". ")).unwrap();
                }
            }
            p.with_color(style, |printer| {
                printer.print((0, y - gdata.start_y), &s);
            });
            // Drawing cursor if it is in the current line (edit mode only)
            if gdata.edit_mode && y == gdata.edit_y {
                let cpos = (gdata.edit_x - gdata.start_x) * 2;
                if cpos >= 0 && cpos <= x_max as i32 {
                    p.with_color(cursor_style, |printer| {
                        printer.print(
                            (cpos, y - gdata.start_y),
                            if gdata.field.contains(&(gdata.edit_x, y as i32)) {
                                "@"
                            } else {
                                "."
                            },
                        )
                    });
                }
            }
        }
    }
}

fn _leave_dialog(siv: &mut Cursive) {
    {
        let mut gd = (*siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap()).borrow_mut();
        gd.start_y += 1;
    }
    siv.set_autohide_menu(false);
    siv.clear_global_callbacks(Key::Esc);
    siv.add_global_callback(Key::Esc, |s| s.select_menubar());
    siv.add_global_callback(Key::F1, |s| _help(s));
}

fn _enter_dialog(siv: &mut Cursive) {
    {
        let mut gd = (*siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap()).borrow_mut();
        gd.start_y -= 1;
    }
    siv.set_autohide_menu(true);
    siv.clear_global_callbacks(Key::Esc);
    siv.clear_global_callbacks(Key::F1);
}

const HELP_TEXT: &str =
"ALL MODES:
  <F1> displays this help
  <F4> toggles between the edit and playback modes
  <F5> to search for live cells (cycles through the visible parts)
  Right-Click to center

EDIT MODE:
  Arrows to position the cursor for keyboard editing
  Left-click or space to toggle cell
  
PLAYBACK MODE:
  <SPACE> to step forward
  Arrows to shift";

fn _help(siv: &mut Cursive) {
    let dlg = Dialog::new()
        .title("Help")
        .content(
            TextView::new(HELP_TEXT)
                .min_width(60)
                .min_height(20),
        )
        .button("Ok", |s| {
            _leave_dialog(s);
            s.pop_layer();
        });
    _enter_dialog(siv);
    siv.add_global_callback(Key::Esc, |s| {
        _leave_dialog(s);
        s.pop_layer();
    });
    siv.add_layer(dlg);
}

fn _load(siv: &mut Cursive) {
    let mut select: SelectView = SelectView::new()
        // Center the text horizontally
        .h_align(HAlign::Center)
        // Use keyboard to jump to the pressed letters
        .autojump();
    {
        let gd = siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap();

        let recs = (*gd).borrow().records().unwrap().clone();
        for (_, n) in recs {
            select.add_item_str(n);
        }
    }
    select.set_on_submit(|siv, name: &str| {
        {
            let mut gd = (*siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap()).borrow_mut();
            match gd.storage.load(name) {
                Ok(v) => {
                    gd.field.clear();
                    for (x, y) in v {
                        gd.field.insert((x, y));
                    }
                }
                Err(_) => {}
            }
        }
        siv.pop_layer();
        _leave_dialog(siv);
    });

    siv.add_layer(
        Dialog::around(select.scrollable())
            .title("Select a position to load")
            .button("Cancel", |s| {
                s.pop_layer();
                _leave_dialog(s);
            }),
    );
    _enter_dialog(siv);
}

fn _delete(siv: &mut Cursive) {
    let mut select: SelectView = SelectView::new()
        // Center the text horizontally
        .h_align(HAlign::Center)
        // Use keyboard to jump to the pressed letters
        .autojump();
    {
        let gd = siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap();

        let recs = (*gd).borrow().records().unwrap().clone();
        for (_, n) in recs {
            select.add_item_str(n);
        }
    }
    select.set_on_submit(|siv, name: &str| {
        {
            let mut gd = (*siv.user_data::<Rc<RefCell<Gamedata>>>().unwrap()).borrow_mut();
            gd.storage.delete(name).unwrap();
        }
        siv.pop_layer();
        _leave_dialog(siv);
    });

    siv.add_layer(
        Dialog::around(select.scrollable())
            .title("Select a position to DELETE")
            .button("Cancel", |s| {
                s.pop_layer();
                _leave_dialog(s);
            }),
    );
    _enter_dialog(siv);
}

fn _save(siv: &mut Cursive) {
    let dlg = Dialog::new()
        .title("Find")
        .content(
            EditView::new()
                //.on_submit(find)
                .with_name("save_name")
                .min_width(10),
        )
        .button("Ok", |siv| {
            let sv_res: Dbres<i64>;
            let text = siv
                .call_on_name("save_name", |view: &mut EditView| view.get_content())
                .unwrap();
            {
                let mut gd = siv
                    .user_data::<Rc<RefCell<Gamedata>>>()
                    .unwrap()
                    .borrow_mut();
                sv_res = gd.save(&text);
            }
            match sv_res {
                Ok(_) => {
                    siv.pop_layer();
                    _leave_dialog(siv);
                }
                _ => {
                    siv.add_layer(
                        Dialog::around(TextView::new(format!("Is the name '{}' unique?", &text)))
                            .title("Cannot save")
                            .dismiss_button("Ok"),
                    );
                }
            }
        })
        .button("Cancel", |siv| {
            siv.pop_layer();
            _leave_dialog(siv);
        });
    _enter_dialog(siv);
    siv.add_layer(dlg.title("Enter a name for the position"));
}

fn _draw_status(gd: &Rc<RefCell<Gamedata>>, p: &Printer) {
    let x_max = p.size.x;

    let gdata = (*gd).borrow();
    let mut s: String = String::new();
    write(
        &mut s,
        format_args!(
            "{}; <F1>: help, <F4>: edit/play, <ESC>: menu; S=({},{}); E=({},{})",
            if gdata.edit_mode {"<= EDIT =>"} else {"<= PLAY =>"}, gdata.start_x, gdata.start_y, gdata.edit_x, gdata.edit_y
        ),
    )
    .unwrap();
    while s.len() <= x_max {
        write(&mut s, format_args!("       ")).unwrap();
    }
    p.print((0, 0), &s);
}

pub fn run() {
    let mut siv = cursive::pancurses();
    let gdata: Rc<RefCell<Gamedata>> = Rc::new(RefCell::new(Gamedata::new()));
    siv.set_user_data(Rc::clone(&gdata));

    siv.set_autohide_menu(false);
    siv.menubar()
        .add_subtree(
            "Archive",
            menu::Tree::new()
                .leaf("Load", |s| {
                    _load(s);
                })
                .delimiter()
                .leaf("Save", |s| {
                    _save(s);
                })
                .leaf("Delete", |s| {
                    _delete(s);
                }),
        )
        .add_delimiter()
        .add_leaf("Quit", |s| s.quit());
    siv.add_fullscreen_layer(OnLayoutView::wrap(
        LinearLayout::vertical()
            .child(
                ResizedView::with_full_screen(FieldView::new(Rc::clone(&gdata)))
                    .with_name("field_view"),
            )
            .child(ResizedView::new(
                SizeConstraint::Full,
                SizeConstraint::Fixed(1),
                Canvas::new(Rc::clone(&gdata))
                    .with_draw(_draw_status)
                    .with_name("status_view"),
            )),
    ));
    _leave_dialog(&mut siv);
    siv.set_autorefresh(true);
    siv.run();
}

fn main() {
    run();
}
