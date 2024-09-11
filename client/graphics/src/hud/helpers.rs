use egui::menu::{BarState, MenuRoot};
use egui::{Id, InnerResponse, Response, Ui};

const CONTEXT_MENU_ID_STR: &str = "__egui::context_menu";

pub(crate) fn primary_menu(
    response: &Response,
    add_contents: impl FnOnce(&mut Ui),
) -> Option<InnerResponse<()>> {
    let menu_id = Id::new(CONTEXT_MENU_ID_STR);
    let mut bar_state = BarState::load(&response.ctx, menu_id);

    // Change from https://github.com/emilk/egui/blob/master/crates/egui/src/menu.rs#L270:
    MenuRoot::stationary_click_interaction(response, &mut bar_state);

    let inner_response = bar_state.show(response, add_contents);

    bar_state.store(&response.ctx, menu_id);
    inner_response
}
