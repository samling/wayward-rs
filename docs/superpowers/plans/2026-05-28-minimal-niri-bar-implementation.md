# Minimal Niri Workspace Bar Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a minimal Rust/Relm4 bar that connects to Niri through `wayle-niri` and renders all workspaces read-only.

**Architecture:** Keep Niri-specific code at the edge in `src/niri.rs`, convert it into local workspace data in `src/workspace.rs`, and let `src/bar.rs` render only local data. This keeps the UI easy to understand while you learn Rust.

**Tech Stack:** Rust 2021, Cargo, Relm4 0.11, GTK4, gtk4-layer-shell, wayle-niri, futures, tracing.

---

## Learning Notes

- A **crate** is Rust's compilation unit. For this project, the crate is the app named `wayward`.
- `Cargo.toml` is roughly like `go.mod` plus package metadata.
- A **module** is Rust's way to split code across files. `mod workspace;` in `main.rs` makes `src/workspace.rs` part of the crate.
- A **struct** is close to a Go struct.
- `Vec<T>` is a growable list of `T`. Think "Go slice that owns its backing array".
- `Option<T>` means "either `Some(value)` or `None`". It replaces nil for optional values.
- `Result<T, E>` means "either `Ok(value)` or `Err(error)`". It replaces many exception-style flows.
- An **adapter** is a boundary layer. Here it means code that translates `wayle-niri` data into your app's own `WorkspaceSummary` type.
- `async fn` returns work that can be awaited. It is similar in purpose to Go goroutines plus channels, but Rust makes waiting explicit with `.await`.
- `clone()` makes an owned copy. Rust is strict about ownership, so small app data often gets cloned at UI boundaries.

## File Structure

- Create `Cargo.toml`: package metadata and dependencies.
- Create `src/main.rs`: start logging, load CSS, run the Relm4 app.
- Create `src/workspace.rs`: local workspace model, label logic, CSS class logic, and tests.
- Create `src/bar.rs`: Relm4 component that owns UI state and renders workspace labels.
- Create `src/niri.rs`: adapter that connects to `NiriService`, watches workspace updates, and sends messages to the bar.

## Task 1: Create The Rust Package

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`

- [ ] **Step 1: Write the package manifest**

Create `Cargo.toml`:

```toml
[package]
name = "wayward"
version = "0.1.0"
edition = "2021"

[dependencies]
futures = "0.3"
gtk4-layer-shell = "0.8"
relm4 = "0.11"
tracing = "0.1"
tracing-subscriber = "0.3"
wayle-niri = "0.1"
```

Rust concept: dependencies are called **crates**. Cargo downloads crates from crates.io and writes exact resolved versions into `Cargo.lock`.

- [ ] **Step 2: Add a tiny app entry point**

Create `src/main.rs`:

```rust
fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");
}
```

Rust concept: `main` is the binary entry point, just like Go's `func main()`.

- [ ] **Step 3: Check that the crate builds**

Run:

```bash
cargo check
```

Expected: Cargo downloads dependencies and finishes with a line containing `Finished`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "Create Rust package"
```

## Task 2: Model Workspace Data Locally

**Files:**
- Create: `src/workspace.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Add the workspace module declaration**

Replace `src/main.rs` with:

```rust
mod workspace;

