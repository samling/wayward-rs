use relm4::gtk;

use super::Bar;
use super::state::BarItemState;
use crate::shell::ShellMsg;

pub(crate) trait BarWidget: Sync {
    fn id(&self) -> &'static str;

    fn render(&self, bar: &Bar, container: &gtk::Box);

    fn initial_state(&self) -> Option<BarItemState> {
        None
    }

    fn start(&self, _sender: relm4::Sender<ShellMsg>) -> Option<relm4::JoinHandle<()>> {
        None
    }
}
