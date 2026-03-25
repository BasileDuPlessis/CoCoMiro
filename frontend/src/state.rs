use hello_world_shared::{Position, Size, StickyNote};
use std::rc::Rc;
use yew::prelude::*;

#[derive(Clone, PartialEq)]
pub struct ViewState {
    pub zoom: f64,
    pub pan_x: f64,
    pub pan_y: f64,
    pub is_dragging: bool,
    pub last_mouse_pos: Option<(f64, f64)>,
}

#[derive(Clone, PartialEq)]
pub struct ToolbarState {
    pub x: f64,
    pub y: f64,
    pub is_dragging: bool,
    pub drag_offset: Option<(f64, f64)>,
}

#[derive(Clone, PartialEq)]
pub struct StickyNotesState {
    pub notes: Vec<StickyNote>,
    pub editing_note_id: Option<String>,
    pub editing_content: Option<String>,
    pub selected_note_id: Option<String>,
}

#[derive(Clone, PartialEq)]
pub struct AppState {
    pub view: ViewState,
    pub toolbar: ToolbarState,
    pub sticky_notes: StickyNotesState,
}

#[derive(Clone, Debug)]
pub enum ViewAction {
    ZoomIn,
    ZoomOut,
    ZoomBy(f64),
    StartDrag(f64, f64),
    UpdateDrag(f64, f64),
    EndDrag,
}