fn main() {
    tracing_subscriber::fmt::init();
    tracing::info!("wayward starting");
}
```

Rust concept: `mod workspace;` tells the compiler to load `src/workspace.rs`.

- [ ] **Step 2: Write the local workspace type and tests**

Create `src/workspace.rs`:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceSummary {
    pub id: u64,
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_urgent: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawWorkspace {
    pub id: u64,
    pub idx: u8,
    pub name: Option<String>,
    pub output: Option<String>,
    pub is_active: bool,
    pub is_focused: bool,
    pub is_urgent: bool,
}

impl WorkspaceSummary {
    pub fn label(&self) -> String {
        self.name
            .as_ref()
            .filter(|name| !name.is_empty())
            .cloned()
            .unwrap_or_else(|| self.idx.to_string())
    }

    pub fn css_classes(&self) -> Vec<&'static str> {
        let mut classes = vec!["workspace"];

        if self.is_active {
            classes.push("active");
        }

        if self.is_focused {
            classes.push("focused");
        }

        if self.is_urgent {
            classes.push("urgent");
        }

        classes
    }
}

impl From<RawWorkspace> for WorkspaceSummary {
    fn from(workspace: RawWorkspace) -> Self {
        Self {
            id: workspace.id,
            idx: workspace.idx,
            name: workspace.name,
            output: workspace.output,
            is_active: workspace.is_active,
            is_focused: workspace.is_focused,
            is_urgent: workspace.is_urgent,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_prefers_name() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: Some("code".to_string()),
            output: Some("DP-1".to_string()),
            is_active: true,
            is_focused: true,
            is_urgent: false,
        };

        assert_eq!(workspace.label(), "code");
    }

    #[test]
    fn label_falls_back_to_index() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: None,
            output: Some("DP-1".to_string()),
            is_active: true,
            is_focused: false,
            is_urgent: false,
        };

        assert_eq!(workspace.label(), "3");
    }

    #[test]
    fn css_classes_include_state() {
        let workspace = WorkspaceSummary {
            id: 10,
            idx: 3,
            name: None,
            output: None,
            is_active: true,
            is_focused: true,
            is_urgent: true,
        };

        assert_eq!(
            workspace.css_classes(),
            vec!["workspace", "active", "focused", "urgent"]
        );
    }
}
```

Rust concept: `impl` adds methods to a type. `From<RawWorkspace>` means Rust can convert a `RawWorkspace` into a `WorkspaceSummary`.

- [ ] **Step 3: Run the model tests**

Run:

```bash
cargo test workspace
```

Expected: output contains `3 passed`.

- [ ] **Step 4: Commit**

```bash
git add src/main.rs src/workspace.rs
git commit -m "Add workspace view model"
```

## Task 3: Render A Relm4 Bar With Fake Data

**Files:**
- Create: `src/bar.rs`
- Modify: `src/main.rs`

- [ ] **Step 1: Create the bar component**

Create `src/bar.rs`:

```rust
use gtk::prelude::*;
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use relm4::gtk;
use relm4::prelude::*;

use crate::workspace::WorkspaceSummary;

pub struct Bar {
    workspaces: Vec<WorkspaceSummary>,
    status: Option<String>,
}

#[derive(Debug)]
pub enum BarMsg {
    WorkspacesChanged(Vec<WorkspaceSummary>),
    NiriUnavailable(String),
    UpdatesStopped,
}

#[relm4::component(pub)]
impl SimpleComponent for Bar {
    type Init = ();
    type Input = BarMsg;
    type Output = ();

    view! {
        gtk::ApplicationWindow {
            set_title: Some("Wayward"),
            set_default_height: 32,
            set_resizable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_start: 8,
                set_margin_end: 8,
                set_margin_top: 4,
                set_margin_bottom: 4,

                #[name = "workspace_row"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 4,
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        root.init_layer_shell();
        root.set_layer(Layer::Top);
        root.set_anchor(Edge::Top, true);
        root.set_anchor(Edge::Left, true);
        root.set_anchor(Edge::Right, true);
        root.auto_exclusive_zone_enable();
        root.set_keyboard_mode(KeyboardMode::None);
        root.set_namespace(Some("wayward"));

        let model = Bar {
            workspaces: vec![
                WorkspaceSummary {
                    id: 1,
                    idx: 1,
                    name: None,
                    output: Some("fake".to_string()),
                    is_active: true,
                    is_focused: true,
                    is_urgent: false,
                },
                WorkspaceSummary {
                    id: 2,
                    idx: 2,
                    name: Some("code".to_string()),
                    output: Some("fake".to_string()),
                    is_active: false,
                    is_focused: false,
                    is_urgent: false,
                },
            ],
            status: None,
        };
        let widgets = view_output!();
        model.render_workspace_row(&widgets.workspace_row);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BarMsg::WorkspacesChanged(workspaces) => {
                self.workspaces = workspaces;
                self.status = None;
            }
            BarMsg::NiriUnavailable(error) => {
                self.workspaces.clear();
                self.status = Some(format!("Niri unavailable: {error}"));
            }
            BarMsg::UpdatesStopped => {
                self.status = Some("Niri updates stopped".to_string());
            }
        }

        self.render_workspace_row(&widgets.workspace_row);
        self.update_view(widgets, sender);
    }
}

impl Bar {
    fn render_workspace_row(&self, row: &gtk::Box) {
        while let Some(child) = row.first_child() {
            row.remove(&child);
        }

        if let Some(status) = &self.status {
            let label = gtk::Label::new(Some(status));
            label.add_css_class("status");
            row.append(&label);
            return;
        }

        for workspace in &self.workspaces {
            let label = gtk::Label::new(Some(&workspace.label()));

            for class_name in workspace.css_classes() {
                label.add_css_class(class_name);
            }

            row.append(&label);
        }
    }
}
```

