use std::path::PathBuf;

use crate::{entity::Entity, layout::cache::GeoChanged};
use vizia_input::{Code, Key, MouseButton};
use vizia_style::CursorIcon;
use vizia_window::{Position, WindowSize};

#[derive(Debug, Clone)]
pub enum DropData {
    File(PathBuf),
    Id(Entity),
}

impl From<Entity> for DropData {
    fn from(value: Entity) -> Self {
        DropData::Id(value)
    }
}

impl From<PathBuf> for DropData {
    fn from(value: PathBuf) -> Self {
        DropData::File(value)
    }
}

/// Events generated by the application in response to OS events as well as events that can be used
/// to set properties of the window.
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// Emitted when a window is closed. Can also be emitted by a view or model to close the window.
    WindowClose,
    /// Emitted when a file is dragged and then dropped onto the window.
    Drop(DropData),
    /// Emitted when a mouse button is double clicked.
    MouseDoubleClick(MouseButton),
    /// Emitted when a mouse button is triple clicked
    MouseTripleClick(MouseButton),
    /// Emitted when a mouse button is pressed
    MouseDown(MouseButton),
    /// Emitted when a mouse button is released.
    MouseUp(MouseButton),
    /// Emitted when the primary mouse button or trigger key is pressed and then released on a view
    Press {
        mouse: bool,
    },
    /// Emitted when the primary mouse button or trigger key is pressed on a view
    PressDown {
        mouse: bool,
    },
    /// Emitted when the mouse cursor is moved
    MouseMove(f32, f32),
    /// Emitted when the mouse scroll wheel is scrolled.
    MouseScroll(f32, f32),
    /// Emitted when the mouse cursor enters the bounding box of an entity.
    MouseOver,
    /// Emitted when the mouse cursor leaves the bounding box of an entity.
    MouseOut,
    /// Emitted when the mouse cursor enters an entity.
    MouseEnter,
    /// Emitted when the mouse cursor leaves an entity.
    MouseLeave,
    // Emitted when an entity gains keyboard focus.
    FocusIn,
    // Emitted when an entity loses keyboard focus.
    FocusOut,
    /// Emitted when a character is typed.
    CharInput(char),
    /// Emitted when a keyboard key is pressed.
    KeyDown(Code, Option<Key>),
    /// Emitted when a keyboard key is released.
    KeyUp(Code, Option<Key>),
    /// Sets the mouse cursor icon.
    SetCursor(CursorIcon),
    /// Grabs the mouse cursor, preventing it from leaving the window.
    GrabCursor(bool),
    /// Sets the (x,y) position of the mouse cursor in window coordinates.
    SetCursorPosition(u32, u32),
    /// Sets the title of the window.
    SetTitle(String),
    /// Sets the size of the window.
    SetSize(WindowSize),
    /// Sets the position of the window.
    SetPosition(Position),
    /// Sets the maximum size of the window.
    SetMaxSize(Option<WindowSize>),
    /// Sets the minimum size of the window.
    SetMinSize(Option<WindowSize>),
    /// Sets whether the window is resizable.
    SetResizable(bool),
    /// Sets whether the window is minimized.
    SetMinimized(bool),
    /// Sets whether the window is maximized.
    SetMaximized(bool),
    /// Sets whether the window is visible.
    SetVisible(bool),
    /// Sets whether the window has decorations.
    SetDecorations(bool),
    /// Sets whether the window remains on top of other windows.
    SetAlwaysOnTop(bool),
    /// Emitted when mouse events have been captured.
    MouseCaptureEvent,
    /// Emitted when mouse events have been released.
    MouseCaptureOutEvent,
    // TODO: check if this includes margins + borders.
    /// Emitted when an entity changes position or size.
    GeometryChanged(GeoChanged),
    /// Requests a redraw of the window contents.
    Redraw,
    /// Request a restyle.
    Restyle,
    /// Requests a relayout.
    Relayout,
    /// Prints the debug message to the console.
    Debug(String),
    ActionRequest(accesskit::ActionRequest),
    /// Reloads all application stylesheets.
    ReloadStyles,
}