impl Reducible for ViewState {
    type Action = ViewAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ViewAction::ZoomIn => Rc::new(ViewState {
                zoom: (self.zoom * 1.2).min(5.0),
                ..*self
            }),
            ViewAction::ZoomOut => Rc::new(ViewState {
                zoom: (self.zoom / 1.2).max(0.1),
                ..*self
            }),
            ViewAction::ZoomBy(factor) => Rc::new(ViewState {
                zoom: (self.zoom * factor).clamp(0.1, 5.0),
                ..*self
            }),
            ViewAction::StartDrag(x, y) => Rc::new(ViewState {
                is_dragging: true,
                last_mouse_pos: Some((x, y)),
                ..*self
            }),
            ViewAction::UpdateDrag(x, y) => {
                if self.is_dragging {
                    if let Some((last_x, last_y)) = self.last_mouse_pos {
                        let delta_x = x - last_x;
                        let delta_y = y - last_y;
                        Rc::new(ViewState {
                            pan_x: self.pan_x + delta_x,
                            pan_y: self.pan_y + delta_y,
                            last_mouse_pos: Some((x, y)),
                            ..*self
                        })
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
            ViewAction::EndDrag => Rc::new(ViewState {
                is_dragging: false,
                last_mouse_pos: None,
                ..*self
            }),
        }
    }
}

#[allow(clippy::enum_variant_names)]
#[derive(Clone, Debug)]
pub enum ToolbarAction {
    StartDrag(f64, f64),
    UpdateDrag(f64, f64),
    EndDrag,
}

impl Reducible for ToolbarState {
    type Action = ToolbarAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            ToolbarAction::StartDrag(offset_x, offset_y) => Rc::new(ToolbarState {
                is_dragging: true,
                drag_offset: Some((offset_x, offset_y)),
                ..*self
            }),
            ToolbarAction::UpdateDrag(client_x, client_y) => {
                if self.is_dragging {
                    if let Some((offset_x, offset_y)) = self.drag_offset {
                        Rc::new(ToolbarState {
                            x: client_x - offset_x,
                            y: client_y - offset_y,
                            ..*self
                        })
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
            ToolbarAction::EndDrag => Rc::new(ToolbarState {
                is_dragging: false,
                drag_offset: None,
                ..*self
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum StickyNotesAction {
    CreateNote(Position, Size),
    StartEdit(String),
    UpdateContent(String),
    SaveEdit,
    CancelEdit,
    UpdatePosition(String, Position),
    SelectNote(String),
    DeselectNote,
}

impl Reducible for StickyNotesState {
    type Action = StickyNotesAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            StickyNotesAction::CreateNote(position, size) => {
                let new_note = StickyNote {
                    id: format!("note-{}", js_sys::Date::now()),
                    position,
                    content: "New sticky note".to_string(),
                    size,
                };
                Rc::new(StickyNotesState {
                    notes: {
                        let mut notes = self.notes.clone();
                        notes.push(new_note);
                        notes
                    },
                    editing_note_id: self.editing_note_id.clone(),
                    editing_content: self.editing_content.clone(),
                    selected_note_id: self.selected_note_id.clone(),
                })
            }
            StickyNotesAction::StartEdit(note_id) => {
                let editing_content = self
                    .notes
                    .iter()
                    .find(|n| n.id == note_id)
                    .map(|n| n.content.clone());
                Rc::new(StickyNotesState {
                    editing_note_id: Some(note_id),
                    editing_content,
                    notes: self.notes.clone(),
                    selected_note_id: self.selected_note_id.clone(),
                })
            }
            StickyNotesAction::UpdateContent(content) => Rc::new(StickyNotesState {
                editing_content: Some(content),
                notes: self.notes.clone(),
                editing_note_id: self.editing_note_id.clone(),
                selected_note_id: self.selected_note_id.clone(),
            }),
            StickyNotesAction::SaveEdit => {
                if let Some(note_id) = self.editing_note_id.clone() {
                    if let Some(content) = self.editing_content.clone() {
                        let notes = self
                            .notes
                            .iter()
                            .cloned()
                            .map(|mut note| {
                                if note.id == note_id {
                                    note.content = content.clone();
                                }
                                note
                            })
                            .collect();
                        Rc::new(StickyNotesState {
                            notes,
                            editing_note_id: None,
                            editing_content: None,
                            selected_note_id: self.selected_note_id.clone(),
                        })
                    } else {
                        self
                    }
                } else {
                    self
                }
            }
            StickyNotesAction::CancelEdit => Rc::new(StickyNotesState {
                editing_note_id: None,
                editing_content: None,
                notes: self.notes.clone(),
                selected_note_id: self.selected_note_id.clone(),
            }),
            StickyNotesAction::UpdatePosition(note_id, new_position) => {
                let notes = self
                    .notes
                    .iter()
                    .cloned()
                    .map(|mut note| {
                        if note.id == note_id {
                            note.position = new_position.clone();
                        }
                        note
                    })
                    .collect();
                Rc::new(StickyNotesState {
                    notes,
                    editing_note_id: self.editing_note_id.clone(),
                    editing_content: self.editing_content.clone(),
                    selected_note_id: self.selected_note_id.clone(),
                })
            }
            StickyNotesAction::SelectNote(note_id) => Rc::new(StickyNotesState {
                selected_note_id: Some(note_id),
                notes: self.notes.clone(),
                editing_note_id: self.editing_note_id.clone(),
                editing_content: self.editing_content.clone(),
            }),
            StickyNotesAction::DeselectNote => Rc::new(StickyNotesState {
                selected_note_id: None,
                notes: self.notes.clone(),
                editing_note_id: self.editing_note_id.clone(),
                editing_content: self.editing_content.clone(),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub enum AppAction {
    View(ViewAction),
    Toolbar(ToolbarAction),
    StickyNotes(StickyNotesAction),
}

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: Rc<Self>, action: Self::Action) -> Rc<Self> {
        match action {
            AppAction::View(view_action) => {
                let new_view = ViewState::reduce(Rc::new(self.view.clone()), view_action);
                Rc::new(AppState {
                    view: (*new_view).clone(),
                    toolbar: self.toolbar.clone(),
                    sticky_notes: self.sticky_notes.clone(),
                })
            }
            AppAction::Toolbar(toolbar_action) => {
                let new_toolbar =
                    ToolbarState::reduce(Rc::new(self.toolbar.clone()), toolbar_action);
                Rc::new(AppState {
                    view: self.view.clone(),
                    toolbar: (*new_toolbar).clone(),
                    sticky_notes: self.sticky_notes.clone(),
                })
            }
            AppAction::StickyNotes(sticky_notes_action) => {
                let new_sticky_notes = StickyNotesState::reduce(
                    Rc::new(self.sticky_notes.clone()),
                    sticky_notes_action,
                );
                Rc::new(AppState {
                    view: self.view.clone(),
                    toolbar: self.toolbar.clone(),
                    sticky_notes: (*new_sticky_notes).clone(),
                })
            }
        }
    }
}