Rust concept: `enum BarMsg` is a tagged union. It is a good fit for UI messages because every update has one clear shape.

- [ ] **Step 2: Start the Relm4 app**

Replace `src/main.rs` with:

```rust
mod bar;
mod workspace;

use relm4::RelmApp;

const CSS: &str = r#"
window {
    background: #202124;
    color: #f1f3f4;
    font: 12px sans-serif;
}

.workspace {
    padding: 3px 8px;
    border-radius: 4px;
    background: #3c4043;
}

.workspace.active {
    background: #5f6368;
}

.workspace.focused {
    background: #8ab4f8;
    color: #202124;
}

.workspace.urgent {
    background: #f28b82;
    color: #202124;
}

.status {
    color: #fdd663;
}
"#;

fn main() {
    tracing_subscriber::fmt::init();

    let app = RelmApp::new("dev.sboynton.wayward");
    app.set_global_css(CSS);
    app.run::<bar::Bar>(());
}
```

Rust concept: `const CSS: &str` is a compile-time string slice. `&str` is a borrowed string view, while `String` is an owned growable string.

- [ ] **Step 3: Build check**

Run:

```bash
cargo check
```

Expected: output contains `Finished`.

- [ ] **Step 4: Manual visual check**

Run this inside a Wayland session:

```bash
cargo run
```

Expected: a top bar appears with fake workspaces `1` and `code`.

- [ ] **Step 5: Commit**

```bash
git add src/main.rs src/bar.rs
git commit -m "Render fake workspace bar"
```

## Task 4: Connect The Bar To Niri

**Files:**
- Create: `src/niri.rs`
- Modify: `src/main.rs`
- Modify: `src/bar.rs`
- Modify: `src/workspace.rs`

- [ ] **Step 1: Add conversion from wayle-niri workspace data**

Append this implementation to `src/workspace.rs`:

```rust
impl WorkspaceSummary {
    pub fn from_wayle_workspace(workspace: &wayle_niri::core::Workspace) -> Self {
        Self {
            id: workspace.id.get(),
            idx: workspace.idx.get(),
            name: workspace.name.get(),
            output: workspace.output.get(),
            is_active: workspace.is_active.get(),
            is_focused: workspace.is_focused.get(),
            is_urgent: workspace.is_urgent.get(),
        }
    }
}
```

Rust concept: `&wayle_niri::core::Workspace` borrows the workspace. Borrowing lets this function read data without taking ownership of the source value.

- [ ] **Step 2: Write the Niri adapter**

Create `src/niri.rs`:

```rust
use futures::StreamExt;
use relm4::Sender;
use wayle_niri::NiriService;

use crate::bar::BarMsg;
use crate::workspace::WorkspaceSummary;

pub async fn run_workspace_watcher(sender: Sender<BarMsg>) {
    let service = match NiriService::new().await {
        Ok(service) => service,
        Err(error) => {
            let _ = sender.send(BarMsg::NiriUnavailable(error.to_string()));
            return;
        }
    };

    send_workspace_snapshot(&sender, &service);

    let mut updates = service.workspaces.watch();
    while let Some(workspaces) = updates.next().await {
        let summaries = workspaces
            .values()
            .map(WorkspaceSummary::from_wayle_workspace)
            .collect();

        if sender.send(BarMsg::WorkspacesChanged(summaries)).is_err() {
            return;
        }
    }

    let _ = sender.send(BarMsg::UpdatesStopped);
}

fn send_workspace_snapshot(sender: &Sender<BarMsg>, service: &NiriService) {
    let summaries = service
        .workspaces
        .get()
        .values()
        .map(WorkspaceSummary::from_wayle_workspace)
        .collect();

    let _ = sender.send(BarMsg::WorkspacesChanged(summaries));
}
```

Rust concept: `.map(...).collect()` transforms an iterator into a collection. Here the collection type is inferred as `Vec<WorkspaceSummary>` because `BarMsg::WorkspacesChanged` requires that type.

- [ ] **Step 3: Register the Niri module**

Replace the module declarations at the top of `src/main.rs` with:

```rust
mod bar;
mod niri;
mod workspace;
```

Rust concept: modules are private by default. Other modules can still use public items from sibling modules through `crate::niri` because they live in the same crate.

- [ ] **Step 4: Start the watcher from the bar**

In `src/bar.rs`, first rename the `init` argument `_sender` to `sender`:

```rust
    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
```

Then replace the fake `model` block in `init` with this:

```rust
        let model = Bar {
            workspaces: Vec::new(),
            status: Some("Connecting to Niri".to_string()),
        };
        let widgets = view_output!();
        model.render_workspace_row(&widgets.workspace_row);

        let input_sender = sender.input_sender().clone();
        relm4::spawn(async move {
            crate::niri::run_workspace_watcher(input_sender).await;
        });

        ComponentParts { model, widgets }
```

Rust concept: `move` transfers captured values into the async task. This is how the background watcher keeps its own sender after `init` returns.

- [ ] **Step 5: Build check**

Run:

```bash
cargo check
```

Expected: output contains `Finished`.

- [ ] **Step 6: Run tests**

Run:

```bash
cargo test
```

Expected: output contains `3 passed`.

- [ ] **Step 7: Manual Niri check**

Run this inside Niri:

```bash
cargo run
```

Expected: the bar shows your real Niri workspaces, and the focused workspace changes styling when focus changes.

- [ ] **Step 8: Commit**

```bash
git add src/main.rs src/bar.rs src/niri.rs src/workspace.rs
git commit -m "Connect workspace bar to Niri"
```

## Task 5: Rust Review Checkpoint

**Files:**
- Review: `src/main.rs`
- Review: `src/bar.rs`
- Review: `src/niri.rs`
- Review: `src/workspace.rs`

- [ ] **Step 1: Run formatting**

Run:

```bash
cargo fmt
```

Expected: command exits successfully with no output.

- [ ] **Step 2: Run compiler checks**

Run:

```bash
cargo check
```

Expected: output contains `Finished`.

- [ ] **Step 3: Run tests**

Run:

```bash
cargo test
```

Expected: output contains `3 passed`.

- [ ] **Step 4: Read the code aloud by responsibility**

Use this checklist:

- `workspace.rs` should not know about GTK or Relm4.
- `bar.rs` should not know how Niri IPC works.
- `niri.rs` should not create GTK widgets.
- `main.rs` should only wire the app together.

Rust concept: this is **separation of concerns**. It is not Rust-specific, but Rust's module system makes these boundaries visible.

- [ ] **Step 5: Commit formatting or small review fixes**

Run:

```bash
git status --short
```

If files are listed, commit them:

```bash
git add Cargo.toml Cargo.lock src/main.rs src/bar.rs src/niri.rs src/workspace.rs
git commit -m "Polish minimal workspace bar"
```

If no files are listed, skip this commit.

## References

- `wayle-niri` docs: https://docs.rs/wayle-niri/latest/wayle_niri/
- `RelmApp` docs: https://relm4.org/docs/stable/relm4/struct.RelmApp.html
- `ComponentSender` docs: https://relm4.org/docs/next/relm4/struct.ComponentSender.html
- `gtk4-layer-shell` docs: https://docs.rs/gtk4-layer-shell/latest/gtk4_layer_shell/trait.LayerShell.html
